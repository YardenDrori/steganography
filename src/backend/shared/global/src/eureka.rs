use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct EurekaConfig {
    pub jwt_public_key: String,
    pub jwt_private_key: Option<String>,
    pub services: HashMap<String, String>,
}

/// Fetches shared config from eureka for the given service.
/// Retries with backoff since eureka might still be starting.
pub async fn fetch_config(
    eureka_url: &str,
    service_name: &str,
) -> Result<EurekaConfig, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/config/{}", eureka_url, service_name);

    let mut last_error = None;
    for attempt in 1..=10 {
        tracing::debug!(
            "Fetching config from eureka (attempt {}/10): {}",
            attempt,
            url
        );
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                let config: EurekaConfig = response.json().await?;
                tracing::debug!("Successfully fetched config from eureka");
                return Ok(config);
            }
            Ok(response) => {
                last_error = Some(format!("Eureka returned status: {}", response.status()));
            }
            Err(e) => {
                last_error = Some(format!("Failed to connect to eureka: {}", e));
            }
        }
        if attempt < 10 {
            tracing::warn!("Eureka not ready, retrying in 2 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    Err(last_error
        .unwrap_or_else(|| "Unknown eureka error".to_string())
        .into())
}

/// Registers this service with eureka.
pub async fn register_service(
    eureka_url: &str,
    service_name: &str,
    service_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/register", eureka_url);

    let body = serde_json::json!({
        "service_name": service_name,
        "service_url": service_url,
    });

    let response = client.post(&url).json(&body).send().await?;

    if response.status().is_success() {
        tracing::info!("Successfully registered {} with eureka", service_name);
        Ok(())
    } else {
        Err(format!("Failed to register with eureka: {}", response.status()).into())
    }
}
/// sends heartbeat to eureka
pub async fn send_heartbeat(
    eureka_url: &str,
    service_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/heartbeat", eureka_url);

    let body = serde_json::json!({ "service_name": service_name });

    let response = client.post(&url).json(&body).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Heartbeat failed: {}", response.status()).into())
    }
}
