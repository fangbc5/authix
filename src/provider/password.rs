use axum::async_trait;

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, utils::jwt};

pub struct PasswordLoginProvider;

#[async_trait]
impl LoginProvider for PasswordLoginProvider {
    async fn login(&self, req: &LoginRequest) -> AuthixResult<R<LoginResponse>> {
        // 假设查数据库
        if req.identifier == "admin" && req.credential == "admin" {
            let resp = jwt::create_token(req.identifier.clone(), req.credential.clone()).await?;
            Ok(R::ok_data(resp))
        } else {
            Err(AuthixError::InvalidCredentials("用户名或密码错误".into()))
        }
    }
}