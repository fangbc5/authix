use std::sync::Arc;

use argon2::{password_hash::{rand_core::OsRng, SaltString, PasswordHasher}, Argon2};
use axum::async_trait;
use deadpool_redis::redis::AsyncCommands;

use crate::{cache::USER_CAN_REGISTER_FLAG_KEY, common::R, enums::AuthEnum, errors::{AuthixError, AuthixResult}, provider::{login::{LoginProvider, LoginRequest, LoginResponse}, register::{RegisterProvider, RegisterRequest}}, user::{User, UserProvider}, utils::{redis::REDIS_POOL, regex::{is_valid_password, is_valid_phone}}};

pub struct SmsLoginProvider;
#[async_trait]
impl LoginProvider for SmsLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 校验短信验证码
        match crate::cache::verify_code(&req.identifier, &req.credential, AuthEnum::Login).await {
            Ok(true) => {}
            Ok(false) => return Err(AuthixError::InvalidCredentials("验证码错误".to_owned())),
            Err(e) => return Err(AuthixError::InvalidCredentials(format!("验证码校验失败, {}", e))),
        }

        // 通过手机号加载用户
        let user = match user_service
            .get_user_by_phone(req.identifier.clone())
            .await? {
                Some(u) => u,
                None => return Err(AuthixError::InvalidCredentials("手机号未注册".to_owned())),
            };

        let resp = crate::utils::jwt::create_token(user.id.to_string(), "0".to_string()).await?;

        // 更新最后登录时间
        let _ = user_service.update_last_login_time(user.id).await?;

        Ok(R::ok_data(resp))
    }
}

pub struct SmsRegisterProvider;
#[async_trait]
impl RegisterProvider for SmsRegisterProvider {
    async fn register(&self, req: &RegisterRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<u64>> {
        // 参数校验
        if !is_valid_phone(&req.identifier) {
            return Ok(R::error(400, "手机号格式不正确".into()));
        }
        if !is_valid_password(&req.credential) {
            return Ok(R::error(400, "密码不合法(8-32位，支持字母数字常见符号)".into()));
        }
        // 唯一性
        if let Some(_) = user_service.get_user_by_phone(req.identifier.clone()).await? {
            return Ok(R::error(409, "phone already exists".into()))
        }
        // 验证 5 分钟标识
        let mut conn = REDIS_POOL.get().await.map_err(|e| AuthixError::InvalidCredentials(format!("redis get conn error: {}", e)))?;
        let register_flag_key = format!("{}:{}", USER_CAN_REGISTER_FLAG_KEY, req.identifier);
        let can_register: Option<String> = conn.get(&register_flag_key).await.map_err(|e| AuthixError::InvalidCredentials(format!("redis get error: {}", e)))?;
        if can_register.is_none() {
            return Ok(R::error(400, "验证码已失效,请重新获取验证码".into()));
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
            username: None,
            phone: Some(req.identifier.clone()),
            email: None,
            password: password_hash,
            crt_by: None,
        };
        match user_service.create_user(new_user).await {
            Ok(u) => {
                let _: () = conn.del(&register_flag_key).await.unwrap_or_default();
                Ok(R::ok_data(u.id))
            }
            Err(e) => Ok(R::error(500, e)),
        }
    }
}