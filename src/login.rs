use std::{collections::HashMap, sync::Arc};

use axum::{async_trait, http::{HeaderMap, StatusCode}, Extension, Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{common::R, errors::{AuthixError, AuthixResult}, provider::{EmailLoginProvider, PasswordLoginProvider, SmsLoginProvider}, utils::jwt};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub login_type: String,    // "password" | "sms" | "email"
    pub identifier: String,    // 用户名/手机号/邮箱
    pub credential: String,    // 密码/验证码
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoginType {
    Password,
    Sms,
    Email,
}

impl From<String> for LoginType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "password" => LoginType::Password,
            "sms" => LoginType::Sms,
            "email" => LoginType::Email,
            _ => LoginType::Password,
        }
    }
}

#[async_trait]
pub trait LoginProvider: Send + Sync {
    async fn login(&self, req: &LoginRequest) -> AuthixResult<R<LoginResponse>>;
}

/// 登录服务，负责调度不同 Provider
pub struct LoginService {
    providers: HashMap<LoginType, Box<dyn LoginProvider>>,
}

impl LoginService {
    pub fn new() -> Self {
        let mut providers: HashMap<LoginType, Box<dyn LoginProvider>> = HashMap::new();
        providers.insert(LoginType::Password, Box::new(PasswordLoginProvider));
        providers.insert(LoginType::Sms, Box::new(SmsLoginProvider));
        providers.insert(LoginType::Email, Box::new(EmailLoginProvider));
        Self { providers }
    }

    pub async fn login(&self, req: LoginRequest) -> AuthixResult<R<LoginResponse>> {
        if let Some(provider) = self.providers.get(&LoginType::from(req.login_type.clone())) {
            provider.login(&req).await
        } else {
            Err(AuthixError::UnknowLoginType(format!("未知的登录方式: {}", req.login_type.clone())))
        }
    }
}

pub async fn login_handler(
    Extension(service): Extension<Arc<LoginService>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match service.login(payload).await {
        Ok(result) => (StatusCode::OK, Json(result)),
        Err(e) => (StatusCode::UNAUTHORIZED, Json(R::<LoginResponse>::error(401, e.to_string()))),
    }
}

pub async fn refresh_token(headers: HeaderMap) -> impl IntoResponse {
    let unauthorized = || (StatusCode::UNAUTHORIZED, Json(R::<LoginResponse>::error(401, "invalid refresh token".to_string())));

    let auth_header = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(v) => v,
        None => return unauthorized(),
    };
    let token = match auth_header.strip_prefix("Bearer ").or_else(|| auth_header.strip_prefix("bearer ")) {
        Some(t) if !t.is_empty() => t,
        _ => return unauthorized(),
    };

    match jwt::verify_refresh_token(token).await {
        Ok(claims) => match jwt::create_token(claims.sub, claims.tenant_id).await {
            Ok(resp) => (StatusCode::OK, Json(R::ok_data(resp))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<LoginResponse>::error(500, e.to_string()))),
        },
        Err(_) => unauthorized(),
    }
}