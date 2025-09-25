use std::sync::Arc;

use axum::{http::{HeaderMap, StatusCode}, Extension, Json, response::IntoResponse};
use axum_extra::TypedHeader;
use serde::Deserialize;

use crate::{common::{UidHeader, R}, enums::{AuthEnum, AuthType}, provider::{login::{LoginProvider, LoginRequest, LoginResponse}, register::{RegisterProvider, RegisterRequest}}, user::UserProvider, utils::jwt};
use crate::utils::regex::{is_valid_email, is_valid_phone};

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyCodeRequest {
    pub identifier: String,     // 手机号/邮箱
    pub credential: String,     // 验证码
    pub verify_type: AuthEnum,  // 验证类型sms、email
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendCodeRequest {
    pub identifier: String,     // 用户名/手机号/邮箱
    pub verify_type: AuthType,  // 验证类型sms、email
}

pub async fn register_handler(
    Extension(user_service): Extension<Arc<dyn UserProvider>>,
    Extension(register_service): Extension<Arc<dyn RegisterProvider>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match register_service.register(&payload, user_service).await {
        Ok(resp) => {
            let status = if resp.code == 0 {
                StatusCode::OK
            } else {
                StatusCode::from_u16(resp.code as u16).unwrap_or(StatusCode::BAD_REQUEST)
            };
            (status, Json(resp))
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(R::<u64>::error(400, e.to_string()))),
    }
}

pub async fn send_code(Json(payload): Json<SendCodeRequest>) -> impl IntoResponse {
    // 根据 verify_type 校验 identifier
    match payload.verify_type {
        AuthType::Sms => {
            if !is_valid_phone(&payload.identifier) {
                return (StatusCode::BAD_REQUEST, Json(R::<String>::error(400, "手机号格式不正确".into())));
            }
        }
        AuthType::Email => {
            if !is_valid_email(&payload.identifier) {
                return (StatusCode::BAD_REQUEST, Json(R::<String>::error(400, "邮箱格式不正确".into())));
            }
        }
        _ => {
            return (StatusCode::BAD_REQUEST, Json(R::<String>::error(400, "不支持的验证类型".into())));
        }
    }

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