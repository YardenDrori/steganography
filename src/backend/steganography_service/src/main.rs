mod app_state;
pub mod dtos;
pub mod errors;
mod routes;
use std::sync::{Arc, RwLock};

use crate::app_state::AppState;
use axum::Router;
use tower_http::trace::TraceLayer;
mod services;
use axum::routing::post;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let eureka_url = std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "steganography_service")
        .await
        .expect("Failed to fetch config from eureka");

    let config = Arc::new(RwLock::new(config));

    tracing::info!("Loaded config from eureka");

    // Register self with eureka
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://steganography_service:3003".to_string());
    shared_global::eureka::register_service(&eureka_url, "steganography_service", &self_url)
        .await
        .expect("Failed to register with eureka");

    let client = reqwest::Client::new();

    let app_state = AppState {
        client,
        eureka_config: Arc::clone(&config),
    };

    // Build router
    let app = Router::new()
        .route("/embed/video", post(routes::embed_video::embed_video))
        // .route("/auth/register", post(routes::auth::register))
        // .route("/auth/login", post(routes::auth::login))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Spawn heartbeat task
    let eureka_url_clone = eureka_url.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) =
                shared_global::eureka::send_heartbeat(&eureka_url_clone, "steganography_service")
                    .await
            {
                tracing::warn!("Heartbeat failed: {}", e);
            } else {
                tracing::debug!("Heartbeat sent");
            }
        }
    });
    //spawn refresh configs task
    let configs_for_refresh = Arc::clone(&config);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            match shared_global::eureka::fetch_config(&eureka_url, "steganography_service").await {
                Ok(fresh_config) => {
                    let mut configs = configs_for_refresh.write().unwrap();
                    *configs = fresh_config;
                    tracing::debug!("Refreshed service URLs from eureka");
                }
                Err(e) => tracing::warn!("Failed to refresh config: {}", e),
            }
        }
    });

    // Start server on port 3003
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003")
        .await
        .expect("Failed to bind to port 3003");

    tracing::info!(
        "Steganography service listening on {}",
        listener.local_addr()?
    );

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
