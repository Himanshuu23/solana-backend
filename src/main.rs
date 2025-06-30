use axum::{
    extract::Json,
    routing::post,
    Router,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::Signer,
    pubkey::Pubkey,
    system_instruction,
    signer::keypair::Keypair as SolanaKeypair,
};
use solana_sdk::signature::Signature;
use spl_token::{
    instruction::{initialize_mint, mint_to, transfer},
    id as spl_token_id,
};
use std::net::SocketAddr;
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};
use bs58;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("✅ Server running at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ErrorResponse {
            success: false,
            error: message.to_string(),
        }),
    )
        .into_response()
}

async fn generate_keypair() -> impl IntoResponse {
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

#[derive(Deserialize)]
struct CreateTokenRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Response {
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
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Response {
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
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> Response {
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
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Response {
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
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Response {
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
    (StatusCode::OK, Json(resp)).into_response()
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Response {
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

    // In a real implementation, you would need to derive the source account from owner+mint
    // This is a simplified version for demonstration
    let source = owner; // This should be the token account in reality

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
    (StatusCode::OK, Json(resp)).into_response()
}
