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

impl AuthError{
    pub fn is_client_error(&self)->bool{
        matches!(self,AuthError::InvalidCredentials|AuthError::UserNotFound)
    }
    pub fn is_server_error(&self)->bool{
        !self.is_client_error()
    }
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthError::InvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            AuthError::UserNotFound => "AUTH_USER_NOT_FOUND",
            AuthError::DatabaseError(_) => "AUTH_DATABASE_ERROR",
            AuthError::JwtError(_) => "AUTH_JWT_ERROR",
            AuthError::PasswordHashError(_) => "AUTH_PASSWORD_ERROR",
            AuthError::MissingJwtSecret => "AUTH_CONFIG_ERROR",
        }
    }

}