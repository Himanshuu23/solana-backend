use axum::{routing::post, Router, Json};
use axum::http::StatusCode;
use solana_sdk::signer::keypair::Keypair as SolanaKeypair;
use bs58;
use crate::models::{SuccessResponse, ErrorResponse};

pub fn routes() -> Router {
    Router::new().route("/keypair", post(generate_keypair))
}

async fn generate_keypair() -> (StatusCode, Json<SuccessResponse<serde_json::Value>>) {
    let keypair = SolanaKeypair::new();
    let resp = SuccessResponse {
        success: true,
        data: serde_json::json!({
            "pubkey": keypair.pubkey().to_string(),
            "secret": bs58::encode(keypair.to_bytes()).into_string(),
        }),
    };
    (StatusCode::OK, Json(resp))
}
