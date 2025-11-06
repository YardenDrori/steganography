use sqlx::{Pool, Postgres};

pub trait HasJwtSecret {
    fn jwt_secret(&self) -> &str;
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub jwt_secret: String,
}

impl HasJwtSecret for AppState {
    fn jwt_secret(&self) -> &str {
        &self.jwt_secret
    }
}
