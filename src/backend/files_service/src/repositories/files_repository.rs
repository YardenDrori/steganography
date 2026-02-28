use crate::models::file::File;
use crate::{entities::file::FileEntity, errors::files_service_errors::FilesServiceError};
use sqlx::{pool, query, query_as, PgPool};

pub async fn create_file(
    pool: &PgPool,
    user_id: i64,
    filename: &str,
    object_key: &str,
) -> Result<File, sqlx::Error> {
    let result = query!(
        r#"
        INSERT INTO files (user_id, filename, object_key)
        VALUES($1,$2,$3) RETURNING id
        "#,
        user_id,
        filename,
        object_key,
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
        r#"SELECT id, user_id, filename, object_key, created_at
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
        SELECT id, user_id, filename, object_key, created_at
        FROM files
        WHERE id = $1
        "#,
        id,
    )
    .fetch_all(pool)
    .await?;

    Ok(result.into_iter().map(|i| i.into()).collect())
}

pub async fn delete_file(pool: &PgPool, file_id: i64) -> Result<(), sqlx::Error> {
    let result = query!(
        r#"
        DELETE FROM files WHERE id = $1
        "#,
        file_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
