#[derive(Debug)]
pub enum UserServiceError {
    DatabaseError(sqlx::Error),
    Unauthorized,
    NotFound,
}
