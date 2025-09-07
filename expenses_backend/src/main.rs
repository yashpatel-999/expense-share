mod models;
mod database;
mod auth;
mod handlers;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let pool = database::create_pool().await
        .expect("Failed to create database pool");

    println!("Starting server at http://localhost:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/login", web::post().to(handlers::login))
            .route("/admin/users", web::post().to(handlers::create_user))
            .route("/admin/users", web::get().to(handlers::get_users))
            .route("/admin/groups", web::post().to(handlers::create_group))
            .route("/groups", web::get().to(handlers::get_user_groups))
            .route("/groups/{id}/expenses", web::post().to(handlers::add_expense))
            .route("/groups/{id}/expenses", web::get().to(handlers::get_group_expenses))
            .route("/groups/{id}/balances", web::get().to(handlers::get_group_balances))
            .route("/groups/{id}/payments", web::post().to(handlers::make_payment))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}