use crate::repositories::user_repository;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use sqlx::PgPool;

pub async fn ensure_admin(pool: &PgPool) {
    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());
    let email = std::env::var("ADMIN_EMAIL")
        .unwrap_or_else(|_| format!("{}@localhost", username));

    match user_repository::get_user_by_email_or_username(pool, None, Some(&username)).await {
        Ok(Some(_)) => {
            tracing::info!("Default admin user '{}' already exists", username);
            return;
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!(error = ?e, "Failed to check for admin user, skipping seed");
            return;
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => {
            tracing::error!(error = ?e, "Failed to hash admin password, skipping seed");
            return;
        }
    };

    match user_repository::create_user(
        pool, &username, "Admin", "User", None, &email, None, &hash,
    )
    .await
    {
        Ok(user) => tracing::info!(
            user_id = user.id(),
            username = %username,
            "Created default admin user"
        ),
        Err(e) => tracing::error!(error = ?e, "Failed to create default admin user"),
    }
}
