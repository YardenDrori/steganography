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
use axum::{Router, routing::get, routing::post};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in environment");

    let postgres_url = std::env::var("POSTGRES_URL")
        .expect("POSTGRES_URL must be set in environment");

    let pool = db::connections::create_pool(&postgres_url)
        .await
        .expect("Failed to create postgres database pool");

    let app_state = AppState {
        jwt_secret,
        pool,
    };

    let app = Router::new()
        .route("/", get(|| async { "Auth Service - Running" }))
        .route("/register", post(routes::auth::register))
        .route("/login", post(routes::auth::login))
        .with_state(app_state);

    // Run auth service on port 3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("🔐 Auth Service listening on port 3001");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
