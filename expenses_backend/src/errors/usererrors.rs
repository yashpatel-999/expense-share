use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum UserError {
    NotFound,
    EmailAlreadyExists,
    UsernameAlreadyExists,
    InvalidCredentials,
    Unauthorized,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NotFound => write!(f, "User not found"),
            UserError::EmailAlreadyExists => write!(f, "Email address already exists"),
            UserError::UsernameAlreadyExists => write!(f, "Username already exists"),
            UserError::InvalidCredentials => write!(f, "Invalid credentials"),
            UserError::Unauthorized => write!(f, "Unauthorized access"),
        }
    }
}

impl ResponseError for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::EmailAlreadyExists | UserError::UsernameAlreadyExists => {
                StatusCode::CONFLICT
            }
            UserError::InvalidCredentials | UserError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": "user_error",
            "message": self.to_string()
        }))
    }
}
