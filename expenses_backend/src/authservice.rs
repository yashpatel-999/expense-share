use sqlx::PgPool;
use crate::models::{User,LoginRequest,UserResponse};
use crate::auth::{verify_password,create_jwt};
use crate::autherrors::AuthError;

pub struct AuthService;

impl AuthService{
    pub async fn login(
        pool:&PgPool,
        login_request:&LoginRequest
    )->Result<(String,UserResponse),AuthError>{
        let user=sqlx::query_as::<_,User>(
            "SELECT * FROM users WHERE email=$1"
        )
        .bind(&login_request.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        
        let user=user.ok_or(AuthError::UserNotFound)?;

        let is_valid=verify_password(&login_request.password,&user.password_hash)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        if !is_valid{
            return Err(AuthError::InvalidCredentials);
        }

        let secret=std::env::var("JWT_SECRET")
            .map_err(|_| AuthError::MissingJwtSecret)?;

        let token=create_jwt(&user,&secret)
            .map_err(|e| AuthError::JwtError(e.to_string()))?;

        let user_response=UserResponse{
            id:user.id,
            email:user.email,
            username:user.username,
            is_admin:user.is_admin,
        };
        Ok((token,user_response))
    }
}