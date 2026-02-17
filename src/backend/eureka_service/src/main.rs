use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::app_state::AppState;
use axum::routing::{get, post};
use axum::Router;
use shared_global::db::postgres::create_pool;

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
    let internal_api_key: String =
        std::env::var("INTERNAL_API_KEY").expect("internal_api_key must be set in env");

    tracing::info!("todo");

    let registered_services: Arc<RwLock<HashMap<String, String>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let app_state = AppState {
        jwt_private_key,
        jwt_public_key,
        internal_api_key,
        registered_services,
    };

    let app = Router::new().with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005")
        .await
        .expect("Failed to bind to port 3005");

    tracing::info!("Eureka service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
