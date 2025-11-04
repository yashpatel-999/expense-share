use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{create_jwt, verify_password};
use crate::errors::{AppResult, AuthError, DatabaseError, ValidationError};
use crate::models::{LoginRequest, UserResponse};

pub struct AuthService;

impl AuthService {
    pub async fn login(pool: &PgPool, request: &LoginRequest) -> AppResult<(String, UserResponse)> {
        let user = sqlx::query_as!(
            crate::models::User,
            "SELECT * FROM users WHERE email = $1",
            request.email
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

        let is_valid = verify_password(&request.password, &user.password_hash)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials.into());
        }

        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| AuthError::MissingJwtSecret)?;

        let token = create_jwt(&user, &jwt_secret)?;

        let user_response = UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            is_admin: user.is_admin.unwrap_or(false),
        };

        Ok((token, user_response))
    }

    pub async fn register(
        pool: &PgPool,
        email: &str,
        username: &str,
        password: &str,
    ) -> AppResult<UserResponse> {
        if email.is_empty() || username.is_empty() || password.is_empty() {
            return Err(ValidationError::RequiredField(
                "email, username, and password".to_string(),
            )
            .into());
        }

        if password.len() < 8 {
            return Err(ValidationError::InvalidFormat(
                "password must be at least 8 characters".to_string(),
            )
            .into());
        }

        let password_hash = crate::auth::hash_password(password)?;

        let user = sqlx::query_as!(
            crate::models::User,
            "INSERT INTO users (email, username, password_hash) VALUES ($1, $2, $3) RETURNING *",
            email,
            username,
            password_hash
        )
        .fetch_one(pool)
        .await
        .map_err(|e| -> crate::errors::AppError {
            match e {
                sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                    if db_err.constraint().unwrap().contains("email") {
                        crate::errors::UserError::EmailAlreadyExists.into()
                    } else if db_err.constraint().unwrap().contains("username") {
                        crate::errors::UserError::UsernameAlreadyExists.into()
                    } else {
                        DatabaseError::ConstraintViolation(db_err.to_string()).into()
                    }
                }
                _ => DatabaseError::QueryFailed(e.to_string()).into(),
            }
        })?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            is_admin: user.is_admin.unwrap_or(false),
        })
    }

    pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<UserResponse> {
        let user = sqlx::query_as!(
            crate::models::User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        .ok_or(crate::errors::UserError::NotFound)?;

        Ok(UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            is_admin: user.is_admin.unwrap_or(false),
        })
    }
}
