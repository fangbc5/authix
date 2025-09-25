use std::{collections::HashMap, sync::Arc};

use axum::{async_trait, http::{HeaderMap, StatusCode}, Extension, Json, response::IntoResponse};
use axum_extra::TypedHeader;
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::{cache::USER_CAN_REGISTER_FLAG_KEY, common::{UidHeader, R}, enums::AuthEnum, errors::{AuthixError, AuthixResult}, provider::{EmailLoginProvider, PasswordLoginProvider, SmsLoginProvider}, user::{User, UserProvider}, utils::{jwt, redis::REDIS_POOL}};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}, password_hash::rand_core::OsRng};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub login_type: String,    // "password" | "sms" | "email"
    pub identifier: String,    // 用户名/手机号/邮箱
    pub credential: String,    // 密码/验证码
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub register_type: String,  // "password" | "sms" | "email"
    pub identifier: String,     // 用户名/手机号/邮箱
    pub credential: String,     // 密码
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyCodeRequest {
    pub identifier: String,     // 用户名/手机号/邮箱
    pub credential: String,     // 验证码
    pub verify_type: AuthEnum,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendCodeRequest {
    pub identifier: String,     // 用户名/手机号/邮箱
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

pub async fn register_handler(
    Extension(_login): Extension<Arc<dyn LoginProvider>>, // _login is unused here
    Extension(user): Extension<Arc<dyn UserProvider>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 使用 argon2 加密密码
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.credential.as_bytes(), &SaltString::generate(&mut OsRng)) {
        Ok(h) => h.to_string(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, format!("hash password error: {}", e)))),
    };

    match payload.register_type.as_str() {
        "password" => {
            // 检查用户名是否已存在
            match user.get_user_by_username(payload.identifier.clone()).await {
                Ok(Some(_)) => {
                    return (StatusCode::CONFLICT, Json(R::<u64>::error(409, "username already exists".into())));
                }
                Ok(None) => { /* continue */ }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e.to_string())));
                }
            }
            
            let new_user = User {
                id: 0,
                tenant_id: 0, // 默认租户ID为0
                username: Some(payload.identifier.clone()),
                phone: None,
                email: None,
                password: password_hash,
                crt_by: Some(payload.identifier.clone()),
            };
            
            match user.create_user(new_user).await {
                Ok(u) => (StatusCode::OK, Json(R::ok_data(u.id))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e))),
            }
        }
        "sms" => {
            // 检查手机号是否已存在
            match user.get_user_by_phone(payload.identifier.clone()).await {
                Ok(Some(_)) => {
                    return (StatusCode::CONFLICT, Json(R::<u64>::error(409, "phone already exists".into())));
                }
                Ok(None) => { /* continue */ }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e.to_string())));
                }
            }
            
            // 检查是否验证过验证码（5分钟内）
            let mut conn = match REDIS_POOL.get().await {
                Ok(conn) => conn,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, format!("redis get conn error: {}", e)))),
            };
            
            let register_flag_key = format!("{}:{}", USER_CAN_REGISTER_FLAG_KEY, payload.identifier);
            let can_register: Option<String> = match conn.get(&register_flag_key).await {
                Ok(flag) => flag,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, format!("redis get error: {}", e)))),
            };
            
            if can_register.is_none() {
                return (StatusCode::BAD_REQUEST, Json(R::<u64>::error(400, "验证码已失效,请重新获取验证码".into())));
            }
            
            let new_user = User {
                id: 0,
                tenant_id: 0,
                username: None,
                phone: Some(payload.identifier.clone()),
                email: None,
                password: password_hash,
                crt_by: Some(payload.identifier.clone()),
            };
            
            match user.create_user(new_user).await {
                Ok(u) => {
                    // 注册成功后清除注册验证标识
                    let _: () = conn.del(&register_flag_key).await.unwrap_or_default();
                    (StatusCode::OK, Json(R::ok_data(u.id)))
                },
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e))),
            }
        }
        "email" => {
            // 检查邮箱是否已存在
            match user.get_user_by_email(payload.identifier.clone()).await {
                Ok(Some(_)) => {
                    return (StatusCode::CONFLICT, Json(R::<u64>::error(409, "email already exists".into())));
                }
                Ok(None) => { /* continue */ }
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e.to_string())));
                }
            }
            
            // 检查是否验证过验证码（5分钟内）
            let mut conn = match REDIS_POOL.get().await {
                Ok(conn) => conn,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, format!("redis get conn error: {}", e)))),
            };
            
            let register_flag_key = format!("{}:{}", USER_CAN_REGISTER_FLAG_KEY, payload.identifier);
            let can_register: Option<String> = match conn.get(&register_flag_key).await {
                Ok(flag) => flag,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, format!("redis get error: {}", e)))),
            };
            
            if can_register.is_none() {
                return (StatusCode::BAD_REQUEST, Json(R::<u64>::error(400, "验证码已失效,请重新获取验证码".into())));
            }
            
            let new_user = User {
                id: 0,
                tenant_id: 0,
                username: None,
                phone: None,
                email: Some(payload.identifier.clone()),
                password: password_hash,
                crt_by: Some(payload.identifier.clone()),
            };
            
            match user.create_user(new_user).await {
                Ok(u) => {
                    // 注册成功后清除注册验证标识
                    let _: () = conn.del(&register_flag_key).await.unwrap_or_default();
                    (StatusCode::OK, Json(R::ok_data(u.id)))
                },
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e))),
            }
        }
        _ => (StatusCode::BAD_REQUEST, Json(R::<u64>::error(400, "unsupported register type".into()))),
    }
}

pub async fn send_code(Json(payload): Json<SendCodeRequest>) -> impl IntoResponse {
    match crate::cache::save_verify_code(&payload.identifier).await {
        Ok(code) => (StatusCode::OK, Json(R::<String>::ok_data(code))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<String>::error(500, e))),
    }
}

pub async fn verify_code(Json(payload): Json<VerifyCodeRequest>) -> impl IntoResponse {
    match crate::cache::verify_code(&payload.identifier, &payload.credential, payload.verify_type).await {
        Ok(true) => (StatusCode::OK, Json(R::<String>::ok())),
        Ok(false) => (StatusCode::UNAUTHORIZED, Json(R::<String>::error(401, "invalid verification code".into()))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<String>::error(500, e))),
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