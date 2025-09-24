use std::sync::Arc;

use axum::async_trait;

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, user::UserProvider};

pub struct EmailLoginProvider;
#[async_trait]
impl LoginProvider for EmailLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 验证邮箱验证码（此处省略，假设上游已校验 req.credential）
        let user = user_service
            .get_user_by_email(req.identifier.clone())
            .await
            .map_err(|_| AuthixError::InvalidCredentials("邮箱未注册".into()))?;
        let resp = crate::utils::jwt::create_token(user.id.to_string(), "default".to_string()).await?;
        Ok(R::ok_data(resp))
    }
}