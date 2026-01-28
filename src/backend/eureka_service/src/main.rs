use crate::app_state::AppState;
use axum::routing::{get, post};
use axum::Router;
use shared_global::db::postgres::create_pool;

mod app_state;
mod bootstrap;
mod dtos;
mod errors;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let service_host =
        std::env::var("SERVICE_HOST").unwrap_or_else(|_| "docker".to_string());

    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres pool");

    sqlx::migrate!().run(&pool).await?;

    let config = bootstrap::bootstrap_config(&pool)
        .await
        .expect("Failed to bootstrap shared config");

    tracing::info!(
        "Eureka service bootstrapped - public_key len={}, api_key len={}",
        config.jwt_public_key.len(),
        config.internal_api_key.len()
    );

    let app_state = AppState {
        pool,
        jwt_private_key: config.jwt_private_key,
        jwt_public_key: config.jwt_public_key,
        internal_api_key: config.internal_api_key,
        service_host,
    };

    let app = Router::new()
        .route("/config/:service_name", get(routes::config::get_config))
        .route("/register", post(routes::register::register_service))
        .route(
            "/discover/:service_name",
            get(routes::discover::discover_service),
        )
        .route("/health", get(routes::health::health))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005")
        .await
        .expect("Failed to bind to port 3005");

    tracing::info!(
        "Eureka service listening on {}",
        listener.local_addr()?
    );

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
