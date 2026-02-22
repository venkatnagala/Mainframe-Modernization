// Gemini MCP Server - Mainframe Modernization Pipeline
// Uses Google Gemini 2.5 Pro to translate COBOL to idiomatic Rust
// Endpoints:
//   POST /translate_cobol      - Translate COBOL source to Rust
//   POST /translate_assembler  - Translate Assembler to Rust
//   POST /explain_code         - Explain COBOL code in plain English
//   GET  /health               - Health check

use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use serde::{Deserialize, Serialize};
use log::{info, error};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent";

// ─── Request/Response Types ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TranslateRequest {
    pub source: String,           // COBOL or Assembler source code
    pub context: Option<String>,  // Optional additional context
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub success: bool,
    pub rust_code: Option<String>,
    pub explanation: Option<String>,
    pub model_used: String,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct ExplainRequest {
    pub source: String,
}

#[derive(Serialize)]
pub struct ExplainResponse {
    pub success: bool,
    pub explanation: Option<String>,
    pub error: Option<String>,
}

// Gemini API types
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

// ─── App State ────────────────────────────────────────────────────────────────

pub struct AppState {
    pub gemini_api_key: String,
    pub http_client: reqwest::Client,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// Translate COBOL source to idiomatic Rust using Gemini 2.5 Pro
async fn translate_cobol(
    state: web::Data<AppState>,
    body: web::Json<TranslateRequest>,
) -> HttpResponse {
    info!("Translating COBOL to Rust ({} chars)", body.source.len());

    let prompt = format!(
        r#"You are an expert COBOL and Rust programmer. Convert the following COBOL program to idiomatic, memory-safe Rust code.

Requirements:
1. The Rust code must produce IDENTICAL output to the COBOL program
2. Use idiomatic Rust (proper error handling, ownership, borrowing)
3. Handle COBOL packed decimals (COMP-3) correctly using Rust decimal arithmetic
4. Match numeric formatting exactly (same decimal places, spacing)
5. Return ONLY the complete Rust source code, no explanations
6. The Rust code must compile with standard cargo build
7. Include necessary use statements and a main() function

COBOL Source:
{}

{}

Return ONLY the Rust source code, starting with `use` statements or `fn main()`."#,
        body.source,
        body.context.as_deref().unwrap_or("")
    );

    match call_gemini(&state, &prompt).await {
        Ok(rust_code) => {
            // Clean up markdown code blocks if present
            let clean_code = rust_code
                .trim()
                .trim_start_matches("```rust")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
                .to_string();

            info!("Translation complete: {} chars of Rust generated", clean_code.len());
            HttpResponse::Ok().json(TranslateResponse {
                success: true,
                rust_code: Some(clean_code),
                explanation: None,
                model_used: "gemini-2.5-pro".to_string(),
                error: None,
            })
        }
        Err(e) => {
            error!("Gemini translation failed: {}", e);
            HttpResponse::InternalServerError().json(TranslateResponse {
                success: false,
                rust_code: None,
                explanation: None,
                model_used: "gemini-2.5-pro".to_string(),
                error: Some(e),
            })
        }
    }
}

/// Translate Assembler source to Rust
async fn translate_assembler(
    state: web::Data<AppState>,
    body: web::Json<TranslateRequest>,
) -> HttpResponse {
    info!("Translating Assembler to Rust ({} chars)", body.source.len());

    let prompt = format!(
        r#"You are an expert IBM mainframe Assembler (HLASM/BAL) and Rust programmer.
Convert the following Assembler program to idiomatic, memory-safe Rust code.

Requirements:
1. Produce IDENTICAL output to the Assembler program
2. Use idiomatic Rust with proper error handling
3. Handle register operations and memory layout correctly
4. Return ONLY the complete Rust source code

Assembler Source:
{}

Return ONLY the Rust source code."#,
        body.source
    );

    match call_gemini(&state, &prompt).await {
        Ok(rust_code) => {
            let clean_code = rust_code
                .trim()
                .trim_start_matches("```rust")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
                .to_string();

            HttpResponse::Ok().json(TranslateResponse {
                success: true,
                rust_code: Some(clean_code),
                explanation: None,
                model_used: "gemini-2.5-pro".to_string(),
                error: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(TranslateResponse {
            success: false,
            rust_code: None,
            explanation: None,
            model_used: "gemini-2.5-pro".to_string(),
            error: Some(e),
        }),
    }
}

/// Explain COBOL code in plain English
async fn explain_code(
    state: web::Data<AppState>,
    body: web::Json<ExplainRequest>,
) -> HttpResponse {
    let prompt = format!(
        "Explain what this COBOL program does in plain English:\n\n{}",
        body.source
    );

    match call_gemini(&state, &prompt).await {
        Ok(explanation) => HttpResponse::Ok().json(ExplainResponse {
            success: true,
            explanation: Some(explanation),
            error: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ExplainResponse {
            success: false,
            explanation: None,
            error: Some(e),
        }),
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "gemini-mcp",
        "version": "1.0.0",
        "model": "gemini-2.5-pro"
    }))
}

// ─── Gemini API Helper ────────────────────────────────────────────────────────

async fn call_gemini(state: &AppState, prompt: &str) -> Result<String, String> {
    let request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt.to_string() }],
        }],
        generation_config: GeminiConfig {
            temperature: 0.1,   // Low temperature for deterministic code generation
            max_output_tokens: 8192,
        },
    };

    let url = format!("{}?key={}", GEMINI_API_URL, state.gemini_api_key);

    let response = state.http_client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Gemini API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API error {}: {}", status, text));
    }

    let gemini_response: GeminiResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

    gemini_response.candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or("Empty response from Gemini".to_string())
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let gemini_api_key = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY must be set");

    let state = web::Data::new(AppState {
        gemini_api_key,
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap(),
    });

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or("0.0.0.0:8082".to_string());
    info!("🧠 Gemini MCP Service starting on {}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .route("/translate_cobol", web::post().to(translate_cobol))
            .route("/translate_assembler", web::post().to(translate_assembler))
            .route("/explain_code", web::post().to(explain_code))
            .route("/health", web::get().to(health))
    })
    .bind(&bind_addr)?
    .run()
    .await
}
