use crate::common::error::DomainError;
use crate::database::file_usecases::{DeletePermanentlyDbUseCase, FindFileUseCase, DeleteByPathPrefixDbUseCase};
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::sync::Arc;

pub struct PurgeUseCase {
    file_repo: Arc<dyn IFileRepository>,
    find_db_file: Arc<FindFileUseCase>,
    delete_db_file: Arc<DeletePermanentlyDbUseCase>,
    delete_by_prefix: Arc<DeleteByPathPrefixDbUseCase>,
}

impl PurgeUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        find_db_file: Arc<FindFileUseCase>,
        delete_db_file: Arc<DeletePermanentlyDbUseCase>,
        delete_by_prefix: Arc<DeleteByPathPrefixDbUseCase>,
    ) -> Self {
        Self {
            file_repo,
            find_db_file,
            delete_db_file,
            delete_by_prefix,
        }
    }

    pub async fn execute(&self, user: &User, id: uuid::Uuid) -> Result<(), DomainError> {
        // 1. Find file in DB
        let db_meta = self.find_db_file.execute(id).await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or(DomainError::FileNotFound)?;

        // 2. Security check: must be owner
        if db_meta.owner_id != user.id {
            return Err(DomainError::PermissionDenied);
        }

        // 3. Delete from physical storage
        // storage_path in DB is "/filename" or "/path/to/file"
        let rel_path = db_meta.storage_path.trim_start_matches('/');
        let is_dir = db_meta.mime_type.as_ref().map_or(false, |m| m.contains("directory"));

        if is_dir {
            self.file_repo.remove_dir(&user.username, rel_path).await?;
            
            // 4. Delete all children from DB recursively
            self.delete_by_prefix.execute(user.id, &db_meta.storage_path).await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        } else {
            self.file_repo.delete_file(&user.username, rel_path).await?;
        }

        // 5. Delete from DB permanently (self)
        self.delete_db_file.execute(id).await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
