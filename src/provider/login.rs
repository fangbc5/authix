use std::{collections::HashMap, sync::Arc};

use axum::async_trait;
use serde::{Deserialize, Serialize};

use crate::{common::R, enums::AuthType, errors::{AuthixError, AuthixResult}, provider::{EmailLoginProvider, PasswordLoginProvider, SmsLoginProvider}, user::UserProvider};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub login_type: String,    // "password" | "sms" | "email"
    pub identifier: String,    // 用户名/手机号/邮箱
    pub credential: String,    // 密码/验证码
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub exp: usize,
    pub iat: usize,
}

#[async_trait]
pub trait LoginProvider: Send + Sync {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>>;
}

/// 登录服务，负责调度不同 Provider
pub struct LoginService {
    providers: HashMap<AuthType, Box<dyn LoginProvider>>,
}

impl Default for LoginService {
    fn default() -> Self {
        let mut providers: HashMap<AuthType, Box<dyn LoginProvider>> = HashMap::new();
        providers.insert(AuthType::Password, Box::new(PasswordLoginProvider));
        providers.insert(AuthType::Sms, Box::new(SmsLoginProvider));
        providers.insert(AuthType::Email, Box::new(EmailLoginProvider));
        Self { providers }
    }
}

#[async_trait]
impl LoginProvider for LoginService {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        if let Some(provider) = self.providers.get(&AuthType::from(req.login_type.clone())) {
            provider.login(&req, user_service).await
        } else {
            Err(AuthixError::UnknowLoginType(format!("未知的登录方式: {}", req.login_type.clone())))
        }
    }
}