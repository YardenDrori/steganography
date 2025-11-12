use sqlx::PgPool;

pub trait HasPgPool {
    fn pool(&self) -> &PgPool;
}
