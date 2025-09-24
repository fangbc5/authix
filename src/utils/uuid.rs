use axum::response::IntoResponse;
use axum_extra::extract::TypedHeader;
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::{common::{TenantIdHeader, UidHeader}, utils::redis::REDIS_POOL};

pub async fn get_token(TypedHeader(tenant_id): TypedHeader<TenantIdHeader>, TypedHeader(uid): TypedHeader<UidHeader>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string().replace("-", "");
    let mut conn = REDIS_POOL.get().await.unwrap();
    let key = format!("{}:{}:aaa",tenant_id.0,uid.0);
    let _: () = conn.set_ex(&key, id.as_str(), 60 * 5).await.unwrap();
    id
}