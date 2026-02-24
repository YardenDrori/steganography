use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::app_state::{AppState, ServiceEntry};
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;

mod app_state;
mod dtos;
mod errors;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let jwt_private_key: String =
        std::env::var("JWT_PRIVATE_KEY").expect("jwt_private_key must be set in env");
    let jwt_public_key: String =
        std::env::var("JWT_PUBLIC_KEY").expect("jwt_public_key must be set in env");
    let registered_services: Arc<RwLock<HashMap<String, ServiceEntry>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let access_token_duration_minutes: i64 = std::env::var("ACCESS_TOKEN_DURATION_MINS")
        .expect("access_token_duration_minutes must be set in env")
        .parse()
        .expect("failed to parse access_token_duration_minutes to i64");
    let refresh_token_duration_minutes: i64 = std::env::var("REFRESH_TOKEN_DURATION_MINS")
        .expect("refresh_token_duration_minutes must be set in env")
        .parse()
        .expect("failed to parse refresh_token_duration_minutes to i64");

    let app_state = AppState {
        jwt_private_key,
        jwt_public_key,
        jwt_duration_access_and_refresh: (
            access_token_duration_minutes,
            refresh_token_duration_minutes,
        ),
        registered_services: Arc::clone(&registered_services),
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            let mut services = registered_services.write().unwrap();
            let before = services.len();

            services.retain(|_, entry| entry.last_heartbeat.elapsed().as_secs() < 90);

            let removed = before - services.len();
            if removed > 0 {
                tracing::info!("Cleaned up {} stale services", removed);
            }
        }
    });

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/register", post(routes::register::register))
        .route("/config/:service_name", get(routes::config::get_config))
        .route("/heartbeat", post(routes::heartbeat::heartbeat))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005")
        .await
        .expect("Failed to bind to port 3005");

    tracing::info!("Eureka service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
