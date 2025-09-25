use std::sync::Arc;

use axum::async_trait;

use crate::{common::R, enums::AuthEnum, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, user::UserProvider};

pub struct SmsLoginProvider;
#[async_trait]
impl LoginProvider for SmsLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 校验短信验证码
        match crate::cache::verify_code(&req.identifier, &req.credential, AuthEnum::Login).await {
            Ok(true) => {}
            Ok(false) => return Err(AuthixError::InvalidCredentials("验证码错误".to_owned())),
            Err(e) => return Err(AuthixError::InvalidCredentials(format!("验证码校验失败, {}", e))),
        }

        // 通过手机号加载用户
        let user = match user_service
            .get_user_by_phone(req.identifier.clone())
            .await? {
                Some(u) => u,
                None => return Err(AuthixError::InvalidCredentials("手机号未注册".to_owned())),
            };

        let resp = crate::utils::jwt::create_token(user.id.to_string(), "0".to_string()).await?;

        // 更新最后登录时间
        let _ = user_service.update_last_login_time(user.id).await?;

        Ok(R::ok_data(resp))
    }
}