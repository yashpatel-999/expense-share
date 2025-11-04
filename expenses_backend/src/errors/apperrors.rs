use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

use crate::errors::autherrors::AuthError;
use crate::errors::dberrors::DatabaseError;
use crate::errors::expenseerrors::ExpenseError;
use crate::errors::grouperrors::GroupError;
use crate::errors::usererrors::UserError;
use crate::errors::validationerrors::ValidationError;

#[derive(Debug)]
pub enum AppError {
    Auth(AuthError),
    Database(DatabaseError),
    Validation(ValidationError),
    Expense(ExpenseError),
    User(UserError),
    Group(GroupError),
    Internal(String),
    NotFound(String),
    BadRequest(String),
    Forbidden(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Auth(err) => write!(f, "Authentication error: {}", err),
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Validation(err) => write!(f, "Validation error: {}", err),
            AppError::Expense(err) => write!(f, "Expense error: {}", err),
            AppError::User(err) => write!(f, "User error: {}", err),
            AppError::Group(err) => write!(f, "Group error: {}", err),
            AppError::Internal(msg) => write!(f, "Internal server error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Auth(err) => err.status_code(),
            AppError::Database(err) => err.status_code(),
            AppError::Validation(err) => err.status_code(),
            AppError::Expense(err) => err.status_code(),
            AppError::User(err) => err.status_code(),
            AppError::Group(err) => err.status_code(),
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Auth(err) => err.error_response(),
            AppError::Database(err) => err.error_response(),
            AppError::Validation(err) => err.error_response(),
            AppError::Expense(err) => err.error_response(),
            AppError::User(err) => err.error_response(),
            AppError::Group(err) => err.error_response(),
            _ => {
                let status = self.status_code();
                HttpResponse::build(status).json(serde_json::json!({
                    "error": "app_error",
                    "message": self.to_string()
                }))
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Auth(err)
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        AppError::Database(err)
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::Validation(err)
    }
}

impl From<ExpenseError> for AppError {
    fn from(err: ExpenseError) -> Self {
        AppError::Expense(err)
    }
}

impl From<UserError> for AppError {
    fn from(err: UserError) -> Self {
        AppError::User(err)
    }
}

impl From<GroupError> for AppError {
    fn from(err: GroupError) -> Self {
        AppError::Group(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(DatabaseError::QueryFailed(err.to_string()))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Auth(AuthError::PasswordHashError(err.to_string()))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Auth(AuthError::JwtError(err.to_string()))
    }
}

impl From<uuid::Error> for AppError {
    fn from(_err: uuid::Error) -> Self {
        AppError::Validation(ValidationError::InvalidFormat("valid UUID".to_string()))
    }
}

impl From<rust_decimal::Error> for AppError {
    fn from(err: rust_decimal::Error) -> Self {
        AppError::Validation(ValidationError::InvalidFormat(format!(
            "valid decimal: {}",
            err
        )))
    }
}

pub type AppResult<T> = Result<T, AppError>;
