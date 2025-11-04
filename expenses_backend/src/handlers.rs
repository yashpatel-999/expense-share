use actix_web::{HttpRequest, HttpResponse, web};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::auth::*;
use crate::authservice::AuthService;
use crate::errors::{
    AppError, AppResult, AuthError, DatabaseError, ExpenseError, GroupError, UserError,
    ValidationError,
};
use crate::models::*;

pub async fn get_group_expenses(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;
    let group_id = path.into_inner();

    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id=$1 AND user_id=$2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if !is_member {
        return Err(GroupError::NotAMember.into());
    }

    let expenses = sqlx::query!(
        r#"SELECT e.*, u.username FROM expenses e JOIN users u ON e.paid_by = u.id WHERE e.group_id = $1 ORDER BY e.created_at DESC"#,
        group_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let response: Vec<ExpenseResponse> = expenses
        .into_iter()
        .map(|e| ExpenseResponse {
            id: e.id,
            group_id: e.group_id.expect("group_id should not be null"),
            paid_by: e.paid_by.expect("paid_by should not be null"),
            amount: e.amount,
            description: e.description,
            created_at: e.created_at.expect("created_at should not be null"),
            username: e.username,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn login(
    pool: web::Data<PgPool>,
    form: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    form.validate()?;

    let (token, user_response) = AuthService::login(pool.get_ref(), &form).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": user_response,
        "message": "Login Successful"
    })))
}

pub async fn register(
    pool: web::Data<PgPool>,
    form: web::Json<CreateUser>,
) -> AppResult<HttpResponse> {
    form.validate()?;

    let user_response =
        AuthService::register(pool.get_ref(), &form.email, &form.username, &form.password).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "user": user_response,
        "message": "User registered successfully"
    })))
}

fn get_user_from_request(req: &HttpRequest) -> AppResult<Claims> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or(AuthError::InvalidCredentials)?;

    let auth_str = auth_header
        .to_str()
        .map_err(|e| AuthError::JwtError(format!("Invalid auth header format: {}", e)))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidCredentials.into());
    }

    let token = &auth_str[7..];
    let secret = std::env::var("JWT_SECRET").map_err(|_| AuthError::MissingJwtSecret)?;

    verify_jwt(token, &secret)
}

pub async fn create_user(
    pool: web::Data<PgPool>,
    form: web::Json<CreateUser>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;

    if !claims.is_admin {
        return Err(AuthError::InsufficientPermissions.into());
    }

    form.validate()?;

    let password_hash = hash_password(&form.password)?;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email,username,password_hash) VALUES ($1,$2,$3) RETURNING *",
    )
    .bind(&form.email)
    .bind(&form.username)
    .bind(&password_hash)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
            if db_err.constraint().unwrap().contains("email") {
                AppError::User(UserError::EmailAlreadyExists)
            } else if db_err.constraint().unwrap().contains("username") {
                AppError::User(UserError::UsernameAlreadyExists)
            } else {
                AppError::Database(DatabaseError::ConstraintViolation(db_err.to_string()))
            }
        }
        _ => AppError::Database(DatabaseError::QueryFailed(e.to_string())),
    })?;

    Ok(HttpResponse::Created().json(UserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        is_admin: user.is_admin.unwrap_or(false),
    }))
}

pub async fn get_users(pool: web::Data<PgPool>, req: HttpRequest) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    if !claims.is_admin {
        return Err(AuthError::InsufficientPermissions.into());
    }

    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            email: u.email,
            username: u.username,
            is_admin: u.is_admin.unwrap_or(false),
        })
        .collect();
    Ok(HttpResponse::Ok().json(user_responses))
}

pub async fn create_group(
    pool: web::Data<PgPool>,
    form: web::Json<CreateGroup>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    if !claims.is_admin {
        return Err(AuthError::InsufficientPermissions.into());
    }

    form.validate()?;

    let creator_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;

    for user_id in &form.user_ids {
        let user_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(user_id)
                .fetch_one(pool.get_ref())
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if !user_exists {
            return Err(UserError::NotFound.into());
        }
    }

    let group = sqlx::query_as::<_, Group>(
        "INSERT INTO groups (name,created_by) VALUES ($1,$2) RETURNING *",
    )
    .bind(&form.name)
    .bind(creator_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    for user_id in &form.user_ids {
        sqlx::query("INSERT INTO group_members (group_id,user_id) VALUES ($1,$2)")
            .bind(group.id)
            .bind(user_id)
            .execute(pool.get_ref())
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
    }
    Ok(HttpResponse::Created().json(group))
}

pub async fn get_user_groups(pool: web::Data<PgPool>, req: HttpRequest) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;

    let groups = sqlx::query_as::<_, Group>(
        "SELECT g.* FROM groups g JOIN group_members gm ON g.id=gm.group_id WHERE gm.user_id=$1",
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(HttpResponse::Ok().json(groups))
}

pub async fn add_expense(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    form: web::Json<CreateExpense>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;
    let group_id = path.into_inner();

    form.validate()?;

    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if !is_member {
        return Err(GroupError::NotAMember.into());
    }

    let amount = Decimal::from_f64_retain(form.amount).ok_or(ValidationError::InvalidFormat(
        "valid decimal number".to_string(),
    ))?;

    let expense = sqlx::query_as::<_, Expense>(
        "INSERT INTO expenses (group_id, paid_by, amount, description) 
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(group_id)
    .bind(user_id)
    .bind(amount)
    .bind(&form.description)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(HttpResponse::Created().json(expense))
}

pub async fn get_group_balances(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;

    let group_id = path.into_inner();

    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id=$1 AND user_id=$2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if !is_member {
        return Err(GroupError::NotAMember.into());
    }

    let balances = calculate_balances(&pool, group_id).await?;

    Ok(HttpResponse::Ok().json(balances))
}

async fn calculate_balances(pool: &PgPool, group_id: Uuid) -> AppResult<Vec<Balance>> {
    let expenses = sqlx::query!(
        "SELECT e.paid_by, e.amount, u.username 
         FROM expenses e 
         JOIN users u ON e.paid_by = u.id 
         WHERE e.group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let payments = sqlx::query!(
        "SELECT p.from_user_id, p.to_user_id,p.amount
        FROM payments p
        WHERE p.group_id=$1",
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let members = sqlx::query!(
        "SELECT u.id, u.username
    From users u
    JOIN group_members gm ON u.id=gm.user_id
    WHERE gm.group_id=$1",
        group_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let member_count = members.len() as f64;
    let total_expenses: f64 = expenses
        .iter()
        .map(|e| e.amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    if member_count == 0.0 {
        return Err(ExpenseError::CalculationError("No members in group".to_string()).into());
    }

    let per_person_share = total_expenses / member_count;

    let mut balances = Vec::new();

    for member in members {
        let total_paid: f64 = expenses
            .iter()
            .filter(|e| e.paid_by == Some(member.id))
            .map(|e| e.amount.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_payments_made: f64 = payments
            .iter()
            .filter(|p| p.from_user_id == Some(member.id))
            .map(|p| p.amount.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_payments_received: f64 = payments
            .iter()
            .filter(|p| p.to_user_id == Some(member.id))
            .map(|p| p.amount.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();

        let balance = total_paid - total_payments_received - per_person_share + total_payments_made;

        balances.push(Balance {
            user_id: member.id,
            username: member.username,
            balance,
        });
    }
    Ok(balances)
}

pub async fn make_payment(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    form: web::Json<CreatePayment>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID".to_string()))?;
    let group_id = path.into_inner();

    form.validate()?;

    let to_user_id = Uuid::from_str(&form.to_user_id)
        .map_err(|_| ValidationError::InvalidFormat("valid UUID for to_user_id".to_string()))?;

    let from_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let to_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(group_id)
    .bind(to_user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if !from_member || !to_member {
        return Err(GroupError::NotAMember.into());
    }

    if user_id == to_user_id {
        return Err(
            ValidationError::InvalidFormat("cannot make payment to yourself".to_string()).into(),
        );
    }

    let amount = Decimal::from_f64_retain(form.amount).ok_or(ValidationError::InvalidFormat(
        "valid decimal number".to_string(),
    ))?;

    sqlx::query!(
        "INSERT INTO payments (group_id, from_user_id, to_user_id, amount) 
         VALUES ($1, $2, $3, $4)",
        group_id,
        user_id,
        to_user_id,
        amount
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Payment recorded"})))
}
