use axum::async_trait;

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}};

pub struct EmailLoginProvider;
#[async_trait]
impl LoginProvider for EmailLoginProvider {
    async fn login(&self, req: &LoginRequest) -> AuthixResult<R<LoginResponse>> {
        if req.credential == "abcdef" {
            Ok(R::ok())
        } else {
            Err(AuthixError::InvalidCredentials("邮箱验证码错误".into()))
        }
    }
}