use crate::entities::file::FileEntity;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct File {
    id: i64,
    user_id: i64,
    filename: String,
    object_key: String,
    created_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl File {
    //getters
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn object_key(&self) -> &str {
        &self.object_key
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

//auto converts from database entity to domain model
impl From<FileEntity> for File {
    fn from(entity: FileEntity) -> Self {
        File {
            id: entity.id,
            user_id: entity.user_id,
            filename: entity.filename,
            object_key: entity.object_key,
            created_at: entity.created_at,
        }
    }
}
