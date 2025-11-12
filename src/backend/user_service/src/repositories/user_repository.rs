use crate::entities::user::UserEntity;
use crate::models::user::User;
use shared_global::auth::roles::Roles;
use sqlx::{query_as, PgPool};
use std::str::FromStr;

pub async fn get_user_by_id(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let result = query_as!(
        UserEntity,
        r#"
        SELECT id, user_name, first_name,
        last_name, is_male,
        email, phone_number,
        created_at as "created_at: _",
        updated_at as "updated_at: _",
        is_active, is_verified
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());
    Ok(result)
}
