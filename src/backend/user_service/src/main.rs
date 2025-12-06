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

    let auth_service_url =
        std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());

    let internal_api_key =
        std::env::var("INTERNAL_API_KEY").expect("INTERNAL_API_KEY must be set in env");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Fetch JWT public key from auth service
    tracing::info!("Fetching JWT public key from auth service at {}", auth_service_url);
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/public-key", auth_service_url))
        .send()
        .await
        .expect("Failed to fetch public key from auth service");

    #[derive(serde::Deserialize)]
    struct PublicKeyResponse {
        public_key: String,
    }

    let public_key_response: PublicKeyResponse = response
        .json()
        .await
        .expect("Failed to parse public key response");

    // JSON serialization escapes newlines, so we need to convert them back
    let jwt_public_key = public_key_response.public_key.replace(r"\n", "\n");

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
        .route("/internal/auth/verify-credentials", post(auth::verify_credentials))
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
