mod config;
mod db;
mod error;
mod cache;
mod utils;
mod models;
mod repositories;
mod services;
mod handlers;
mod middleware;
mod websocket;

use actix_web::{web, App, HttpServer, HttpResponse};
use config::Config;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load environment variables
    dotenv::dotenv().ok();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    log::info!("✅ Configuration loaded");

    // Create database connection pool
    let db_pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Test database connection
    db::test_connection(&db_pool)
        .await
        .expect("Database connection test failed");

    // Create Redis client
    let redis_client = cache::create_client(&config.redis_url)
        .expect("Failed to create Redis client");
    
    // Test Redis connection
    cache::test_connection(&redis_client)
        .expect("Redis connection test failed");

    let server_address = config.server_address();
    log::info!("🚀 Starting server at http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(config.clone()))
            // Public routes
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            // Auth routes
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/me", web::get().to(handlers::auth::get_me).wrap(middleware::AuthMiddleware))
                    .route("/logout", web::post().to(handlers::auth::logout).wrap(middleware::AuthMiddleware))
            )
            // Room routes (all protected)
            .service(
                web::scope("/api/rooms")
                    .wrap(middleware::AuthMiddleware)
                    .route("", web::get().to(handlers::room::list_rooms))
                    .route("", web::post().to(handlers::room::create_room))
                    .route("/{id}", web::get().to(handlers::room::get_room))
                    .route("/{id}", web::put().to(handlers::room::update_room))
                    .route("/{id}", web::delete().to(handlers::room::delete_room))
                    .route("/{id}/join", web::post().to(handlers::room::join_room))
                    .route("/{id}/leave", web::post().to(handlers::room::leave_room))
                    .route("/{id}/members", web::get().to(handlers::room::get_members))
            )
    })
    .bind(server_address)?
    .run()
    .await
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Welcome to Ngobrol API",
        "version": "1.0.0"
    }))
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
