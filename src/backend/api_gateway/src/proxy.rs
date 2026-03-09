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

    // Make the HTTP client
    let client = reqwest::Client::new();

    // Forward the request — stream the body directly without buffering
    let backend_response = client
        .request(method, &url)
        .headers(headers)
        .body(reqwest::Body::wrap_stream(
            req.into_body().into_data_stream(),
        ))
        .send()
        .await
        .map_err(|e| {
            tracing::error!(url = %url, error = %e, "Failed to reach backend service");
            StatusCode::BAD_GATEWAY
        })?;

    // Build response to send back to frontend
    let mut response = Response::builder().status(backend_response.status());

    // Copy headers from backend response
    for (key, value) in backend_response.headers() {
        response = response.header(key, value);
    }

    response
        .body(Body::from_stream(backend_response.bytes_stream()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for /api/auth/*
pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let auth_url = state
        .eureka_configs
        .read()
        .unwrap()
        .services
        .get("auth_service")
        .ok_or_else(|| {
            tracing::error!("auth_service not found in eureka config");
            StatusCode::SERVICE_UNAVAILABLE
        })?
        .clone();
    proxy_request(&auth_url, req).await
}

/// Handler for /api/user/*
pub async fn user_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let user_url = state
        .eureka_configs
        .read()
        .unwrap()
        .services
        .get("user_service")
        .ok_or_else(|| {
            tracing::error!("user_service not found in eureka config");
            StatusCode::SERVICE_UNAVAILABLE
        })?
        .clone();

    proxy_request(&user_url, req).await
}

/// Handler for /api/files/*
pub async fn files_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let files_url = state
        .eureka_configs
        .read()
        .unwrap()
        .services
        .get("files_service")
        .ok_or_else(|| {
            tracing::error!("files_service not found in eureka config");
            StatusCode::SERVICE_UNAVAILABLE
        })?
        .clone();

    proxy_request(&files_url, req).await
}

/// Handler for /api/embed/*
pub async fn steg_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, StatusCode> {
    let steg_url = state
        .eureka_configs
        .read()
        .unwrap()
        .services
        .get("steganography_service")
        .ok_or_else(|| {
            tracing::error!("steganography_service not found in eureka config");
            StatusCode::SERVICE_UNAVAILABLE
        })?
        .clone();

    proxy_request(&steg_url, req).await
}
