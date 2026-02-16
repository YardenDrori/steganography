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

    let eureka_url =
        std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "api_gateway")
        .await
        .expect("Failed to fetch config from eureka");


    tracing::info!("Loaded config from eureka");

    // Register self with eureka
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://api_gateway:3000".to_string());
    shared_global::eureka::register_service(&eureka_url, "api_gateway", &self_url)
        .await
        .expect("Failed to register with eureka");


    // Build router
    let app = Router::new();

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
