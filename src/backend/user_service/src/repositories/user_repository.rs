use crate::entities::user::UserEntity;
use crate::models::user::User;
use sqlx::{query, query_as, PgPool};

pub async fn get_user_by_id(pool: &PgPool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
    let result = query_as!(
        UserEntity,
        r#"
        SELECT id, user_name, first_name,
        last_name, is_male,
        email, phone_number, password_hash,
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

pub async fn create_user(
    pool: &PgPool,
    user_name: &str,
    first_name: &str,
    last_name: &str,
    is_male: Option<bool>,
    email: &str,
    phone_number: Option<&str>,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let result = query!(
        r#"
        INSERT INTO users (user_name, first_name,
        last_name, is_male,
        email, phone_number, password_hash)
        VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING id
        "#,
        user_name,
        first_name,
        last_name,
        is_male,
        email,
        phone_number,
        password_hash,
    )
    .fetch_one(pool)
    .await?;

    let user = get_user_by_id(pool, result.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(user)
}

pub async fn update_user(
    pool: &PgPool,
    user_id: i64,
    first_name: Option<&str>,
    last_name: Option<&str>,
    email: Option<&str>,
    phone_number: Option<&str>,
    is_male: Option<bool>,
) -> Result<User, sqlx::Error> {
    // Build dynamic UPDATE query only for provided fields
    query!(
        r#"
        UPDATE users
        SET
            first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            email = COALESCE($4, email),
            phone_number = COALESCE($5, phone_number),
            is_male = COALESCE($6, is_male),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
        user_id,
        first_name,
        last_name,
        email,
        phone_number,
        is_male
    )
    .execute(pool)
    .await?;

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    Ok(user)
}

pub async fn delete_user(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let result = query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn set_user_active_status(
    pool: &PgPool,
    user_id: i64,
    is_active: bool,
) -> Result<User, sqlx::Error> {
    query!(
        r#"
        UPDATE users
        SET is_active = $2, updated_at = CURRENT_TIMESTAMP
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

pub async fn get_user_by_email_or_username(
    pool: &PgPool,
    email: Option<&str>,
    user_name: Option<&str>,
) -> Result<Option<User>, sqlx::Error> {
    let result = query_as!(
        UserEntity,
        r#"
        SELECT id, user_name, first_name,
        last_name, is_male,
        email, phone_number, password_hash,
        created_at as "created_at: _",
        updated_at as "updated_at: _",
        is_active, is_verified
        FROM users
        WHERE email = $1 OR user_name = $2
        "#,
        email,
        user_name
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());
    Ok(result)
}
