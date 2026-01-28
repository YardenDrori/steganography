use crate::app_state::AppState;
use axum::routing::{delete, get, post};
use axum::Router;
use minior::Minio;
use minior::aws_sdk_s3::Client as S3Client;
use shared_global::db::postgres::create_pool;
use std::sync::Arc;

mod app_state;
mod dtos;
mod errors;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let eureka_url =
        std::env::var("EUREKA_URL").expect("EUREKA_URL must be set in env");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in env");

    let minio_endpoint =
        std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set in env");

    let minio_bucket = std::env::var("MINIO_BUCKET").expect("MINIO_BUCKET must be set in env");

    // Initialize MinIO client with path-style addressing (required for MinIO)
    let aws_config = aws_config::from_env()
        .endpoint_url(&minio_endpoint)
        .load()
        .await;
    let s3_config = minior::aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .build();
    let s3_client = S3Client::from_conf(s3_config);
    let minio = Minio {
        client: Arc::new(s3_client),
    };
    if !minio.bucket_exists(&minio_bucket).await? {
        tracing::info!("Bucket '{}' not found. Creating it now.", minio_bucket);
        minio.create_bucket(&minio_bucket).await?;
    }
    tracing::info!("MinIO bucket '{}' is ready", minio_bucket);

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "files_service")
        .await
        .expect("Failed to fetch config from eureka");

    let jwt_public_key = config.jwt_public_key;
    let internal_api_key = config.internal_api_key;

    tracing::info!(
        "Loaded config from eureka - public_key len={}",
        jwt_public_key.len()
    );

    // Register self with eureka
    let self_url = std::env::var("SELF_URL")
        .unwrap_or_else(|_| "http://files_service:3004".to_string());
    shared_global::eureka::register_service(&eureka_url, "files_service", &self_url)
        .await
        .expect("Failed to register with eureka");

    // Create database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run database migrations
    sqlx::migrate!().run(&pool).await?;

    // Create app state
    let app_state = AppState {
        pool,
        jwt_public_key,
        internal_api_key,
        minio: Arc::new(minio),
        minio_bucket,
    };

    // Build router
    let app = Router::new()
        .route("/files/prepare", post(routes::prepare_upload::prepare_upload))
        .route(
            "/files/:id/confirm",
            post(routes::confirm_upload::confirm_upload),
        )
        .route("/files/:id", get(routes::get_file::get_file))
        .route("/files", get(routes::list_files::list_files))
        .route("/files/:id", delete(routes::delete_file::delete_file))
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004")
        .await
        .expect("Failed to bind to port 3004");

    tracing::info!("Files service listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}
