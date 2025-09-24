use std::env;

use once_cell::sync::Lazy;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tracing::info;

/// 全局数据库连接池
pub static DB_POOL: Lazy<MySqlPool> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@127.0.0.1:3306/test".to_string());
    info!("initialize database url = {}", database_url);

    MySqlPoolOptions::new()
                .max_connections(10)
                .connect_lazy(&database_url)
                .expect("Failed to create database pool")
});