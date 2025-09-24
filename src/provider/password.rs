use std::sync::Arc;

use axum::async_trait;
use argon2::{Argon2, password_hash::{PasswordHash, PasswordVerifier}};

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, user::UserProvider, utils::jwt};

pub struct PasswordLoginProvider;

#[async_trait]
impl LoginProvider for PasswordLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 通过用户名加载用户
        let user = user_service
            .get_user_by_username(req.identifier.clone())
            .await
            .map_err(|e| AuthixError::InvalidCredentials(format!("查询用户错误, {}", e)))?;

        // 使用 argon2 校验密码（user.password 应存储为 PHC 字符串）
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| AuthixError::InvalidCredentials(format!("密码加密失败, {}", e)))?;
        let argon2 = Argon2::default();
        if argon2.verify_password(req.credential.as_bytes(), &parsed_hash).is_err() {
            return Err(AuthixError::InvalidCredentials("用户名或密码错误".into()));
        }

        // 使用用户 id 作为 sub 生成 token
        let resp = jwt::create_token(user.id.to_string(), "default".to_string()).await?;
        Ok(R::ok_data(resp))
    }
}