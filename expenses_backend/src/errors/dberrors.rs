use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    TransactionFailed(String),
    ConstraintViolation(String),
    NotFound,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed(msg) => {
                write!(f, "Database connection failed: {}", msg)
            }
            DatabaseError::QueryFailed(msg) => write!(f, "Database query failed: {}", msg),
            DatabaseError::TransactionFailed(msg) => {
                write!(f, "Database transaction failed: {}", msg)
            }
            DatabaseError::ConstraintViolation(msg) => {
                write!(f, "Database constraint violation: {}", msg)
            }
            DatabaseError::NotFound => write!(f, "Database record not found"),
        }
    }
}

impl ResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            DatabaseError::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": "database_error",
            "message": self.to_string()
        }))
    }
}
