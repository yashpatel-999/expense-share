use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::{AppResult, ValidationError};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub is_admin: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl CreateUser {
    pub fn validate(&self) -> AppResult<()> {
        if self.email.is_empty() {
            return Err(ValidationError::RequiredField("email".to_string()).into());
        }
        if self.username.is_empty() {
            return Err(ValidationError::RequiredField("username".to_string()).into());
        }
        if self.password.is_empty() {
            return Err(ValidationError::RequiredField("password".to_string()).into());
        }
        if self.password.len() < 8 {
            return Err(ValidationError::InvalidFormat(
                "password must be at least 8 characters".to_string(),
            )
            .into());
        }
        if !self.email.contains('@') {
            return Err(ValidationError::InvalidFormat("valid email address".to_string()).into());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.email.is_empty() {
            return Err(ValidationError::RequiredField("email".to_string()).into());
        }
        if self.password.is_empty() {
            return Err(ValidationError::RequiredField("password".to_string()).into());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroup {
    pub name: String,
    pub user_ids: Vec<Uuid>,
}

impl CreateGroup {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.is_empty() {
            return Err(ValidationError::RequiredField("group name".to_string()).into());
        }
        if self.user_ids.is_empty() {
            return Err(ValidationError::RequiredField("at least one user ID".to_string()).into());
        }
        if self.name.len() > 255 {
            return Err(ValidationError::InvalidFormat(
                "group name must be less than 255 characters".to_string(),
            )
            .into());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Expense {
    pub id: Uuid,
    pub group_id: Uuid,
    pub paid_by: Uuid,
    pub amount: Decimal,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub paid_by: Uuid,
    pub amount: Decimal,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpense {
    pub amount: f64,
    pub description: String,
}

impl CreateExpense {
    pub fn validate(&self) -> AppResult<()> {
        if self.description.is_empty() {
            return Err(ValidationError::RequiredField("description".to_string()).into());
        }
        if self.amount <= 0.0 {
            return Err(ValidationError::InvalidFormat(
                "amount must be greater than 0".to_string(),
            )
            .into());
        }
        if self.amount > 999999.99 {
            return Err(ValidationError::InvalidFormat(
                "amount must be less than 1,000,000".to_string(),
            )
            .into());
        }
        if self.description.len() > 500 {
            return Err(ValidationError::InvalidFormat(
                "description must be less than 500 characters".to_string(),
            )
            .into());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct Balance {
    pub user_id: Uuid,
    pub username: String,
    pub balance: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayment {
    pub to_user_id: String,
    pub amount: f64,
}

impl CreatePayment {
    pub fn validate(&self) -> AppResult<()> {
        if self.to_user_id.is_empty() {
            return Err(ValidationError::RequiredField("to_user_id".to_string()).into());
        }
        if self.amount <= 0.0 {
            return Err(ValidationError::InvalidFormat(
                "payment amount must be greater than 0".to_string(),
            )
            .into());
        }
        if self.amount > 999999.99 {
            return Err(ValidationError::InvalidFormat(
                "payment amount must be less than 1,000,000".to_string(),
            )
            .into());
        }
        uuid::Uuid::parse_str(&self.to_user_id)
            .map_err(|_| ValidationError::InvalidFormat("valid UUID for to_user_id".to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub group_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}
