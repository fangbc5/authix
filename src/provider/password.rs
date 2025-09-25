use std::sync::Arc;

use axum::async_trait;
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2, PasswordHash, PasswordVerifier};

use crate::{common::R, errors::{AuthixError, AuthixResult}, provider::{login::{LoginProvider, LoginRequest, LoginResponse}, register::{RegisterProvider, RegisterRequest}}, user::{User, UserProvider}, utils::{jwt, regex::{is_valid_password, is_valid_username}}};

pub struct PasswordLoginProvider;

#[async_trait]
impl LoginProvider for PasswordLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 通过用户名加载用户
        let user = match user_service
            .get_user_by_username(req.identifier.clone())
            .await? {
                Some(u) => u,
                None => return Err(AuthixError::InvalidCredentials("用户名或密码错误".into())),
            };

        // 使用 argon2 校验密码（user.password 应存储为 PHC 字符串）
        let parsed_hash = PasswordHash::new(&user.password)
            .map_err(|e| AuthixError::InvalidCredentials(format!("密码加密失败, {}", e)))?;
        let argon2 = Argon2::default();
        if argon2.verify_password(req.credential.as_bytes(), &parsed_hash).is_err() {
            return Err(AuthixError::InvalidCredentials("用户名或密码错误".into()));
        }

        // 使用用户 id 作为 sub 生成 token
        let resp = jwt::create_token(user.id.to_string(), "0".to_string()).await?;

        // 更新最后登录时间
        let _ = user_service.update_last_login_time(user.id).await?;

        Ok(R::ok_data(resp))
    }
}

pub struct PasswordRegisterProvider;
#[async_trait]
impl RegisterProvider for PasswordRegisterProvider {
    async fn register(&self, req: &RegisterRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<u64>> {
        // 参数校验
        if !is_valid_username(&req.identifier) {
            return Ok(R::error(400, "用户名不合法".into()));
        }
        if !is_valid_password(&req.credential) {
            return Ok(R::error(400, "密码不合法(8-32位，支持字母数字常见符号)".into()));
        }
        // 唯一性校验
        if let Some(_) = user_service.get_user_by_username(req.identifier.clone()).await? {
            return Ok(R::error(409, "username already exists".into()))
        }
        // hash 密码
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.credential.as_bytes(), &SaltString::generate(&mut OsRng))
            .map(|h| h.to_string())
            .map_err(|e| AuthixError::InvalidCredentials(format!("hash password error: {}", e)))?;

        let new_user = User {
            id: 0,
            tenant_id: 0,
            username: Some(req.identifier.clone()),
            phone: None,
            email: None,
            password: password_hash,
            crt_by: None,
        };
        match user_service.create_user(new_user).await {
            Ok(u) => Ok(R::ok_data(u.id)),
            Err(e) => Ok(R::error(500, e)),
        }
    }
}