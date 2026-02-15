use axum::{routing::post, Json, Router}; // Fixed from ax_utils to axum
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

#[tokio::main]
async fn main() {
    let app = Router::new().route("/solve", post(handle_modernization));

    // Fix: Explicitly define the address to solve type inference errors
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("🧠 Purple Agent (AI Modernizer) Online | Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_modernization(Json(payload): Json<ModernizeRequest>) -> Json<ModernizeResponse> {
    println!("📖 Received COBOL for refactoring...");

    let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY must be set");
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

    let gemini_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let gemini_payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("{}\n\nCOBOL SOURCE:\n{}", system_instructions, payload.cobol_code)
            }]
        }]
    });

    let mut rust_output = String::from("// Error generating code");

    let res = client.post(gemini_url)
        .json(&gemini_payload)
        .send()
        .await;

    if let Ok(response) = res {
        let json: serde_json::Value = response.json().await.unwrap_or_default();
        if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            rust_output = text.trim().to_string();
        }
    }

    // Cleaning potential markdown artifacts
    rust_output = rust_output.replace("```rust", "").replace("```", "").trim().to_string();

    Json(ModernizeResponse {
        modernized_rust: rust_output,
    })
}