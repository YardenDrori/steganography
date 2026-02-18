mod app_state;
mod auth;
mod dtos;
mod entities;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;
use shared_global::db::postgres::create_pool;

use crate::app_state::AppState;
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let eureka_url = std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "auth_service")
        .await
        .expect("Failed to fetch config from eureka");

    let jwt_private_key = config
        .jwt_private_key
        .expect("Eureka should provide jwt_private_key for auth_service");
    let jwt_public_key = config.jwt_public_key;
    let user_service_url = config
        .services
        .get("user_service")
        .cloned()
        .expect("user_service URL not found in eureka registry");

    tracing::info!(
        "Loaded config from eureka - private_key len={}, public_key len={}",
        jwt_private_key.len(),
        jwt_public_key.len()
    );

    // Register self with eureka
    let self_url =
        std::env::var("SELF_URL").unwrap_or_else(|_| "http://auth_service:3001".to_string());
    shared_global::eureka::register_service(&eureka_url, "auth_service", &self_url)
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
        jwt_private_key,
        jwt_public_key,
        user_service_url,
        pool: pool.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
        .route("/auth/logout", post(routes::auth::logout))
        .route(
            "/auth/deactivate",
            post(routes::account::deactivate_my_account),
        )
        .route("/public-key", get(routes::public_key::get_public_key))
        .route(
            "/admin/users/:id/activate",
            patch(routes::account::activate_user_admin),
        )
        .route(
            "/admin/users/:id/deactivate",
            patch(routes::account::deactivate_user_admin),
        )
        .route(
            "/internal/users/:id/tokens",
            delete(routes::tokens::revoke_user_tokens),
        )
        .with_state(app_state);

    // Spawn heartbeat task
    let eureka_url_clone = eureka_url.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) =
                shared_global::eureka::send_heartbeat(&eureka_url_clone, "auth_service").await
            {
                tracing::warn!("Heartbeat failed: {}", e);
            } else {
                tracing::info!("Heartbeat sent");
            }
        }
    });

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
