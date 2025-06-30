use axum::{routing::post, Router, Json, response::IntoResponse};
use axum::http::StatusCode;
use solana_sdk::{pubkey::Pubkey, system_instruction};
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};
use crate::models::*;
use crate::utils::error_response;

pub fn routes() -> Router {
    Router::new().route("/send/sol", post(send_sol))
}

async fn send_sol(Json(payload): Json<SendSolRequest>) -> impl IntoResponse {
    let from = match Pubkey::from_str(&payload.from) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid from address"),
    };
    let to = match Pubkey::from_str(&payload.to) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid to address"),
    };

    let instr = system_instruction::transfer(&from, &to, payload.lamports);

    let accounts = instr.accounts.iter().map(|a| {
        serde_json::json!({
            "pubkey": a.pubkey.to_string(),
            "is_signer": a.is_signer,
            "is_writable": a.is_writable
        })
    }).collect::<Vec<_>>();

    let resp = SuccessResponse {
        success: true,
        data: serde_json::json!({
            "program_id": instr.program_id.to_string(),
            "accounts": accounts,
            "instruction_data": general_purpose::STANDARD.encode(instr.data),
        }),
    };
    (StatusCode::OK, Json(resp))
}
