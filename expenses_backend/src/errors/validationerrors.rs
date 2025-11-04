use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    RequiredField(String),
    InvalidFormat(String),
    InvalidLength(String),
    InvalidRange(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::RequiredField(field) => write!(f, "Required field missing: {}", field),
            ValidationError::InvalidFormat(expected) => {
                write!(f, "Invalid format, expected: {}", expected)
            }
            ValidationError::InvalidLength(msg) => write!(f, "Invalid length: {}", msg),
            ValidationError::InvalidRange(msg) => write!(f, "Invalid range: {}", msg),
        }
    }
}

impl ResponseError for ValidationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": "validation_error",
            "message": self.to_string()
        }))
    }
}
