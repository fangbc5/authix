use std::{env, time::{Duration, SystemTime, UNIX_EPOCH}};
use jsonwebtoken::{encode, decode, Algorithm, EncodingKey, DecodingKey, Header, Validation};
use crate::{cache, errors::{AuthixError, AuthixResult}, login::LoginResponse, utils::Claims};

pub const ACCESS_TOKEN_EXP: usize = 1000 * 60 * 5;
pub const REFRESH_TOKEN_EXP: usize = 1000 * 60 * 60 * 24 * 7;

pub async fn create_token(sub: String, tenant_id: String) -> AuthixResult<LoginResponse> {
    let (access_token,access_exp,iat) = get_token(&sub, &tenant_id, ACCESS_TOKEN_EXP, "access").await?;
    let (refresh_token,_,_) = get_token(&sub, &tenant_id, REFRESH_TOKEN_EXP, "refresh").await?;
    Ok(LoginResponse { access_token, refresh_token, exp: access_exp, iat })
}

pub async fn get_token(sub: &str, tenant_id: &str, exp: usize, token_type: &str) -> AuthixResult<(String,usize,usize)> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis() as usize;
    let jwt_secret = env::var("JWT_DECODING_KEY")?;
    let claims = Claims { sub: sub.to_string(), tenant_id: tenant_id.to_string(), exp: now + exp, iat: now, token_type: token_type.to_string() };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    if "access" == token_type {
        let uid: u64 = sub
            .parse()
            .map_err(|e| AuthixError::InvalidCredentials(format!("invalid user id: {}", e)))?;
        cache::save_user_access_token(uid, &token, ACCESS_TOKEN_EXP)
            .await
            .map_err(|e| AuthixError::InvalidCredentials(format!("cache save error: {}", e)))?;
    }
    Ok((token,claims.exp,claims.iat))
}

#[allow(dead_code)]
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
