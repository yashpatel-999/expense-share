use actix_web::{HttpResponse,ResponseError};
use actix_web::http::StatusCode;
use std::fmt;

#[derive(Debug)]
pub enum AuthError{
    InvalidCredentials,
    UserNotFound,
    DatabaseError(String),
    JwtError(String),
    PasswordHashError(String),
    MissingJwtSecret,
    TokenExpired,
    InvalidToken,
    InsufficientPermissions,
}

impl fmt::Display for AuthError{
    fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self{
            AuthError::InvalidCredentials=>write!(f,"Invalid email or password"),
            AuthError::UserNotFound=>write!(f,"User not found"),
            AuthError::DatabaseError(msg)=>write!(f,"Database error: {}",msg),
            AuthError::JwtError(msg)=>write!(f,"JWT error: {}",msg),
            AuthError::PasswordHashError(msg)=>write!(f,"Password hashing error: {}",msg),
            AuthError::MissingJwtSecret=>write!(f,"JWT secret not configured"),
            AuthError::TokenExpired=>write!(f,"Authentication Token expired"),
            AuthError::InvalidToken=>write!(f,"Invalid Authentication Token"),
            AuthError::InsufficientPermissions=>write!(f,"Insufficient permissions")
        }
    }
}

impl ResponseError for AuthError{
    fn status_code(&self)->StatusCode{
        match self{
            AuthError::InvalidCredentials=>StatusCode::UNAUTHORIZED,
            AuthError::UserNotFound=>StatusCode::UNAUTHORIZED,
            AuthError::DatabaseError(_)=>StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::JwtError(_)=>StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::PasswordHashError(_)=>StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::MissingJwtSecret=>StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::TokenExpired=>StatusCode::UNAUTHORIZED,
            AuthError::InvalidToken=>StatusCode::UNAUTHORIZED,
            AuthError::InsufficientPermissions=>StatusCode::FORBIDDEN
        }
    }
    fn error_response(&self)->HttpResponse{
        let status =self.status_code();
        let error_response=match  self {
            AuthError::InvalidCredentials=>{
                serde_json::json!({
                    "error":"Invalid credentials",
                    "message":"Email or password is incorrect",
                    "code":"AUTH_INVALID_CREDENTIALS",
                    "status_code":status.as_u16()
                })
            }
            AuthError::UserNotFound=>{
                serde_json::json!({
                    "error":"User not found",
                    "message":"No user found with the provided email",
                    "code":"AUTH_USER_NOT_FOUND",
                    "status":status.as_u16()
                })
            }
            AuthError::TokenExpired=>{
                serde_json::json!({
                    "error":"Token expired",
                    "message":"Authentication token has expired",
                    "code":"AUTH_TOKEN_EXPIRED",
                    "status":status.as_u16()
                })
            }
            AuthError::InvalidToken => {
                serde_json::json!({
                    "error": "Invalid token",
                    "message": "Authentication token is invalid",
                    "code": "AUTH_INVALID_TOKEN",
                    "status": status.as_u16()
                })
            }
            AuthError::InsufficientPermissions => {
                serde_json::json!({
                    "error": "Insufficient permissions",
                    "message": "You don't have permission to perform this action",
                    "code": "AUTH_INSUFFICIENT_PERMISSIONS",
                    "status": status.as_u16()
                })
            }
            AuthError::DatabaseError(msg)=>{
                log::error!("Datbase error: {}",msg);
                serde_json::json!({
                    "error":"Internal server error",
                    "message":"A database error occured",
                    "code":"AUTH_DATABASE_ERROR",
                    "status":status.as_u16()
                })
            }
            AuthError::JwtError(msg)=>{
                log::error!("JWT error: {}",msg);
                serde_json::json!({
                    "error":"Authentication service error",
                    "message":"Failed to generate authentication token",
                    "code":"AUTH_JWT_ERROR",
                    "status":status.as_u16()
                })
            }
            AuthError::PasswordHashError(msg)=>{
                log::error!("Password hash error: {}",msg);
                serde_json::json!({
                    "error":"Authentication service error",
                    "message":"Failed to verify password",
                    "code":"AUTH_PASSWORD_ERROR",
                    "status":status.as_u16()
                })
            }
            AuthError::MissingJwtSecret => {
                log::error!("JWT_SECRET environment variable not configured");
                serde_json::json!({
                    "error": "Server configuration error",
                    "message": "Authentication service not properly configured",
                    "code": "AUTH_CONFIG_ERROR",
                    "status": status.as_u16()
                })
            }
        };
        HttpResponse::build(status).json(error_response)
    }
}