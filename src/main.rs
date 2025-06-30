use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    serve,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use dotenvy::dotenv;
use std::env;

async fn get_balance(rpc_url: String, pubkey_str: String) -> impl IntoResponse {
    let client = RpcClient::new(rpc_url);
    let pubkey = Pubkey::from_str(&pubkey_str).unwrap();

    match client.get_balance(&pubkey) {
        Ok(balance) => format!("Balance: {} lamports", balance),
        Err(err) => format!("Error: {}", err),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL").unwrap();
    let pubkey_str = env::var("PUBKEY").unwrap();

    let app = Router::new().route("/", get(move || get_balance(rpc_url.clone(), pubkey_str.clone())));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("✅ Server running at http://{}", addr);

    serve(listener, app).await;
    Ok(())
}
