use crate::app_state::AppState;
use crate::routes::post_files;
use crate::services::files_service;
use std::sync::{Arc, RwLock};
// use crate::routes::{auth, delete_users, patch_users, post_users, sync};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use s3::creds::Credentials;
use s3::{region, Bucket, Region};
// use routes::get_users;
use shared_global::db::postgres::create_pool;
use tower_http::trace::TraceLayer;
mod app_state;
mod dtos;
mod entities;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;

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

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Fetch shared config from eureka
    let config = shared_global::eureka::fetch_config(&eureka_url, "files_service")
        .await
        .expect("Failed to fetch config from eureka");

    let config = Arc::new(RwLock::new(config));

    tracing::info!("Loaded config from eureka",);

    // Register self with eureka
    let self_url =
        std::env::var("SELF_URL").unwrap_or_else(|_| "http://files_service:3004".to_string());
    shared_global::eureka::register_service(&eureka_url, "files_service", &self_url)
        .await
        .expect("Failed to register with eureka");

    // Create database connection pool
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create postgres database pool");

    // Run database migrations
    sqlx::migrate!().run(&pool).await?;

    let bucket_name = std::env::var("MINIO_BUCKET").expect("minio_bucket must be set up in env");
    let region = Region::Custom {
        region: ("yo how are you?".to_string()),
        endpoint: (std::env::var("MINIO_ENDPOINT").expect("minio_endpoint must be in env")),
    };
    let credentials = Credentials {
        access_key: Some(std::env::var("ACCESS_KEY").expect("access_key must be in env")),
        secret_key: Some(std::env::var("SECRET_KEY").expect("secret_key must be in env")),
        security_token: None,
        session_token: None,
        expiration: None,
    };
    let bucket =
        Bucket::new(&bucket_name, region, credentials).expect("failed to initialize bucket. error");

    // Create app state
    let app_state = AppState {
        pool: pool,
        eureka_config: Arc::clone(&config),
        bucket: bucket,
    };

    // Build router
    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .route(
            "/files/tempnamecauseimbadatnamingendpointsstart",
            post(post_files::initiate),
        )
        .route("/files/tmp2", post(post_files::upload_chunk))
        .route("file/tmp3", post(post_files::complete_upload))
        .with_state(app_state);

    // Spawn heartbeat task
    let eureka_url_clone = eureka_url.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) =
                shared_global::eureka::send_heartbeat(&eureka_url_clone, "files_service").await
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
            match shared_global::eureka::fetch_config(&eureka_url, "files_service").await {
                Ok(fresh_config) => {
                    let mut configs = configs_for_refresh.write().unwrap();
                    *configs = fresh_config;
                    tracing::debug!("Refreshed service URLs from eureka");
                }
                Err(e) => tracing::warn!("Failed to refresh config: {}", e),
            }
        }
    });

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
