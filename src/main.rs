use std::{env, sync::Arc};

use axum::{routing::{get, post}, Router};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{login::{login_handler, refresh_token, LoginService}, user::{user_profile, UserService}, utils::uuid::get_token};

mod common;
mod login;
mod provider;
mod user;
mod utils;
mod errors;

#[tokio::main]
async fn main() {
    // 初始化配置
    dotenv().ok();
    // 初始化日志：若无 RUST_LOG 则默认 info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
    let login_service = Arc::new(LoginService::new());
    let user_service = Arc::new(UserService::default());
    let api_router = Router::new()
        .route("/login", post(login_handler))
        .route("/profile", get(user_profile))
        .route("/token/refresh", get(refresh_token))
        .route("/token/get", get(get_token));
    let app = Router::new()
    .nest("/auth", api_router)
    .layer(axum::Extension(login_service))
    .layer(axum::Extension(user_service));
    let server_addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:30000".to_owned());
    info!("🚀 Auth service running at http://{}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
