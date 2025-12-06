use shared_global::auth::roles::{Role, Roles, ToStr};
use sqlx::PgPool;
use std::str::FromStr;

pub async fn get_user_roles(pool: &PgPool, user_id: i64) -> Result<Roles, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT role
        FROM user_roles
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let roles: Roles = rows
        .into_iter()
        .filter_map(|row| Role::from_str(&row.role).ok())
        .collect();

    Ok(roles)
}

pub async fn add_user_role(pool: &PgPool, user_id: i64, role: Role) -> Result<(), sqlx::Error> {
    let role_str = role.to_str();

    sqlx::query!(
        r#"
        INSERT INTO user_roles (user_id, role)
        VALUES ($1, $2)
        ON CONFLICT (user_id, role) DO NOTHING
        "#,
        user_id,
        role_str
    )
    .execute(pool)
    .await?;

    Ok(())
}
