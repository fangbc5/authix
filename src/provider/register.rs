use std::{collections::HashMap, sync::Arc};

use axum::async_trait;
use serde::Deserialize;

use crate::{common::R, enums::AuthType, errors::{AuthixError, AuthixResult}, provider::{EmailRegisterProvider, PasswordRegisterProvider, SmsRegisterProvider}, user::UserProvider};

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub register_type: String,  // "password" | "sms" | "email"
    pub identifier: String,     // 用户名/手机号/邮箱
    pub credential: String,     // 密码
}

#[async_trait]
pub trait RegisterProvider: Send + Sync {
    async fn register(&self, req: &RegisterRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<u64>>;
}

pub struct RegisterService {
    providers: HashMap<AuthType, Box<dyn RegisterProvider>>,
}

impl Default for RegisterService {
    fn default() -> Self {
        let mut providers: HashMap<AuthType, Box<dyn RegisterProvider>> = HashMap::new();
        providers.insert(AuthType::Password, Box::new(PasswordRegisterProvider));
        providers.insert(AuthType::Sms, Box::new(SmsRegisterProvider));
        providers.insert(AuthType::Email, Box::new(EmailRegisterProvider));
        Self { providers }
    }
}

#[async_trait]
impl RegisterProvider for RegisterService {
    async fn register(&self, req: &RegisterRequest, user_service: Arc<dyn UserProvider>) -> AuthixResult<R<u64>> {
        if let Some(provider) = self.providers.get(&AuthType::from(req.register_type.clone())) {
            provider.register(&req, user_service).await
        } else {
            Err(AuthixError::UnknowRegisterType(format!("未知的注册方式: {}", req.register_type.clone())))
        }
    }
}