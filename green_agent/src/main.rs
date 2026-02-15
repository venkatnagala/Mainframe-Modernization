use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
use serde_json::json;
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use std::fs;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

#[derive(Deserialize)]
struct SourceLocation { 
    bucket: String, 
    key: String 
}

#[derive(Deserialize)]
struct EvalRequest { 
    task_id: String, 
    source_location: SourceLocation 
}

#[derive(Serialize)]
struct EvalResponse { 
    task_id: String, 
    status: String, 
    match_confirmed: bool,
    rust_code_url: Option<String>,
    logs_url: Option<String>,
}

#[post("/evaluate")]
async fn handle_start_eval(req: web::Json<EvalRequest>) -> impl Responder {
    let task_id = req.task_id.clone();
    println!("📡 [TASK: {}] Starting Evaluation...", task_id);

    match process_modernization(&req).await {
        Ok(response) => {
            println!("✅ [TASK: {}] {}", task_id, response.status);
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            eprintln!("❌ [TASK: {}] Error: {:?}", task_id, e);
            HttpResponse::InternalServerError().json(EvalResponse {
                task_id, 
                status: format!("{:?}", e), 
                match_confirmed: false,
                rust_code_url: None,
                logs_url: None,
            })
        }
    }
}

async fn process_modernization(req: &EvalRequest) -> Result<EvalResponse> {
    // 1. Setup S3 client
    let ak = std::env::var("AWS_ACCESS_KEY_ID")?;
    let sk = std::env::var("AWS_SECRET_ACCESS_KEY")?;
    let credentials = aws_sdk_s3::config::Credentials::new(ak, sk, None, None, "manual");
    
    let s3_config = aws_sdk_s3::config::Builder::new()
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .credentials_provider(credentials)
        .build();

    let s3_client = S3Client::from_conf(s3_config);
    
    // 2. Fetch source code (COBOL or Assembler)
    println!("📥 Fetching source from S3: {}", req.source_location.key);
    let data = s3_client.get_object()
        .bucket(&req.source_location.bucket)
        .key(&req.source_location.key)
        .send().await
        .context("S3 Access Denied")?;
    
    let bytes = data.body.collect().await?.to_vec();
    let source_code = String::from_utf8(bytes)?;
    println!("✅ Fetched {} bytes of source code", source_code.len());
    
    // 3. Detect language
    let is_cobol = req.source_location.key.ends_with(".cbl") || 
                   req.source_location.key.ends_with(".cob");
    let language = if is_cobol { "COBOL" } else { "Assembler" };

    // 4. Fetch input data
    let input_data = fetch_input_data(&s3_client, &req.source_location.bucket).await?;
    println!("📥 Input data: '{}'", input_data);
    
    // 5. Modernize with Gemini
    println!("🤖 Invoking Gemini-2.5-pro for {} modernization...", language);
    let (rust_code, raw_logs) = translate_with_gemini_detailed(&source_code, language).await?;
    
    let ts = Utc::now().timestamp();
    let task_id = &req.task_id;
    
    // 6. Validate if COBOL
    let (status, match_confirmed, rust_folder) = if is_cobol {
        validate_cobol_modernization(&source_code, &rust_code, &input_data).await?
    } else {
        validate_assembler_modernization(&rust_code).await?
    };
    
    // 7. Save to S3
    let rust_key = format!("{}/{}_{}.rs", rust_folder, task_id, ts);
    let logs_key = format!("raw_logs/{}_{}.json", task_id, ts);
        
    println!("💾 Saving to S3: {}", rust_key);
    archive_with_client(&s3_client, &req.source_location.bucket, &rust_key, &rust_code).await?;
    archive_with_client(&s3_client, &req.source_location.bucket, &logs_key, &raw_logs).await?;
    println!("✅ Saved to S3");
    
    // 8. Return response (with pre-signed URLs)
    // Generate pre-signed URLs
    let rust_url = generate_presigned_url(&s3_client, &req.source_location.bucket, &rust_key).await.ok();
    let logs_url = generate_presigned_url(&s3_client, &req.source_location.bucket, &logs_key).await.ok();
    
        Ok(EvalResponse {
        task_id: task_id.clone(),
        status,
        match_confirmed,
        rust_code_url: rust_url,
        logs_url: logs_url,
    })
    
}

async fn fetch_input_data(client: &S3Client, bucket: &str) -> Result<String> {
    let data = client.get_object()
        .bucket(bucket)
        .key("data/loan_data.json")
        .send()
        .await?;
    
    let bytes = data.body.collect().await?.to_vec();
    let json_str = String::from_utf8(bytes)?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;
    
    // Convert "001000000" cents to "10000.00" dollars
    let loan_cents = json["loan_amount"].as_str().unwrap_or("0");
    let loan_decimal = format!("{}.{:02}", 
        loan_cents[..loan_cents.len().saturating_sub(2)].trim_start_matches('0').parse::<i64>().unwrap_or(0),
        &loan_cents[loan_cents.len().saturating_sub(2)..]
    );
    
    Ok(loan_decimal)
}

async fn generate_presigned_url(
    client: &S3Client, 
    bucket: &str, 
    key: &str
) -> Result<String> {
    let presigned_request = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(3600))? // 1 hour
        )
        .await?;
    
    Ok(presigned_request.uri().to_string())
}

async fn validate_cobol_modernization(
    cobol_source: &str, 
    rust_code: &str, 
    input_data: &str
) -> Result<(String, bool, String)> {
    println!("🔨 Compiling and executing COBOL...");
    
    // Write input file
    fs::write("/tmp/input.txt", format!("{}\n", input_data))?;
    println!("📝 Wrote input.txt: '{}'", input_data);
    
    // Compile COBOL (fixed format, no -free flag)
    fs::write("/tmp/program.cbl", cobol_source)?;
    let cobol_compile = Command::new("cobc")
        .args(&["-x", "-std=ibm", "/tmp/program.cbl", "-o", "/tmp/cobol_prog"])
        .output()?;
    
    if !cobol_compile.status.success() {
        let error = String::from_utf8_lossy(&cobol_compile.stderr);
        return Err(anyhow::anyhow!("COBOL compilation failed: {}", error));
    }
    
    // Run COBOL
    let cobol_run = Command::new("/tmp/cobol_prog")
        .current_dir("/tmp")
        .output()?;
    
    // After running COBOL, capture ALL output
    let cobol_run = Command::new("/tmp/cobol_prog")
    .current_dir("/tmp")
    .output()?;

        println!("📋 COBOL stdout (DISPLAY output): '{}'", String::from_utf8_lossy(&cobol_run.stdout));
        println!("📋 COBOL stderr: '{}'", String::from_utf8_lossy(&cobol_run.stderr));

    let cobol_output_file = fs::read_to_string("/tmp/output.txt")?;
        println!("📄 COBOL output.txt: '{}'", cobol_output_file.trim());

    // Also show what's IN the input file COBOL is reading
    let input_check = fs::read_to_string("/tmp/input.txt")?;
        println!("🔍 COBOL sees input.txt as: '{}'", input_check);
    
    if !cobol_run.status.success() {
        let error = String::from_utf8_lossy(&cobol_run.stderr);
        return Err(anyhow::anyhow!("COBOL execution failed: {}", error));
    }
    
    let cobol_output = fs::read_to_string("/tmp/output.txt")
        .unwrap_or_else(|_| String::from_utf8_lossy(&cobol_run.stdout).to_string());
    
    println!("📊 COBOL output: {}", cobol_output.trim());
    
    // Compile and run Rust
    println!("🦀 Compiling and executing Rust...");
    
    // Reset input for Rust
    fs::write("/tmp/input.txt", format!("{}\n", input_data))?;
    
    // Create Cargo project
    let project_dir = "/tmp/rust_project";
    let _ = fs::remove_dir_all(project_dir);
    fs::create_dir_all(&format!("{}/src", project_dir))?;

    //rust_decimal_macros = "1.36"
    //num-traits = "0.2"
    //num-format = "0.4"
    
    let cargo_toml = r#"[package]
                            name = "modernized"
                            version = "0.1.0"
                            edition = "2021"

                            [dependencies]
                            rust_decimal = "1.36"
                            rust_decimal_macros = "1.36"
                            "#;
    fs::write(&format!("{}/Cargo.toml", project_dir), cargo_toml)?;
    fs::write(&format!("{}/src/main.rs", project_dir), rust_code)?;
    
    println!("🔨 Building Rust project...");
    
    let rust_compile = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", &format!("{}/Cargo.toml", project_dir)])
        .current_dir(project_dir)
        .output()?;
    
    if !rust_compile.status.success() {
        let error = String::from_utf8_lossy(&rust_compile.stderr);
        println!("❌ Rust compilation failed: {}", error);
        return Ok((
            format!("Rust compilation failed: {}", error),
            false,
            "modernized/failed".to_string()
        ));
    }
    
    println!("✅ Rust compiled successfully");
    
    // Run Rust binary
    let rust_run = Command::new(&format!("{}/target/release/modernized", project_dir))
        .current_dir("/tmp")
        .output()?;
    
    let rust_output = fs::read_to_string("/tmp/output.txt")
        .unwrap_or_else(|_| String::from_utf8_lossy(&rust_run.stdout).to_string());
    
    println!("📊 Rust output: {}", rust_output.trim());
    
    // Compare outputs - normalize whitespace
    let normalize = |s: &str| -> String {
            s.split_whitespace()
            .collect::<Vec<_>>()
             .join(" ")
            };

    let cobol_normalized = normalize(&cobol_output);
    let rust_normalized = normalize(&rust_output);

    let matched = cobol_normalized.trim() == rust_normalized.trim();

        println!("🔍 Comparing outputs:");
        println!("   COBOL (normalized): '{}'", cobol_normalized);
        println!("   Rust (normalized):  '{}'", rust_normalized);
        println!("   Match: {}", matched);
    
    let (status, folder) = if matched {
        ("SUCCESS - Outputs match!".to_string(), "modernized/validated".to_string())
    } else {
        (format!("Output mismatch. COBOL: '{}', Rust: '{}'", 
                cobol_output.trim(), rust_output.trim()),
         "modernized/needs-review".to_string())
    };
    
    Ok((status, matched, folder))
}

async fn validate_assembler_modernization(rust_code: &str) -> Result<(String, bool, String)> {
    println!("🦀 Compiling Rust (Assembler source - no validation)...");
    
    let project_dir = "/tmp/rust_project";
    let _ = fs::remove_dir_all(project_dir);
    fs::create_dir_all(&format!("{}/src", project_dir))?;

    //rust_decimal_macros = "1.36"                            
    //num-traits = "0.2"
    //num-format = "0.4"

    let cargo_toml = r#"[package]
                            name = "modernized"
                            version = "0.1.0"
                            edition = "2021"

                            [dependencies]
                            rust_decimal = "1.36"
                            rust_decimal_macros = "1.36"                            
                            "#;
    fs::write(&format!("{}/Cargo.toml", project_dir), cargo_toml)?;
    fs::write(&format!("{}/src/main.rs", project_dir), rust_code)?;
    
    let rust_compile = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", &format!("{}/Cargo.toml", project_dir)])
        .current_dir(project_dir)
        .output()?;
    
    if !rust_compile.status.success() {
        let error = String::from_utf8_lossy(&rust_compile.stderr);
        return Ok((
            format!("Rust compilation failed: {}", error),
            false,
            "modernized/failed".to_string()
        ));
    }
    
    Ok((
        "Rust compiles successfully. Assembler validation requires mainframe access.".to_string(),
        false,
        "modernized".to_string()
    ))
}

async fn translate_with_gemini_detailed(source_code: &str, language: &str) -> Result<(String, String)> {
    let api_key = std::env::var("GEMINI_API_KEY").context("GEMINI_API_KEY missing")?;
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3)))
        .build();

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent?key={}", api_key);

    /*let system_instruction = format!(
        "Modernize the input {} code to idiomatic Rust. \
         Use file I/O: read from 'input.txt' and write to 'output.txt'. \
         \
         CRITICAL PARSING REQUIREMENTS: \
         - Input contains decimal numbers like '10000.00' \
         - MUST use rust_decimal::Decimal::from_str() for ALL number parsing \
         - NEVER use i64, u64, or integer parsing for financial data \
         - Example: use rust_decimal::Decimal; use std::str::FromStr; let amount = Decimal::from_str(input.trim()).unwrap(); \
         \
         For packed decimal (COMP-3), use rust_decimal::Decimal for exact arithmetic. \
         Keep the output format identical to the original program. \
         \
         Return the Rust code as plain text in 'modernized_rust'.",
        language
    ); */

    let system_instruction = format!(
    "Modernize the input {} code to idiomatic Rust. \
     Use file I/O: read from 'input.txt' and write to 'output.txt'. \
     \
     CRITICAL REQUIREMENTS: \
     - Use ONLY rust_decimal::Decimal for all numbers \
     - Parse input with: Decimal::from_str(input.trim()).unwrap() \
     - Format output with standard Rust format macro: format!(\"{{:.2}}\", value) \
     - DO NOT use num-format, to_formatted_string, or any formatting libraries \
     - Match exact output format: 'CALCULATED INTEREST: 550.00' \
     \
     Example output formatting: \
     let result = format!(\"CALCULATED INTEREST: {{:.2}}\", total_interest); \
     \
     Return the Rust code as plain text in 'modernized_rust'.",
    language
    );

    let payload = json!({
        "system_instruction": {
            "parts": [{ "text": system_instruction }]
        },
        "contents": [{ 
            "role": "user", 
            "parts": [{ "text": format!("Modernize this {} code to Rust:\n{}\n", language, source_code) }] 
        }],
        "generationConfig": {
            "temperature": 0.7,
            "responseMimeType": "application/json",
            "responseSchema": {
                "type": "object",
                "properties": { 
                    "modernized_rust": { "type": "string" },
                    "notes": { "type": "string" } 
                },
                "required": ["modernized_rust"]
            }
        }
    });

    let resp = client.post(url).json(&payload).send().await?;
    let status = resp.status();
    let raw_text = resp.text().await?;

    if !status.is_success() { 
        return Err(anyhow::anyhow!("API Error {}: {}", status, raw_text)); 
    }

    let json: serde_json::Value = serde_json::from_str(&raw_text)?;
    let text_content = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str().unwrap_or("{}");
    let inner_json: serde_json::Value = serde_json::from_str(text_content)?;
    
    let rust_code = inner_json["modernized_rust"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Rust code missing"))?
        .to_string();

    println!("✅ Generated {} lines of Rust code", rust_code.lines().count());

    Ok((rust_code, raw_text))
}

async fn archive_with_client(client: &S3Client, bucket: &str, key: &str, content: &str) -> Result<()> {
    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(content.as_bytes().to_vec()))
        .send()
        .await
        .context("S3 PutObject failed")?;
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    println!("🛡️ Green Agent [GEMINI-2.5-pro] Online at 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new().wrap(Cors::permissive()).service(handle_start_eval)
    })
    .bind(("0.0.0.0", 8080))?.run().await
}