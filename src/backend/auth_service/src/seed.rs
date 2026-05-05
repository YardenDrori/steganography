use crate::repositories::user_repository;
use shared_global::auth::roles::Role;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};

pub async fn ensure_admin_role(pool: PgPool, user_service_url: String) {
    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    match any_admin_role_exists(&pool).await {
        Ok(true) => {
            tracing::info!("Admin role already exists, skipping seed");
            return;
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!(error = ?e, "Failed to check admin role existence, skipping seed");
            return;
        }
    }

    let client = reqwest::Client::new();
    let verify_url = format!("{}/internal/auth/verify-credentials", user_service_url);
    let body = serde_json::json!({ "user_name": username, "password": password });

    let mut admin_user_id: Option<i64> = None;
    for attempt in 1u8..=10 {
        match client.post(&verify_url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(val) => {
                        admin_user_id = val["id"].as_i64();
                        break;
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to parse user_service response");
                        break;
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!(
                    attempt,
                    status = %resp.status(),
                    "Admin seed: verify-credentials returned error, admin user may not exist yet"
                );
                // Not a connection error — user_service is up but admin user missing; retry
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::warn!(attempt, error = ?e, "Admin seed: user_service not reachable, retrying");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }

    let Some(user_id) = admin_user_id else {
        tracing::warn!("Admin seed: could not get admin user_id from user_service after retries");
        return;
    };

    match user_repository::add_user_role(&pool, user_id, Role::Admin).await {
        Ok(_) => tracing::info!(user_id, "Admin role assigned successfully"),
        Err(e) => tracing::error!(error = ?e, "Failed to assign admin role"),
    }
}

async fn any_admin_role_exists(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!("SELECT user_id FROM user_roles WHERE role = 'admin' LIMIT 1")
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}
