mod app_state;
pub mod errors;
mod routes;
use crate::app_state::AppState;
use axum::Router;
mod services;
use axum::routing::post;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let eureka_url = std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "steganography_service")
        .await
        .expect("Failed to fetch config from eureka");

    tracing::info!("Loaded config from eureka");

    // Register self with eureka
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://steganography_service:3003".to_string());
    shared_global::eureka::register_service(&eureka_url, "steganography_service", &self_url)
        .await
        .expect("Failed to register with eureka");

    // Create app state
    let app_state = AppState {};

    // Build router
    let app = Router::new()
        .route("/embed/image", post(routes::embed_image::embed_image))
        // .route("/auth/register", post(routes::auth::register))
        // .route("/auth/login", post(routes::auth::login))
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
                tracing::info!("Heartbeat sent");
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
