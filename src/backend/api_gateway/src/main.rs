mod app_state;
use std::sync::{Arc, RwLock};
mod proxy;
use axum::routing::any;
use axum::Router;
use tokio::time::{sleep, Duration};
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    // Load environment variables
    dotenvy::dotenv().ok();

    let eureka_url = std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "api_gateway")
        .await
        .expect("Failed to fetch config from eureka");

    let config = Arc::new(RwLock::new(config));

    let state = AppState {
        eureka_configs: Arc::clone(&config),
        client: reqwest::Client::new(),
    };

    tracing::info!("Loaded config from eureka");

    // Register self with eureka
    let self_url =
        std::env::var("SELF_URL").unwrap_or_else(|_| "http://api_gateway:3000".to_string());
    shared_global::eureka::register_service(&eureka_url, "api_gateway", &self_url)
        .await
        .expect("Failed to register with eureka");

    // cors stuff
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/api/auth/*path", any(proxy::auth_handler))
        .route("/api/user/*path", any(proxy::user_handler))
        .route("/api/files/*path", any(proxy::files_handler))
        .route("/api/embed/*path", any(proxy::embed_handler))
        .layer(cors)
        .with_state(state.into());

    // Spawn heartbeat task
    let eureka_url_clone = eureka_url.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) =
                shared_global::eureka::send_heartbeat(&eureka_url_clone, "api_gateway").await
            {
                tracing::warn!("Heartbeat failed: {}", e);
            } else {
                tracing::info!("Heartbeat sent");
            }
        }
    });

    let configs_for_refresh = Arc::clone(&config);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            match shared_global::eureka::fetch_config(&eureka_url, "api_gateway").await {
                Ok(fresh_config) => {
                    let mut configs = configs_for_refresh.write().unwrap();
                    *configs = fresh_config;
                    tracing::info!("Refreshed service URLs from eureka");
                }
                Err(e) => tracing::warn!("Failed to refresh config: {}", e),
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

    // Start server on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("API Gateway listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
