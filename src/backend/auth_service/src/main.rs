mod app_state;
mod auth;
mod db;
mod dtos;
mod entities;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;

use crate::app_state::AppState;
use axum::{routing::get, routing::post, Router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("Starting auth service");

    // Load environment variables
    dotenvy::dotenv().ok();
    tracing::debug!("Environment variables loaded");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");
    if jwt_secret.len() < 32 {
        panic!("JWT_SECRET must be at least 32 characters for security");
    }
    tracing::debug!("JWT secret validated");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    tracing::info!("Connecting to database");
    let pool = db::connections::create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");
    tracing::info!("Database connection pool created successfully");

    // Create app state
    let app_state = AppState { jwt_secret, pool };

    // Build router
    tracing::debug!("Building router");
    let app = Router::new()
        .route("/", get(|| async { "Auth Service" }))
        .route("/register", post(routes::auth::register))
        .route("/login", post(routes::auth::login))
        .route("/refresh", post(routes::auth::refresh))
        .route("/logout", post(routes::auth::logout))
        .with_state(app_state);

    // Start server on port 3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("Auth service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    tracing::info!("Server shutting down");
    Ok(())
}
