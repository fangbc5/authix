use std::{env, sync::Arc};

use axum::{routing::{get, post}, Router};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::login::{login_handler, logout_handler, refresh_token, LoginProvider, LoginService};
use crate::user::{user_profile, UserService, UserProvider};
use crate::utils::uuid::get_token;

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
    // 初始化日志
    init_logger().await;
    // 初始化app配置
    let app = init_app().await;
    let server_addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:30000".to_owned());
    info!("🚀 Auth service running at http://{}", server_addr);
    let listener = tokio::net::TcpListener::bind(server_addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

pub async fn init_logger() {
    // 初始化日志：若无 RUST_LOG 则默认 info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
}

pub async fn init_app() -> Router {
    let login_service = Arc::new(LoginService::default());
    let user_service = Arc::new(UserService::default());
    let auth_router = Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler));
    let token_router = Router::new()
        .route("/token/refresh", get(refresh_token))
        .route("/token/get", get(get_token));
    let user_router =  Router::new()
        .route("/profile", get(user_profile));

    Router::new()
    .nest("/auth", auth_router)
    .nest("/token", token_router)
    .nest("/user", user_router)
    .layer(axum::Extension(login_service as Arc<dyn LoginProvider>))
    .layer(axum::Extension(user_service as Arc<dyn UserProvider>))
}