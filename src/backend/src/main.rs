mod models;
mod steganography;

use axum::{Router, routing::get};

// const CARRIER: &str = "../../examples/videos_lossless/chicken_jockey.mkv";
// const PAYLOAD: &str = "../../examples/hideable_files/smol-hornet.png";
// const STEG_FILE: &str = "../../output/steg_files/output.mkv";
// const EXTRACTED_PAYLOAD: &str = "../../output/payloads/output";
const PORT: u32 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
