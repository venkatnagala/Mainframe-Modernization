use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use reqwest::Client;
use std::net::SocketAddr;

#[derive(Deserialize)]
struct ModernizeRequest {
    cobol_code: String,
}

#[derive(Serialize)]
struct ModernizeResponse {
    modernized_rust: String,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/solve", post(handle_modernization));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("🟣 Purple Agent (AI Modernizer) Online | Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_modernization(Json(payload): Json<ModernizeRequest>) -> Json<ModernizeResponse> {
    println!("📖 Received COBOL for modernization...");

    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY must be set");
    let client = Client::new();

    let system_instructions = "
        You are a Senior Mainframe Modernization Engineer. 
        Convert the provided COBOL code into clean, idiomatic Rust.
        STRICT RULES:
        1. Output ONLY the raw Rust code. No markdown blocks, no explanations.
        2. Use standard Rust types (f64 or Decimal).
        3. Match the COBOL 'DISPLAY' logic exactly in println! statements.
        4. Ensure the Rust logic produces the same numeric result as the COBOL logic.
        5. Include necessary imports like 'use std::io;'.
    ";

    let prompt = format!("{}\n\nCOBOL SOURCE:\n{}", system_instructions, payload.cobol_code);

    let claude_request = ClaudeRequest {
        model: "claude-opus-4-6".to_string(),
        max_tokens: 32768,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let mut rust_output = String::from("// Error generating code");

    let res = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&claude_request)
        .send()
        .await;

    if let Ok(response) = res {
        if let Ok(claude_response) = response.json::<ClaudeResponse>().await {
            if let Some(content) = claude_response.content
                .into_iter()
                .find(|c| c.content_type == "text")
            {
                if let Some(text) = content.text {
                    rust_output = text.trim().to_string();
                }
            }
        }
    }

    // Clean potential markdown artifacts
    rust_output = rust_output
        .replace("```rust", "")
        .replace("```", "")
        .trim()
        .to_string();

    println!("✅ Modernization complete!");

    Json(ModernizeResponse {
        modernized_rust: rust_output,
    })
}