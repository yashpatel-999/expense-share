use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, FromRow)]
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

#[derive(Debug, Serialize, FromRow)]
pub struct Expense {
    pub id: Uuid,
    pub group_id: Uuid,
    pub paid_by: Uuid,
    pub amount: Decimal,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpense {
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct Balance {
    pub user_id: Uuid,
    pub username: String,
    pub balance: f64, // positive = should receive, negative = should pay
}

#[derive(Debug, Serialize)]
pub struct Settlement {
    pub from_user: String,
    pub to_user: String,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayment {
    pub to_user_id: Uuid,
    pub amount: f64,
}