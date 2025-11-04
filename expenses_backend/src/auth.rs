use crate::errors::{AppResult, AuthError};
use crate::models::User;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,
}

pub fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AuthError::PasswordHashError(e.to_string()).into())
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, hash).map_err(|e| AuthError::PasswordHashError(e.to_string()).into())
}

pub fn create_jwt(user: &User, secret: &str) -> AppResult<String> {
    let expiration = Utc::now() + Duration::hours(24);

    let claims = Claims {
        user_id: user.id.to_string(),
        email: user.email.clone(),
        username: user.username.clone(),
        is_admin: user.is_admin.unwrap_or(false),
        exp: expiration.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| AuthError::JwtError(e.to_string()).into())
}

pub fn verify_jwt(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired.into(),
        jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken.into(),
        _ => AuthError::JwtError(e.to_string()).into(),
    })
}
