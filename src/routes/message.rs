use axum::{routing::post, Router, Json, response::IntoResponse};
use axum::http::StatusCode;
use solana_sdk::{signature::Signature, signer::keypair::Keypair as SolanaKeypair, pubkey::Pubkey};
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};
use bs58;
use crate::models::*;
use crate::utils::error_response;

pub fn routes() -> Router {
    Router::new()
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
}

async fn sign_message(Json(payload): Json<SignMessageRequest>) -> impl IntoResponse {
    let secret_bytes = match bs58::decode(&payload.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid secret key"),
    };

    let keypair = match SolanaKeypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid keypair"),
    };

    let signature = keypair.sign_message(payload.message.as_bytes());
    let signature_base64 = general_purpose::STANDARD.encode(signature.as_ref());

    let resp = SuccessResponse {
        success: true,
        data: serde_json::json!({
            "signature": signature_base64,
            "public_key": keypair.pubkey().to_string(),
            "message": payload.message,
        }),
    };
    (StatusCode::OK, Json(resp))
}

async fn verify_message(Json(payload): Json<VerifyMessageRequest>) -> impl IntoResponse {
    let pubkey = match Pubkey::from_str(&payload.pubkey) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid public key"),
    };

    let signature_bytes = match general_purpose::STANDARD.decode(&payload.signature) {
        Ok(bytes) => bytes,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid signature format"),
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid signature length"),
    };

    let valid = signature.verify(pubkey.as_ref(), payload.message.as_bytes());

    let resp = SuccessResponse {
        success: true,
        data: serde_json::json!({
            "valid": valid,
            "message": payload.message,
            "pubkey": payload.pubkey,
        }),
    };
    (StatusCode::OK, Json(resp))
}
