use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub user_service_url: String,
}

impl shared_global::auth::jwt::HasJwtPublicKey for AppState {
    fn jwt_public_key(&self) -> String {
        self.jwt_public_key.to_string()
    }
}
