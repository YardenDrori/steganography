use crate::app_state::AppState;
use crate::routes::{delete_users, patch_users, post_users};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use routes::get_users;
use shared_global::db::postgres::create_pool;
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

    let internal_api_key =
        std::env::var("INTERNAL_API_KEY").expect("INTERNAL_API_KEY must be set in env");

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
        internal_api_key: internal_api_key,
    };

    // Build router
    let app = Router::new()
        .route("/users/me", get(get_users::get_current_profile))
        .route("/users/me", patch(patch_users::update_my_profile))
        .route("/users/:id", get(get_users::get_user))
        .route("/users/:id", patch(patch_users::update_user))
        .route("/users", post(post_users::create_user))
        .route("/users/:id", delete(delete_users::delete_user))
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
