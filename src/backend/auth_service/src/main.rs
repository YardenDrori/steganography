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

    // Load environment variables
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    let pool = db::connections::create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run migrations
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    // Create app state
    let app_state = AppState { jwt_secret, pool };

    // Build router
    let app = Router::new()
        .route("/", get(|| async { "Auth Service" }))
        .route("/register", post(routes::auth::register))
        .route("/login", post(routes::auth::login))
        .with_state(app_state);

    // Start server on port 3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    tracing::info!("Auth service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
