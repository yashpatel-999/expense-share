use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use expenses_backend::errors::AppError;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal("DATABASE_URL must be set".to_string()))?;

    let _jwt_secret = env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT_SECRET must be set".to_string()))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4200")
            .allowed_origin("http://127.0.0.1:4200")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route(
                "/api/auth/login",
                web::post().to(expenses_backend::handlers::login),
            )
            .route(
                "/api/auth/register",
                web::post().to(expenses_backend::handlers::register),
            )
            .route(
                "/api/users",
                web::post().to(expenses_backend::handlers::create_user),
            )
            .route(
                "/api/users",
                web::get().to(expenses_backend::handlers::get_users),
            )
            .route(
                "/api/groups",
                web::post().to(expenses_backend::handlers::create_group),
            )
            .route(
                "/api/groups",
                web::get().to(expenses_backend::handlers::get_user_groups),
            )
            .route(
                "/api/groups/{group_id}/expenses",
                web::get().to(expenses_backend::handlers::get_group_expenses),
            )
            .route(
                "/api/groups/{group_id}/expenses",
                web::post().to(expenses_backend::handlers::add_expense),
            )
            .route(
                "/api/groups/{group_id}/balances",
                web::get().to(expenses_backend::handlers::get_group_balances),
            )
            .route(
                "/api/groups/{group_id}/payments",
                web::post().to(expenses_backend::handlers::make_payment),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
