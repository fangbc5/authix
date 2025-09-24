use std::{collections::HashMap, sync::Arc};

use axum::{async_trait, http::{HeaderMap, StatusCode}, Extension, Json, response::IntoResponse};
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};

use crate::{common::{UidHeader, R}, errors::{AuthixError, AuthixResult}, provider::{EmailLoginProvider, PasswordLoginProvider, SmsLoginProvider}, user::UserProvider, utils::jwt};

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
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>>;
}

/// 登录服务，负责调度不同 Provider
pub struct LoginService {
    providers: HashMap<LoginType, Box<dyn LoginProvider>>,
}

impl Default for LoginService {
    fn default() -> Self {
        let mut providers: HashMap<LoginType, Box<dyn LoginProvider>> = HashMap::new();
        providers.insert(LoginType::Password, Box::new(PasswordLoginProvider));
        providers.insert(LoginType::Sms, Box::new(SmsLoginProvider));
        providers.insert(LoginType::Email, Box::new(EmailLoginProvider));
        Self { providers }
    }
}

#[async_trait]
impl LoginProvider for LoginService {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        if let Some(provider) = self.providers.get(&LoginType::from(req.login_type.clone())) {
            provider.login(&req, user_service).await
        } else {
            Err(AuthixError::UnknowLoginType(format!("未知的登录方式: {}", req.login_type.clone())))
        }
    }
}

pub async fn login_handler(
    Extension(login): Extension<Arc<dyn LoginProvider>>,
    Extension(user): Extension<Arc<dyn UserProvider>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match login.login(&payload, user).await {
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
        Ok(claims) => match jwt::get_token(&claims.sub, &claims.tenant_id, 1, "access").await {
            Ok((access_token,exp, _)) => (StatusCode::OK, Json(R::ok_data(LoginResponse { access_token, refresh_token: token.to_string(), exp, iat: claims.iat}))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<LoginResponse>::error(500, e.to_string()))),
        },
        Err(_) => unauthorized(),
    }
}

pub async fn logout_handler(TypedHeader(uid): TypedHeader<UidHeader>) -> impl IntoResponse {
    let id: u64 = match uid.0.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(R::<String>::error(400, "invalid uid".into()))),
    };
    match crate::cache::delete_user_access_token(id).await {
        Ok(_) => (StatusCode::OK, Json(R::<String>::ok())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<String>::error(500, e))),
    }
}