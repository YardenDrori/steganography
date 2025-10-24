use crate::models::user::User;
use sqlx::MySqlPool;

pub async fn create_user(
    pool: &MySqlPool,
    user_name: String,
    first_name: String,
    last_name: String,
    email: String,
    password_hash: String,
    phone_number: Option<String>,
    is_male: Option<bool>,
) -> Result<User, sqlx::Error> {
    // Step 1: Insert the user
    let result = sqlx::query!(
        r#"
        INSERT INTO users (user_name, first_name, last_name, email, password_hash, phone_number, is_male)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user_name,
        first_name,
        last_name,
        email,
        password_hash,
        phone_number,
        is_male
    )
    .execute(pool)
    .await?;

    // Step 2: Get the ID of the inserted user
    let user_id = result.last_insert_id() as i64;

    // Step 3: Fetch the complete user record
    let user = sqlx::query_as!(
        User,
        r#"
    SELECT id, user_name, first_name, last_name, 
           is_male as "is_male: bool",
           email, phone_number, 
           password_hash, created_at, updated_at, 
           is_active as "is_active: bool",
           is_verified as "is_verified: bool"
    FROM users
    WHERE id = ?
    "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_username(
    pool: &MySqlPool,
    user_name: String,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
    SELECT id, user_name, first_name, last_name, 
           is_male as "is_male: bool",
           email, phone_number, 
           password_hash, created_at, updated_at, 
           is_active as "is_active: bool",
           is_verified as "is_verified: bool"
    FROM users
    WHERE user_name = ?
    "#,
        user_name
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
