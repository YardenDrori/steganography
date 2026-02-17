use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::app_state::AppState;
use axum::routing::{get, post};
use axum::Router;

mod app_state;
mod dtos;
mod errors;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_private_key: String =
        std::env::var("JWT_PRIVATE_KEY").expect("jwt_private_key must be set in env");
    let jwt_public_key: String =
        std::env::var("JWT_PUBLIC_KEY").expect("jwt_public_key must be set in env");
    let registered_services: Arc<RwLock<HashMap<String, String>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let app_state = AppState {
        jwt_private_key,
        jwt_public_key,
        registered_services,
    };

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/register", post(routes::register::register))
        .route("/config/:service_name", get(routes::config::get_config))
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
