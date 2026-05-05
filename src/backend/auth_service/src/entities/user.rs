#[derive(Debug, Clone)]
pub(crate) struct UserEntity {
    pub id: i64,
    pub user_name: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
}
