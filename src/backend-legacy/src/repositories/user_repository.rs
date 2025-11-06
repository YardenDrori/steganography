use crate::models::user::User;
use crate::entities::user::UserEntity;
use sqlx::PgPool;

pub async fn create_user(
    pool: &PgPool,
    user_name: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    password_hash: &str,
    phone_number: &Option<&str>,
    is_male: &Option<bool>,
) -> Result<User, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO users (user_name, first_name, last_name, email, password_hash, phone_number, is_male) 
        VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
        "#,
        user_name,
        first_name,
        last_name,
        email,
        password_hash,
        *phone_number,
        *is_male
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
    SELECT id, user_name, first_name, last_name, 
           is_male,
           email, phone_number,
           password_hash, created_at as "created_at: _", updated_at as "updated_at: _", 
           is_active ,
           is_verified 
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
    SELECT id, user_name, first_name, last_name, 
           is_male ,
           email, phone_number, 
           password_hash, created_at as "created_at: _", updated_at as "updated_at: _", 
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
          SELECT id, user_name, first_name, last_name,
                 is_male as "is_male: bool",
                 email, phone_number,
                 password_hash,
                 created_at as "created_at: _",
                 updated_at as "updated_at: _",
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
