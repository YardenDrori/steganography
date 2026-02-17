use crate::app_state::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
};
use std::sync::Arc;

/// Forwards a request to a backend service
pub async fn proxy_request(service_url: &str, req: Request) -> Result<Response, StatusCode> {
    // Extract everything BEFORE consuming the request
    let method = req.method().clone();
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let headers = req.headers().clone();

    // Strip the /api prefix before forwarding
    let stripped_path = path_and_query
        .strip_prefix("/api")
        .unwrap_or(path_and_query);

    // Build the full URL to the backend service
    let url = format!("{}{}", service_url, stripped_path);

    tracing::info!("Proxying {} to {}", path_and_query, url);

    // NOW consume the request to get the body
    let body = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Make the HTTP client
    let client = reqwest::Client::new();

    // Forward the request using extracted values
    let backend_response = client
        .request(method, &url)
        .headers(headers)
        .body(body.to_vec())
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    // Build response to send back to frontend
    let mut response = Response::builder().status(backend_response.status());

    // Copy headers from backend response
    for (key, value) in backend_response.headers() {
        response = response.header(key, value);
    }

    // Get body from backend
    let body_bytes = backend_response
        .bytes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    response
        .body(Body::from(body_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for /api/auth/*
pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let configs = state.eureka_configs.read().unwrap();
    let auth_url = configs
        .services
        .get("auth_service")
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    proxy_request(auth_url, req).await
}

/// Handler for /api/user/*
pub async fn user_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let configs = state.eureka_configs.read().unwrap();
    let user_url = configs
        .services
        .get("user_service")
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    proxy_request(user_url, req).await
}

/// Handler for /api/files/*
pub async fn files_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let configs = state.eureka_configs.read().unwrap();
    let files_url = configs
        .services
        .get("files_service")
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    proxy_request(files_url, req).await
}

/// Handler for /api/embed/*
pub async fn embed_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let configs = state.eureka_configs.read().unwrap();
    let steg_url = configs
        .services
        .get("steganography_service")
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    proxy_request(steg_url, req).await
}
