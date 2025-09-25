use std::sync::Arc;

use axum::async_trait;

use crate::{common::R, errors::{AuthixError, AuthixResult}, login::{LoginProvider, LoginRequest, LoginResponse}, user::UserProvider};

pub struct SmsLoginProvider;
#[async_trait]
impl LoginProvider for SmsLoginProvider {
    async fn login(&self, req: &LoginRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<LoginResponse>> {
        // 验证短信验证码（此处省略，假设上游已校验 req.credential）
        let user = user_service
            .get_user_by_phone(req.identifier.clone())
            .await
            .map_err(|_| AuthixError::InvalidCredentials("手机号未注册".to_owned()))?;
        
        let resp = crate::utils::jwt::create_token(user.id.to_string(), "0".to_string()).await?;

        // 更新用户最后登录时间
        user_service.update_last_login_time(user.id).await?;
        Ok(R::ok_data(resp))
    }
}