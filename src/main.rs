use axum::Router;
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;

mod routes;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();

    let app = Router::new()
        .merge(routes::keypair::routes())
        .merge(routes::token::routes())
        .merge(routes::message::routes())
        .merge(routes::sol::routes());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
