use std::{env, time::{Duration, SystemTime, UNIX_EPOCH}};
use jsonwebtoken::{encode, decode, Algorithm, EncodingKey, DecodingKey, Header, Validation};
use crate::{errors::{AuthixError, AuthixResult}, login::LoginResponse, utils::Claims};

pub async fn create_token(sub: String, tenant_id: String) -> AuthixResult<LoginResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis() as usize;
    let exp = now + 1000 * 60 * 5; // 5 min
    let refresh_exp = now + 1000 * 60 * 60 * 24 * 7; // 7 day
    let claims = Claims { sub: sub.clone(), tenant_id: tenant_id.clone(), exp, token_type: "access".to_string() };
    let refresh_claims = Claims { sub, tenant_id, exp: refresh_exp, token_type: "refresh".to_string() };
    let jwt_secret = env::var("JWT_DECODING_KEY")?;
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    let refresh_token = encode(
        &Header::new(Algorithm::HS256),
        &refresh_claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(LoginResponse { token, refresh_token, exp, iat: now })
}

pub async fn verify_access_token(token: &str) -> AuthixResult<Claims> {
    let jwt_secret = env::var("JWT_DECODING_KEY")?;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    let claims = data.claims;
    if claims.token_type != "access" {
        return Err(AuthixError::InvalidCredentials("token type must be access".into()));
    }
    Ok(claims)
}

pub async fn verify_refresh_token(token: &str) -> AuthixResult<Claims> {
    let jwt_secret = env::var("JWT_DECODING_KEY")?;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    let claims = data.claims;
    if claims.token_type != "refresh" {
        return Err(AuthixError::InvalidCredentials("token type must be refresh".into()));
    }
    Ok(claims)
}
