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

    let internal_api_key =
        std::env::var("INTERNAL_API_KEY").expect("INTERNAL_API_KEY must be set in env");

    // Create app state
    let app_state = AppState { internal_api_key };

    // Build router
    let app = Router::new()
        .route("/embed/image", post(routes::embed_image::embed_image))
        // .route("/auth/register", post(routes::auth::register))
        // .route("/auth/login", post(routes::auth::login))
        .with_state(app_state);

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
