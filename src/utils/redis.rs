use std::env;

use deadpool_redis::{Config, Pool};
use once_cell::sync::Lazy;
use tracing::info;

/// 全局 Redis 连接池
pub static REDIS_POOL: Lazy<Pool> = Lazy::new(|| {
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
    info!("initialize redis url = {}", redis_url);

    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create redis pool")
});
