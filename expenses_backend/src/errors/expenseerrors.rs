use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum ExpenseError {
    NotFound,
    InvalidAmount(String),
    CalculationError(String),
    InsufficientFunds,
    DuplicateExpense,
}

impl fmt::Display for ExpenseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpenseError::NotFound => write!(f, "Expense not found"),
            ExpenseError::InvalidAmount(msg) => write!(f, "Invalid amount: {}", msg),
            ExpenseError::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
            ExpenseError::InsufficientFunds => write!(f, "Insufficient funds for this operation"),
            ExpenseError::DuplicateExpense => write!(f, "Duplicate expense detected"),
        }
    }
}

impl ResponseError for ExpenseError {
    fn status_code(&self) -> StatusCode {
        match self {
            ExpenseError::NotFound => StatusCode::NOT_FOUND,
            ExpenseError::InvalidAmount(_) | ExpenseError::DuplicateExpense => {
                StatusCode::BAD_REQUEST
            }
            ExpenseError::InsufficientFunds => StatusCode::PAYMENT_REQUIRED,
            ExpenseError::CalculationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": "expense_error",
            "message": self.to_string()
        }))
    }
}
