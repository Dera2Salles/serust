use crate::database::domain::DbFileMetadata;
use crate::database::interfaces::IFileDatabaseRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct FileRepository {
    db: Database,
}

impl FileRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IFileDatabaseRepository for FileRepository {
    async fn create(&self, file: &DbFileMetadata) -> Result<()> {
        let id_str = file.id.to_string();
        let owner_str = file.owner_id.to_string();

        sqlx::query(
            "INSERT INTO files (id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&owner_str)
        .bind(&file.filename)
        .bind(&file.storage_path)
        .bind(file.size_bytes)
        .bind(&file.mime_type)
        .bind(&file.checksum)
        .bind(file.created_at)
        .bind(file.updated_at)
        .bind(file.is_deleted)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbFileMetadata>> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted FROM files WHERE id = ?"
        )
        .bind(&id_str)
        .fetch_optional(&*self.db.pool)
        .await?;

        Self::row_to_metadata(row)
    }

    async fn find_by_storage_path(&self, path: &str) -> Result<Option<DbFileMetadata>> {
        let row = sqlx::query(
            "SELECT id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted FROM files WHERE storage_path = ? AND is_deleted = 0"
        )
        .bind(path)
        .fetch_optional(&*self.db.pool)
        .await?;

        Self::row_to_metadata(row)
    }

    async fn update(&self, file: &DbFileMetadata) -> Result<()> {
        let id_str = file.id.to_string();

        // 1. Save current state to file_versions
        let current = self.find_by_id(file.id).await?;
        if let Some(old) = current {
            let version_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO file_versions (id, file_id, storage_path, size_bytes, checksum, created_at) VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(&version_id)
            .bind(&id_str)
            .bind(&old.storage_path)
            .bind(old.size_bytes)
            .bind(&old.checksum)
            .bind(old.updated_at)
            .execute(&*self.db.pool)
            .await?;
        }

        // 2. Update files table
        sqlx::query(
            "UPDATE files SET size_bytes = ?, checksum = ?, updated_at = ? WHERE id = ?"
        )
        .bind(file.size_bytes)
        .bind(&file.checksum)
        .bind(file.updated_at)
        .bind(&id_str)
        .execute(&*self.db.pool)
        .await?;

        Ok(())
    }
}

impl FileRepository {
    fn row_to_metadata(row: Option<sqlx::sqlite::SqliteRow>) -> Result<Option<DbFileMetadata>> {
        use sqlx::Row;
        if let Some(r) = row {
            let id_str: String = r.try_get("id")?;
            let owner_str: String = r.try_get("owner_id")?;
            Ok(Some(DbFileMetadata {
                id: Uuid::parse_str(&id_str)?,
                owner_id: Uuid::parse_str(&owner_str)?,
                filename: r.try_get("filename")?,
                storage_path: r.try_get("storage_path")?,
                size_bytes: r.try_get("size_bytes")?,
                mime_type: r.try_get("mime_type")?,
                checksum: r.try_get("checksum")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
                is_deleted: r.try_get("is_deleted")?,
            }))
        } else {
            Ok(None)
        }
    }
}
