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

    async fn find_by_storage_path(&self, owner_id: Uuid, path: &str) -> Result<Option<DbFileMetadata>> {
        let owner_str = owner_id.to_string();
        let row = sqlx::query(
            "SELECT id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted FROM files WHERE owner_id = ? AND storage_path = ?"
        )
        .bind(&owner_str)
        .bind(path)
        .fetch_optional(&*self.db.pool)
        .await?;

        Self::row_to_metadata(row)
    }

    async fn update(&self, file: &DbFileMetadata) -> Result<()> {
        let id_str = file.id.to_string();

        sqlx::query("UPDATE files SET size_bytes = ?, checksum = ?, updated_at = ? WHERE id = ?")
            .bind(file.size_bytes)
            .bind(&file.checksum)
            .bind(file.updated_at)
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;

        Ok(())
    }

    async fn rename(&self, id: Uuid, new_storage_path: &str, new_filename: &str) -> Result<()> {
        let id_str = id.to_string();
        let now = chrono::Utc::now();

        sqlx::query("UPDATE files SET storage_path = ?, filename = ?, updated_at = ? WHERE id = ?")
            .bind(new_storage_path)
            .bind(new_filename)
            .bind(now)
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;

        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("UPDATE files SET is_deleted = 1, updated_at = ? WHERE id = ?")
            .bind(chrono::Utc::now())
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }

    async fn restore(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("UPDATE files SET is_deleted = 0, updated_at = ? WHERE id = ?")
            .bind(chrono::Utc::now())
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }

    async fn find_deleted_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbFileMetadata>> {
        let owner_str = owner_id.to_string();
        let rows = sqlx::query(
            "SELECT id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted FROM files WHERE owner_id = ? AND is_deleted = 1"
        )
        .bind(&owner_str)
        .fetch_all(&*self.db.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            if let Some(meta) = Self::row_to_metadata(Some(row))? {
                results.push(meta);
            }
        }
        Ok(results)
    }

    async fn delete_permanently(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM files WHERE id = ?")
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }

    async fn find_by_parent_path(&self, owner_id: Uuid, parent_path: &str) -> Result<Vec<DbFileMetadata>> {
        let prefix = if parent_path.ends_with('/') {
            parent_path.to_string()
        } else {
            format!("{}/", parent_path)
        };
        let pattern = format!("{}%", prefix);
        let owner_str = owner_id.to_string();

        // SQL query to find direct children:
        // 1. Path must start with prefix
        // 2. There should be no more '/' after the prefix (direct child)
        // 3. Filter by owner_id
        let rows = sqlx::query(
            "SELECT id, owner_id, filename, storage_path, size_bytes, mime_type, checksum, created_at, updated_at, is_deleted 
             FROM files 
             WHERE owner_id = ?
             AND storage_path LIKE ? 
             AND instr(substr(storage_path, length(?) + 1), '/') = 0"
        )
        .bind(&owner_str)
        .bind(&pattern)
        .bind(&prefix)
        .fetch_all(&*self.db.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            if let Some(meta) = Self::row_to_metadata(Some(row))? {
                results.push(meta);
            }
        }
        Ok(results)
    }

    async fn delete_by_path_prefix(&self, owner_id: Uuid, path_prefix: &str) -> Result<()> {
        let pattern = if path_prefix.ends_with('/') {
            format!("{}%", path_prefix)
        } else {
            format!("{}/%", path_prefix)
        };
        let owner_str = owner_id.to_string();

        sqlx::query("DELETE FROM files WHERE owner_id = ? AND (storage_path = ? OR storage_path LIKE ?)")
            .bind(&owner_str)
            .bind(path_prefix)
            .bind(&pattern)
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
