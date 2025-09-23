use axum::response::IntoResponse;
use uuid::Uuid;

pub async fn get_token() -> impl IntoResponse {
    Uuid::new_v4().to_string().replace("-", "")
}