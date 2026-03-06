use crate::entities::file::FileEntity;
use crate::models::file::File;
use sqlx::{query, query_as, PgPool};

pub async fn create_file(
    pool: &PgPool,
    user_id: i64,
    filename: &str,
    object_key: &str,
    is_carrier: bool,
    is_steg_object: bool,
) -> Result<File, sqlx::Error> {
    let result = query!(
        r#"
        INSERT INTO files (user_id, filename, object_key, is_carrier, is_steg_object)
        VALUES($1,$2,$3,$4,$5) RETURNING id
        "#,
        user_id,
        filename,
        object_key,
        is_carrier,
        is_steg_object,
    )
    .fetch_one(pool)
    .await?
    .id;
    let file = get_file_by_id(pool, result)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(file)
}

pub async fn get_file_by_id(pool: &PgPool, id: i64) -> Result<Option<File>, sqlx::Error> {
    let result = query_as!(
        FileEntity,
        r#"SELECT id, user_id, filename, object_key, created_at, is_carrier, is_steg_object
        FROM files
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .map(|db| db.into());
    Ok(result)
}

pub async fn list_file_by_user_id(pool: &PgPool, id: i64) -> Result<Vec<File>, sqlx::Error> {
    let result: Vec<FileEntity> = query_as!(
        FileEntity,
        r#"
        SELECT id, user_id, filename, object_key, created_at, is_carrier, is_steg_object
        FROM files
        WHERE user_id = $1
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;

    Ok(result.into_iter().map(|i| i.into()).collect())
}

pub async fn delete_file(pool: &PgPool, file_id: i64) -> Result<bool, sqlx::Error> {
    let result = query!(
        r#"
        DELETE FROM files WHERE id = $1
        "#,
        file_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
pub async fn update_file_name(
    pool: &PgPool,
    file_id: i64,
    new_name: &str,
) -> Result<bool, sqlx::Error> {
    let result = query!(
        r#"
        UPDATE files
        SET filename = $1
        WHERE id = $2
        "#,
        new_name,
        file_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}
