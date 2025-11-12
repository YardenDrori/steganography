use axum::routing::get;
use axum::{routing::post, Router};
use routes::get_users::{get_current_profile, get_user};
use shared_global::db::postgres::create_pool;

use crate::app_state::AppState;
mod app_state;
mod dtos;
mod entities;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in env");
    if jwt_secret.len() < 32 {
        panic!("JWT_SECRET must be at least 32 characters for security");
    }

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Create database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run database migrations
    sqlx::migrate!().run(&pool).await?;

    // Create app state
    let app_state = AppState {
        pool: pool,
        jwt_secret: jwt_secret,
    };

    // Build router
    let app = Router::new()
        .route("/users/me", get(get_current_profile))
        .route("/users/:id", get(get_user))
        .with_state(app_state);

    // Start server on port 3002
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002")
        .await
        .expect("Failed to bind to port 3002");

    tracing::info!("User service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
