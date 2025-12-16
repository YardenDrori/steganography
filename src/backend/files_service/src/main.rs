use crate::app_state::AppState;
use axum::routing::post;
use axum::Router;
use shared_global::db::postgres::create_pool;
mod app_state;
mod dtos;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let auth_service_url =
        std::env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());

    let user_service_url =
        std::env::var("USER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());

    let internal_api_key =
        std::env::var("INTERNAL_API_KEY").expect("INTERNAL_API_KEY must be set in env");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in env");

    // Fetch JWT public key from auth service
    tracing::info!(
        "Fetching JWT public key from auth service at {}",
        auth_service_url
    );
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/public-key", auth_service_url))
        .send()
        .await
        .expect("Failed to fetch public key from auth service");

    #[derive(serde::Deserialize)]
    struct PublicKeyResponse {
        public_key: String,
    }

    let public_key_response: PublicKeyResponse = response
        .json()
        .await
        .expect("Failed to parse public key response");

    // JSON serialization escapes newlines, so we need to convert them back
    let jwt_public_key = public_key_response.public_key.replace(r"\n", "\n");

    tracing::info!("Done! recieved public jwt key {} (len={})", jwt_public_key, jwt_public_key.len());
    tracing::info!("Raw public key from JSON: {} (len={})", public_key_response.public_key, public_key_response.public_key.len());

    // Create database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run database migrations
    sqlx::migrate!().run(&pool).await?;

    // Create app state
    let app_state = AppState {
        pool: pool,
        jwt_public_key: jwt_public_key,
        internal_api_key: internal_api_key,
        user_service_url: user_service_url,
        auth_service_url: auth_service_url,
    };

    // Build router
    let app = Router::new()
        .route("/files", post(routes::post_files::post_files))
        // .route("/internal/users/:id/status", patch(sync::sync_user_status))
        // .route("/internal/auth/verify-credentials", post(auth::verify_credentials))
        .with_state(app_state);

    // Start server on port 3004
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004")
        .await
        .expect("Failed to bind to port 3004");

    tracing::info!("Files service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
