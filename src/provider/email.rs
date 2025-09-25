use std::sync::Arc;

use axum::async_trait;

use crate::{common::R, enums::AuthEnum, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, user::UserProvider};

pub struct EmailLoginProvider;
#[async_trait]
impl LoginProvider for EmailLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 校验邮箱验证码
        match crate::cache::verify_code(&req.identifier, &req.credential, AuthEnum::Login).await {
            Ok(true) => {}
            Ok(false) => return Err(AuthixError::InvalidCredentials("验证码错误".into())),
            Err(e) => return Err(AuthixError::InvalidCredentials(format!("验证码校验失败, {}", e))),
        }

        // 通过邮箱加载用户
        let user = match user_service
            .get_user_by_email(req.identifier.clone())
            .await? {
                Some(u) => u,
                None => return Err(AuthixError::InvalidCredentials("邮箱未注册".into())),
            };

        let resp = crate::utils::jwt::create_token(user.id.to_string(), "0".to_string()).await?;

        // 更新最后登录时间
        let _ = user_service.update_last_login_time(user.id).await?;

        Ok(R::ok_data(resp))
    }
}