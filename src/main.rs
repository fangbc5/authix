use std::{env, sync::Arc};

use axum::{routing::{get, post}, Router};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{login::{login_handler, logout_handler, refresh_token, register_handler, send_code, verify_code, LoginProvider, LoginService}, user::delete_user};
use crate::user::{online_count, user_profile, online_users, UserService, UserProvider};
use crate::utils::uuid::get_token;

mod common;
mod login;
mod provider;
mod user;
mod utils;
mod errors;
mod cache;
mod enums;

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
        .route("/register", post(register_handler))
        .route("/code/verify", post(verify_code))
        .route("/code/send", post(send_code))
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler));
    let token_router = Router::new()
        .route("/refresh", get(refresh_token))
        .route("/get", get(get_token));
    let user_router =  Router::new()
        .route("/online_count", get(online_count))
        .route("/online_users", get(online_users))
        .route("/profile", get(user_profile))
        .route("/delete", get(delete_user));

    Router::new()
    .nest("/auth", auth_router)
    .nest("/token", token_router)
    .nest("/user", user_router)
    .layer(axum::Extension(login_service as Arc<dyn LoginProvider>))
    .layer(axum::Extension(user_service as Arc<dyn UserProvider>))
}