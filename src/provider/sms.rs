use axum::async_trait;

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}};

pub struct SmsLoginProvider;
#[async_trait]
impl LoginProvider for SmsLoginProvider {
    async fn login(&self, req: &LoginRequest) -> AuthixResult<R<LoginResponse>> {
        // 这里验证手机号验证码逻辑，例如 Redis 中存储的验证码
        if req.credential == "123456" {
            Ok(R::ok())
        } else {
            Err(AuthixError::InvalidCredentials("手机号或验证码错误".to_owned()))
        }
    }
}