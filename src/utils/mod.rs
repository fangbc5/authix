use serde::{Deserialize, Serialize};

pub mod jwt;
pub mod uuid;
pub mod database;
pub mod redis;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,       // 用户 ID
    pub exp: usize,        // 过期时间（秒）或毫秒，取决于生成策略
    pub iat: usize,        // 签发时间
    pub tenant_id: String, // 多租户 ID
    pub token_type: String, // "access" | "refresh"
}