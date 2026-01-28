use crate::app_state::AppState;
use crate::routes::{auth, delete_users, patch_users, post_users, sync};
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

    let eureka_url =
        std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "user_service")
        .await
        .expect("Failed to fetch config from eureka");

    let jwt_public_key = config.jwt_public_key;
    let internal_api_key = config.internal_api_key;
    let auth_service_url = config
        .services
        .get("auth_service")
        .cloned()
        .unwrap_or_else(|| "http://auth_service:3001".to_string());

    tracing::info!(
        "Loaded config from eureka - public_key len={}, auth_service_url={}",
        jwt_public_key.len(),
        auth_service_url
    );

    // Register self with eureka
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://user_service:3002".to_string());
    shared_global::eureka::register_service(&eureka_url, "user_service", &self_url)
        .await
        .expect("Failed to register with eureka");

    // Create database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run database migrations
    sqlx::migrate!().run(&pool).await?;

    // Create app state
    let app_state = AppState {
        pool: pool,
        jwt_public_key: jwt_public_key,
        internal_api_key: internal_api_key,
        auth_service_url: auth_service_url,
    };

    // Build router
    let app = Router::new()
        .route("/users/me", get(get_users::get_current_profile))
        .route("/users/me", patch(patch_users::update_my_profile))
        .route("/users/:id", get(get_users::get_user))
        .route("/users/:id", patch(patch_users::update_user))
        .route("/users", post(post_users::create_user))
        .route("/users/:id", delete(delete_users::delete_user))
        .route("/internal/users/:id/status", patch(sync::sync_user_status))
        .route(
            "/internal/auth/verify-credentials",
            post(auth::verify_credentials),
        )
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
