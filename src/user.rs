use std::sync::Arc;

use axum::{async_trait, response::IntoResponse};
use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::cache::get_online_user_count;
use crate::common::PageQuery;
use crate::common::PageResult;
use crate::errors::AuthixResult;
use crate::utils::database::DB_POOL;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::TypedHeader;
use crate::common::UidHeader;
use crate::common::R;
use axum::Extension;
use axum::extract::Query;

pub const USER_TABLE_NAME: &str = "i18n_users";

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: u64,
    pub tenant_id: u64,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: String,
    #[serde(skip)]
    #[sqlx(skip)]
    pub crt_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct ProfileInfo {
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<u8>,
    pub birthday: Option<NaiveDate>,
    pub last_login: Option<DateTime<Local>>,
}

#[async_trait]
pub trait UserProvider: Send + Sync {
    async fn get_user_profile(&self, id: u64) -> Result<ProfileInfo, String>;
    async fn get_user_profiles(&self, ids: Vec<u64>) -> Result<Vec<ProfileInfo>, String>;
    async fn create_user(&self, user: User) -> Result<User, String>;
    async fn delete_user(&self, id: u64) -> Result<(), String>;
    async fn get_user_by_username(&self, username: String) -> AuthixResult<Option<User>>;
    async fn get_user_by_phone(&self, phone: String) -> AuthixResult<Option<User>>;
    async fn get_user_by_email(&self, email: String) -> AuthixResult<Option<User>>;
    async fn update_last_login_time(&self, id: u64) -> AuthixResult<User>;
}

#[derive(Default)]
pub struct UserService;

#[async_trait]
impl UserProvider for UserService {
    async fn get_user_profile(&self, id: u64) -> Result<ProfileInfo, String> {
        let pool = &*DB_POOL;
        let user = sqlx::query_as::<_, ProfileInfo>(&format!("SELECT username, nickname, avatar, gender, birthday, last_login FROM {} WHERE id = ?", USER_TABLE_NAME))
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(user)
    }

    async fn get_user_profiles(&self, ids: Vec<u64>) -> Result<Vec<ProfileInfo>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let pool = &*DB_POOL;
        // 动态占位符
        let placeholders = std::iter::repeat("?").take(ids.len()).collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT username, nickname, avatar, gender, birthday, last_login FROM {} WHERE id IN ({})",
            USER_TABLE_NAME, placeholders
        );
        let mut q = sqlx::query_as::<_, ProfileInfo>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let list = q
            .fetch_all(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(list)
    }

    async fn create_user(&self, user: User) -> Result<User, String> {
        let pool = &*DB_POOL;
        let result = sqlx::query(&format!("INSERT INTO {} (tenant_id, username, phone, email, password, crt_by) VALUES (?, ?, ?, ?, ?, ?)", USER_TABLE_NAME))
            .bind(&user.tenant_id)
            .bind(&user.username)
            .bind(&user.phone)
            .bind(&user.email)
            .bind(&user.password)
            .bind(&user.crt_by)
            .execute(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        
        let new_user = User {
            id: result.last_insert_id(),
            tenant_id: user.tenant_id,
            username: user.username,
            phone: user.phone,
            email: user.email,
            password: user.password,
            crt_by: None,
        };
        Ok(new_user)
    }

    async fn delete_user(&self, id: u64) -> Result<(), String> {
        let pool = &*DB_POOL;
        sqlx::query(&format!("DELETE FROM {} WHERE id = ?", USER_TABLE_NAME))
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        Ok(())
    }

    async fn get_user_by_username(&self, username: String) -> AuthixResult<Option<User>> {
        let pool = &*DB_POOL;
        let user = sqlx::query_as::<_, User>(&format!("SELECT id, tenant_id, username, phone, email, password FROM {} WHERE username = ?", USER_TABLE_NAME))
            .bind(&username)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    async fn get_user_by_phone(&self, phone: String) -> AuthixResult<Option<User>> {
        let pool = &*DB_POOL;
        let user = sqlx::query_as::<_, User>(&format!("SELECT id, tenant_id, username, phone, email, password FROM {} WHERE phone = ?", USER_TABLE_NAME))
            .bind(&phone)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    async fn get_user_by_email(&self, email: String) -> AuthixResult<Option<User>> {
        let pool = &*DB_POOL;
        let user = sqlx::query_as::<_, User>(&format!("SELECT id, tenant_id, username, phone, email, password FROM {} WHERE email = ?", USER_TABLE_NAME))
            .bind(&email)
            .fetch_optional(pool)
            .await?;
        Ok(user)
    }

    async fn update_last_login_time(&self, id: u64) -> AuthixResult<User> {
        let pool = &*DB_POOL;
        
        // 使用 NOW() 函数获取当前服务器时间
        let result = sqlx::query(&format!("UPDATE {} SET last_login = NOW() WHERE id = ?", USER_TABLE_NAME))
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| crate::errors::AuthixError::DatabaseError(format!("Database error: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(crate::errors::AuthixError::UserNotFound(format!("User with id {} not found", id)));
        }
        
        // 查询更新后的用户信息
        let user = sqlx::query_as::<_, User>(&format!("SELECT id, tenant_id, username, phone, email, password FROM {} WHERE id = ?", USER_TABLE_NAME))
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| crate::errors::AuthixError::DatabaseError(format!("Database error: {}", e)))?;
        
        Ok(user)
    }
}

pub async fn user_profile(
    Extension(user_provider): Extension<Arc<dyn UserProvider>>,
    TypedHeader(uid): TypedHeader<UidHeader>,
) -> impl IntoResponse {
    let id: u64 = match uid.0.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(R::<ProfileInfo>::error(400, "invalid uid".into()))),
    };
    match user_provider.get_user_profile(id).await {
        Ok(user) => (StatusCode::OK, Json(R::ok_data(user))),
        Err(e) => (StatusCode::NOT_FOUND, Json(R::<ProfileInfo>::error(404, e))),
    }
}

pub async fn online_count() -> impl IntoResponse {
    match get_online_user_count().await {
        Ok(count) => (StatusCode::OK, Json(R::ok_data(count))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<u64>::error(500, e))),
    }
}

pub async fn online_users(
    Extension(user_provider): Extension<Arc<dyn UserProvider>>,
    Query(q): Query<PageQuery>,
) -> impl IntoResponse {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 200);
    match crate::cache::get_online_user_ids_paginated(page, page_size).await {
        Ok(page_result) => {
            match user_provider.get_user_profiles(page_result.records.clone()).await {
                Ok(profiles) => {
                    let data = PageResult { total: page_result.total, records: profiles };
                    (StatusCode::OK, Json(R::ok_data(data)))
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<PageResult<ProfileInfo>>::error(500, e)))
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<PageResult<ProfileInfo>>::error(500, e))),
    }
}

pub async fn delete_user(Extension(
    user_provider): Extension<Arc<dyn UserProvider>>,
    TypedHeader(uid): TypedHeader<UidHeader>
) -> impl IntoResponse {
    let id: u64 = match uid.0.parse() {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(R::<String>::error(400, "invalid uid".into()))),
    };
    match user_provider.delete_user(id).await {
        Ok(_) => (StatusCode::OK, Json(R::<String>::ok())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(R::<String>::error(500, e))),
    }
}