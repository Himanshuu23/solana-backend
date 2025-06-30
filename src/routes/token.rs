use axum::{routing::post, Router, Json, response::IntoResponse};
use axum::http::StatusCode;
use solana_sdk::pubkey::Pubkey;
use spl_token::{instruction::{initialize_mint, mint_to, transfer}, id as spl_token_id};
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};
use crate::models::*;
use crate::utils::error_response;

pub fn routes() -> Router {
    Router::new()
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/send/token", post(send_token))
}

async fn create_token(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid mint pubkey"),
    };
    let authority = match Pubkey::from_str(&payload.mint_authority) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid mint_authority pubkey"),
    };
    let instr = initialize_mint(
        &spl_token_id(),
        &mint,
        &authority,
        None,
        payload.decimals,
    ).unwrap();

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

async fn mint_token(Json(payload): Json<MintTokenRequest>) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid mint pubkey"),
    };
    let destination = match Pubkey::from_str(&payload.destination) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid destination pubkey"),
    };
    let authority = match Pubkey::from_str(&payload.authority) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid authority pubkey"),
    };

    let instr = mint_to(
        &spl_token_id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    ).unwrap();

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

async fn send_token(Json(payload): Json<SendTokenRequest>) -> impl IntoResponse {
    let _mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid mint address"),
    };
    let destination = match Pubkey::from_str(&payload.destination) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid destination address"),
    };
    let owner = match Pubkey::from_str(&payload.owner) {
        Ok(pk) => pk,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid owner address"),
    };

    let source = owner;
    let instr = transfer(
        &spl_token_id(),
        &source,
        &destination,
        &owner,
        &[],
        payload.amount,
    ).unwrap();

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
