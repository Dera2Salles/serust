use crate::database::domain::DbFileMetadata;
use crate::database::entities::{prelude::*, files};
use crate::database::interfaces::IFileDatabaseRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct FileRepository {
    db: Database,
}

impl FileRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn model_to_metadata(model: files::Model) -> DbFileMetadata {
        DbFileMetadata {
            id: Uuid::parse_str(&model.id).unwrap_or_else(|_| Uuid::nil()),
            owner_id: Uuid::parse_str(&model.owner_id).unwrap_or_else(|_| Uuid::nil()),
            filename: model.filename,
            storage_path: model.storage_path,
            size_bytes: model.size_bytes,
            mime_type: model.mime_type,
            checksum: model.checksum,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
            is_deleted: model.is_deleted,
        }
    }
}

#[async_trait]
impl IFileDatabaseRepository for FileRepository {
    async fn create(&self, file: &DbFileMetadata) -> Result<()> {
        let active_model = files::ActiveModel {
            id: Set(file.id.to_string()),
            owner_id: Set(file.owner_id.to_string()),
            filename: Set(file.filename.clone()),
            storage_path: Set(file.storage_path.clone()),
            size_bytes: Set(file.size_bytes),
            mime_type: Set(file.mime_type.clone()),
            checksum: Set(file.checksum.clone()),
            created_at: Set(file.created_at.into()),
            updated_at: Set(file.updated_at.into()),
            is_deleted: Set(file.is_deleted),
        };

        Files::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbFileMetadata>> {
        let model = Files::find_by_id(id.to_string())
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_metadata))
    }

    async fn find_by_storage_path(&self, owner_id: Uuid, path: &str) -> Result<Option<DbFileMetadata>> {
        let model = Files::find()
            .filter(files::Column::OwnerId.eq(owner_id.to_string()))
            .filter(files::Column::StoragePath.eq(path))
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_metadata))
    }

    async fn update(&self, file: &DbFileMetadata) -> Result<()> {
        let model = Files::find_by_id(file.id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: files::ActiveModel = model.into();
            active_model.size_bytes = Set(file.size_bytes);
            active_model.checksum = Set(file.checksum.clone());
            active_model.updated_at = Set(file.updated_at.into());
            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn rename(&self, id: Uuid, new_storage_path: &str, new_filename: &str) -> Result<()> {
        let model = Files::find_by_id(id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: files::ActiveModel = model.into();
            active_model.storage_path = Set(new_storage_path.to_string());
            active_model.filename = Set(new_filename.to_string());
            active_model.updated_at = Set(chrono::Utc::now().into());
            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn soft_delete(&self, id: Uuid) -> Result<()> {
        let model = Files::find_by_id(id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: files::ActiveModel = model.into();
            active_model.is_deleted = Set(true);
            active_model.updated_at = Set(chrono::Utc::now().into());
            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn restore(&self, id: Uuid) -> Result<()> {
        let model = Files::find_by_id(id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: files::ActiveModel = model.into();
            active_model.is_deleted = Set(false);
            active_model.updated_at = Set(chrono::Utc::now().into());
            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn find_deleted_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbFileMetadata>> {
        let models = Files::find()
            .filter(files::Column::OwnerId.eq(owner_id.to_string()))
            .filter(files::Column::IsDeleted.eq(true))
            .all(&self.db.connection)
            .await?;

        Ok(models.into_iter().map(Self::model_to_metadata).collect())
    }

    async fn delete_permanently(&self, id: Uuid) -> Result<()> {
        Files::delete_by_id(id.to_string())
            .exec(&self.db.connection)
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

        let models = files::Model::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT * FROM files 
               WHERE owner_id = $1 
               AND storage_path LIKE $2 
               AND position('/' in substring(storage_path from length($3) + 1)) = 0"#,
            vec![owner_id.to_string().into(), pattern.into(), prefix.into()],
        ))
        .all(&self.db.connection)
        .await?;

        Ok(models.into_iter().map(Self::model_to_metadata).collect())
    }

    async fn delete_by_path_prefix(&self, owner_id: Uuid, path_prefix: &str) -> Result<()> {
        let pattern = if path_prefix.ends_with('/') {
            format!("{}%", path_prefix)
        } else {
            format!("{}/%", path_prefix)
        };

        Files::delete_many()
            .filter(files::Column::OwnerId.eq(owner_id.to_string()))
            .filter(
                files::Column::StoragePath
                    .eq(path_prefix)
                    .or(files::Column::StoragePath.like(&pattern)),
            )
            .exec(&self.db.connection)
            .await?;

        Ok(())
    }

    async fn update_path_prefix(&self, owner_id: Uuid, old_prefix: &str, new_prefix: &str) -> Result<()> {
        let pattern = if old_prefix.ends_with('/') {
            format!("{}%", old_prefix)
        } else {
            format!("{}/%", old_prefix)
        };

        self.db.connection.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"UPDATE files 
               SET storage_path = $1 || substring(storage_path from length($2) + 1)
               WHERE owner_id = $3 AND storage_path LIKE $4"#,
            vec![
                new_prefix.into(),
                old_prefix.into(),
                owner_id.to_string().into(),
                pattern.into(),
            ],
        )).await?;

        Ok(())
    }
}
