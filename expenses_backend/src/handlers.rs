#[derive(serde::Serialize)]
pub struct ExpenseResponse {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub paid_by: uuid::Uuid,
    pub amount: rust_decimal::Decimal,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
}

pub async fn get_group_expenses(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let group_id = path.into_inner();

    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id=$1 AND user_id=$2)"
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error": "Not a group member"})));
    }

    let expenses = sqlx::query!(
        r#"SELECT e.*, u.username FROM expenses e JOIN users u ON e.paid_by = u.id WHERE e.group_id = $1 ORDER BY e.created_at DESC"#,
        group_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let response: Vec<ExpenseResponse> = expenses.into_iter().map(|e| ExpenseResponse {
        id: e.id,
        group_id: e.group_id.expect("group_id should not be null"),
        paid_by: e.paid_by.expect("paid_by should not be null"),
        amount: e.amount,
        description: e.description,
        created_at: e.created_at.expect("created_at should not be null"),
        username: e.username,
    }).collect();

    Ok(HttpResponse::Ok().json(response))
}
use actix_web::{web,HttpResponse,Result,HttpRequest};
use sqlx::PgPool;
use uuid::Uuid;
use std::str::FromStr;
use rust_decimal::Decimal;

use crate::models::*;
use crate::auth::*;

pub async fn login(
    pool:web::Data<PgPool>,
    form:web::Json<LoginRequest>,
)->Result<HttpResponse>{
    let user=sqlx::query_as::<_,User>(
        "SELECT * FROM users WHERE email=$1"
    )
    .bind(&form.email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if let Some(user)=user{
        if verify_password(&form.password,&user.password_hash)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        {
            let secret=std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let token=create_jwt(&user,&secret)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "token":token,
                "user":UserResponse{
                    id:user.id,
                    email:user.email,
                    username:user.username,
                    is_admin:user.is_admin,
                }
            })))
        }
    }
    Ok(HttpResponse::Unauthorized().json(serde_json::json!({
        "error":"Invalid credentials"
    })))
}

fn get_user_from_request(req: &HttpRequest) -> Result<Claims> {
    let auth_header = req.headers().get("Authorization")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("No auth header"))?;
    
    let auth_str = auth_header.to_str()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(actix_web::error::ErrorUnauthorized("Invalid auth format"));
    }
    
    let token = &auth_str[7..];
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    verify_jwt(token, &secret)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e))
}

pub async fn create_user(
    pool:web::Data<PgPool>,
    form:web::Json<CreateUser>,
    req:HttpRequest,
)->Result<HttpResponse>{
    let claims=get_user_from_request(&req)?;
    if !claims.is_admin{
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error":"Admin only"})));
    }

    let password_hash=hash_password(&form.password)
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let user=sqlx::query_as::<_,User>(
        "INSERT INTO users (email,username,password_hash) VALUES ($1,$2,$3) RETURNING *"
    )
    .bind(&form.email)
    .bind(&form.username)
    .bind(&password_hash)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().json(UserResponse{
        id:user.id,
        email:user.email,
        username:user.username,
        is_admin:user.is_admin,
    }))
}

pub async fn get_users(
    pool:web::Data<PgPool>,
    req:HttpRequest,
)->Result<HttpResponse>{
    let claims=get_user_from_request(&req)?;
    if !claims.is_admin{
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error":"Admin Only!"})));
    }
    let users=sqlx::query_as::<_,User>("SELECT * FROM users ORDER BY created_at")
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let user_responses:Vec<UserResponse>=users.into_iter()
    .map(|u| UserResponse {id:u.id,email:u.email,username:u.username,is_admin:u.is_admin})
    .collect();
    Ok(HttpResponse::Ok().json(user_responses))
}

pub async fn create_group(
    pool:web::Data<PgPool>,
    form:web::Json<CreateGroup>,
    req:HttpRequest,
)->Result<HttpResponse>{
    let claims=get_user_from_request(&req)?;
    if !claims.is_admin{
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error":"Admin Only!"})));
    }

    let creator_id=Uuid::from_str(&claims.user_id)
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let group =sqlx::query_as::<_,Group>("INSERT INTO groups (name,created_by) VALUES ($1,$2) RETURNING *")
    .bind(&form.name)
    .bind(creator_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    for user_id in &form.user_ids{
        sqlx::query("INSERT INTO group_members (group_id,user_id) VALUES ($1,$2)")
        .bind(group.id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }
    Ok(HttpResponse::Created().json(group))
}

pub async fn get_user_groups(
    pool:web::Data<PgPool>,
    req:HttpRequest,
)->Result<HttpResponse>{
    let claims=get_user_from_request(&req)?;
    let user_id=Uuid::from_str(&claims.user_id)
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let groups=sqlx::query_as::<_,Group>(
        "SELECT g.* FROM groups g JOIN group_members gm ON g.id=gm.group_id WHERE gm.user_id=$1"
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(groups))
}

pub async fn add_expense(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    form: web::Json<CreateExpense>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    let group_id = path.into_inner();

    // Check if user is member of group
    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)"
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if !is_member {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error": "Not a group member"})));
    }

    let expense = sqlx::query_as::<_, Expense>(
        "INSERT INTO expenses (group_id, paid_by, amount, description) 
         VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(group_id)
    .bind(user_id)
    .bind(Decimal::from_f64_retain(form.amount).unwrap())
    .bind(&form.description)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().json(expense))
}

pub async fn get_group_balances(
    pool:web::Data<PgPool>,
    path:web::Path<Uuid>,
    req:HttpRequest,
)->Result<HttpResponse>{
    let claims=get_user_from_request(&req)?;
    let user_id=Uuid::from_str(&claims.user_id)
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let group_id=path.into_inner();

    let is_member=sqlx::query_scalar::<_,bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id=$1 AND user_id=$2)"
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if !is_member{
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error":"Not a group member"})));
    }

    let balances=calculate_balances(&pool,group_id).await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(balances))
}

async fn calculate_balances(pool:&PgPool,group_id:Uuid)->Result<Vec<Balance>,sqlx::Error>{
    let expenses = sqlx::query!(
        "SELECT e.paid_by, e.amount, u.username 
         FROM expenses e 
         JOIN users u ON e.paid_by = u.id 
         WHERE e.group_id = $1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    let payments=sqlx::query!(
        "SELECT p.from_user_id, p.to_user_id,p.amount
        FROM payments p
        WHERE p.group_id=$1",
        group_id
    )
    .fetch_all(pool)
    .await?;

    let members=sqlx::query!("SELECT u.id, u.username
    From users u
    JOIN group_members gm ON u.id=gm.user_id
    WHERE gm.group_id=$1",
    group_id
    )
    .fetch_all(pool)
    .await?;

    let member_count=members.len() as f64;
    let total_expenses:f64=expenses.iter()
    .map(|e| e.amount.to_string().parse::<f64>().unwrap_or(0.0))
    .sum();

    let per_person_share=total_expenses/member_count;

    let mut balances=Vec::new();

    for member in members{
        let total_paid:f64=expenses.iter()
        .filter(|e| e.paid_by==Some(member.id))
        .map(|e| e.amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

        let total_payments_made:f64=payments.iter()
        .filter(|p|p.from_user_id==Some(member.id))
        .map(|p|p.amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

        let total_payments_received:f64=payments.iter()
        .filter(|p|p.to_user_id==Some(member.id))
        .map(|p|p.amount.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

        let balance=total_paid-total_payments_received-per_person_share+total_payments_made;

        balances.push(Balance{
            user_id:member.id,
            username:member.username,
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
) -> Result<HttpResponse> {
    let claims = get_user_from_request(&req)?;
    let user_id = Uuid::from_str(&claims.user_id)
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    let group_id = path.into_inner();

    sqlx::query!(
        "INSERT INTO payments (group_id, from_user_id, to_user_id, amount) 
         VALUES ($1, $2, $3, $4)",
        group_id,
        user_id,
        form.to_user_id,
        Decimal::from_f64_retain(form.amount).unwrap()
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Payment recorded"})))
}