use crate::entities::user::UserEntity;
use crate::models::user::User;
use shared_global::auth::roles::{Role, Roles, ToStr};
use sqlx::PgPool;
use std::str::FromStr;

pub async fn save_user(
    pool: &PgPool,
    user_name: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO users (user_name,   email, password_hash )
        VALUES ($1, $2, $3) RETURNING id
        "#,
        user_name,
        email,
        password_hash,
    )
    .fetch_one(pool)
    .await?;
    let user_id = result.id; //get the inserted user's ID

    let user: User = get_user_by_id(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(user)
}

pub async fn get_user_by_username(
    pool: &PgPool,
    user_name: &str,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
    SELECT id, user_name, email, password_hash, is_active, is_verified
    FROM users
    WHERE user_name = $1
    "#,
        user_name
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
    SELECT id, user_name,
           email,
           password_hash,
           is_active ,
           is_verified
    FROM users
    WHERE email= $1
    "#,
        email
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());

    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        UserEntity,
        r#"
          SELECT id, user_name,
                 email,
                 password_hash,
                 is_active as "is_active: bool",
                 is_verified as "is_verified: bool"
          FROM users
          WHERE id = $1
          "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());

    Ok(user)
}

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

pub async fn set_user_active_status(
    pool: &PgPool,
    user_id: i64,
    is_active: bool,
) -> Result<User, sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE users
        SET is_active = $2
        WHERE id = $1
        "#,
        user_id,
        is_active
    )
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(user)
}
