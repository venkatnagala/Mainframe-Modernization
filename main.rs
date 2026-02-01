use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use aws_sdk_s3::Client as S3Client;
//use aws_config::endpoint::Endpoint;
//use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream; // Updated for SDK 0.25.1
use reqwest_middleware::{ClientBuilder};       //{}, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
use serde_json::json;
use anyhow::{Context, Result};
use chrono::Utc;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[derive(Deserialize)]
struct SourceLocation { bucket: String, key: String }

#[derive(Deserialize)]
struct EvalRequest { task_id: String, source_location: SourceLocation }

#[derive(Serialize)]
struct EvalResponse { task_id: String, status: String, match_confirmed: bool }

#[post("/evaluate")]
async fn handle_start_eval(req: web::Json<EvalRequest>) -> impl Responder {
    let task_id = req.task_id.clone();
    println!("📡 [TASK: {}] Starting Evaluation...", task_id);

    match process_modernization(&req).await {
        Ok(_) => {
            println!("✅ [TASK: {}] Success.", task_id);
            HttpResponse::Ok().json(EvalResponse {
                task_id, status: "SUCCESS".to_string(), match_confirmed: true,
            })
        },
        Err(e) => {
            eprintln!("❌ [TASK: {}] Error: {:?}", task_id, e);
            HttpResponse::InternalServerError().json(EvalResponse {
                task_id, status: format!("{:?}", e), match_confirmed: false,
            })
        }
    }
}

async fn process_modernization(req: &EvalRequest) -> Result<()> {
    // 1. Manually pull variables from the env
    let ak = std::env::var("AWS_ACCESS_KEY_ID")?;
    let sk = std::env::var("AWS_SECRET_ACCESS_KEY")?;
    
    // 2. Create credentials explicitly
    let credentials = aws_sdk_s3::config::Credentials::new(ak, sk, None, None, "manual");
    
    // 3. Build the config WITHOUT loading from env (to avoid conflicts)
    let s3_config = aws_sdk_s3::config::Builder::new()
        .region(aws_sdk_s3::config::Region::new("eu-west-1"))
        .credentials_provider(credentials)
        .endpoint_url("https://s3.eu-west-1.amazonaws.com")
        // Force path style one last time - it helps with AccessDenied in some accounts
        .force_path_style(true) 
        .build();
    
    let s3_client = S3Client::from_conf(s3_config);

    // 4. Fetch COBOL
    let data = s3_client.get_object()
        .bucket(&req.source_location.bucket)
        .key(&req.source_location.key)
        .send().await
        .context("S3 Access Denied. Last check: Is 'Block All Public Access' actually OFF?")?;
    
    let bytes = data.body.collect().await?.to_vec();
    let cobol = String::from_utf8(bytes)?;

    // 5. Modernize 
    println!("🤖 Invoking Gemini 3 Pro (High Thinking)...");
    let (rust_code, raw_logs) = translate_with_gemini_detailed(&cobol).await?;
    let ts = Utc::now().timestamp();

    // 6. Archive (Using the same client)
    archive_with_client(&s3_client, &req.source_location.bucket, &format!("modernized/{}_{}.rs", req.task_id, ts), &rust_code).await?;
    archive_with_client(&s3_client, &req.source_location.bucket, &format!("raw_logs/{}_{}.json", req.task_id, ts), &raw_logs).await?;

    Ok(())
}

async fn translate_with_gemini_detailed(cobol: &str) -> Result<(String, String)> {
    let api_key = std::env::var("GEMINI_API_KEY").context("GEMINI_API_KEY missing")?;
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(ExponentialBackoff::builder().build_with_max_retries(3)))
        .build();

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-3-pro-preview:generateContent?key={}", api_key);
    let encoded_source = STANDARD.encode(cobol);

    let payload = json!({
        "system_instruction": {
            "parts": [{ "text": "Modernize the input to idiomatic Rust. Return result as Base64 in 'modernized_rust_b64'." }]
        },
        "contents": [{ "role": "user", "parts": [{ "text": format!("Modernize logic: \n{}\n", encoded_source) }] }],
        "generationConfig": {
            "temperature": 1.0, 
            "responseMimeType": "application/json",
            "responseJsonSchema": {
                "type": "object",
                "properties": { "modernized_rust_b64": { "type": "string" }, "notes": { "type": "string" } },
                "required": ["modernized_rust_b64"]
            },
            "thinking_config": { "include_thoughts": true, "thinking_level": "high" }
        }
    });

    let resp = client.post(url).json(&payload).send().await?;
    
    // 1. Capture the status code BEFORE consuming the body
    let status = resp.status();
    
    // 2. Consume the body text
    let raw_text = resp.text().await?;

    // 3. Now check the status using the stored variable
    if !status.is_success() { 
        return Err(anyhow::anyhow!("API Error {}: {}", status, raw_text)); 
    }

    let json: serde_json::Value = serde_json::from_str(&raw_text)?;
    let text_content = json["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("{}");
    let inner_json: serde_json::Value = serde_json::from_str(text_content)?;
    
    let b64_rust = inner_json["modernized_rust_b64"].as_str().ok_or_else(|| anyhow::anyhow!("B64 missing"))?;
    let rust_code = String::from_utf8(STANDARD.decode(b64_rust.trim())?)?;

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
    println!("🛡️ Green Agent [GEMINI 3 PRO PREVIEW] Online at 0.0.0.0:8080");

    HttpServer::new(|| {
        App::new().wrap(Cors::permissive()).service(handle_start_eval)
    })
    .bind(("0.0.0.0", 8080))?.run().await
}