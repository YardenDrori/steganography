mod app_state;
mod auth;
mod dtos;
mod entities;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;
use shared_global::eureka;
use shared_global::{db::postgres::create_pool, eureka::EurekaConfig};
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};

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

    let config: Arc<RwLock<eureka::EurekaConfig>> = Arc::new(RwLock::new(config));

    tracing::info!("Loaded config from eureka");

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
        pool: pool.clone(),
        eureka_config: Arc::clone(&config),
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

    //refresh configs
    let eureka_url_clone = eureka_url.clone();
    let configs_for_refresh = Arc::clone(&config);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            match shared_global::eureka::fetch_config(&eureka_url_clone, "auth_service").await {
                Ok(fresh_config) => {
                    let mut configs = configs_for_refresh.write().unwrap();
                    *configs = fresh_config;
                    tracing::info!("Refreshed service URLs from eureka");
                }
                Err(e) => tracing::warn!("Failed to refresh config: {}", e),
            }
        }
    });

    // Spawn heartbeat task
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) = shared_global::eureka::send_heartbeat(&eureka_url, "auth_service").await
            {
                tracing::warn!("Heartbeat failed: {}", e);
            } else {
                tracing::info!("Heartbeat sent");
            }
        }
    });

    for i in 1..10 {
        if !config.read().unwrap().services.contains_key("user_service") {
            tracing::info!("couldn't find user_service in eureka configs waiting 30S and retrying. (attempt {}/10)", i);
            sleep(Duration::new(30, 0)).await;
        } else {
            break;
        }
    }
    if !config.read().unwrap().services.contains_key("user_service") {
        panic!("no user_service found from eureka service maximum attempt limit reached");
    }

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
