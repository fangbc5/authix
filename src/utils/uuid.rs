use axum::response::IntoResponse;
use axum_extra::extract::TypedHeader;
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::{common::{TenantIdHeader, UidHeader}, utils::redis::REDIS_POOL};

const ONE_TIME_TOKEN_KEY: &str = "one_time_token";

pub async fn get_token(
    TypedHeader(tenant_id): TypedHeader<TenantIdHeader>,
    TypedHeader(uid): TypedHeader<UidHeader>,
) -> impl IntoResponse {
    let token = Uuid::new_v4().to_string().replace("-", "");
    let key = format!("{}:{}:{}:{}", ONE_TIME_TOKEN_KEY, tenant_id.0, uid.0, token);

    let mut conn = REDIS_POOL.get().await.unwrap();
    let _: () = conn.set_ex(&key, "1", 60 * 5).await.unwrap();

    token
}