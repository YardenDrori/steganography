mod auth;
mod db;
mod dtos;
mod errors;
mod models;
mod repositories;
mod routes;
mod services;
mod steganography;
use axum::{Router, routing::get, routing::post};

// const CARRIER: &str = "../../examples/videos_lossless/chicken_jockey.mkv";
// const PAYLOAD: &str = "../../examples/hideable_files/smol-hornet.png";
// const STEG_FILE: &str = "../../output/steg_files/output.mkv";
// const EXTRACTED_PAYLOAD: &str = "../../output/payloads/output";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("database url must be set in .env file");

    let pool = db::connections::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/register", post(routes::auth::register))
        .route("/login", post(routes::auth::login))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
