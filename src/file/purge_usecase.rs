use crate::common::error::DomainError;
use crate::database::file_usecases::{FindDeletedFilesDbUseCase, PermanentDeleteFileDbUseCase};
use crate::database::user_usecases::FindUserUseCase;
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::sync::Arc;

pub struct PurgeUseCase {
    file_repo: Arc<dyn IFileRepository>,
    find_db_user: Arc<FindUserUseCase>,
    find_deleted_files: Arc<FindDeletedFilesDbUseCase>,
    permanent_delete: Arc<PermanentDeleteFileDbUseCase>,
}

impl PurgeUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        find_db_user: Arc<FindUserUseCase>,
        find_deleted_files: Arc<FindDeletedFilesDbUseCase>,
        permanent_delete: Arc<PermanentDeleteFileDbUseCase>,
    ) -> Self {
        Self {
            file_repo,
            find_db_user,
            find_deleted_files,
            permanent_delete,
        }
    }

    pub async fn execute(&self, user: &User) -> Result<(), DomainError> {
        let db_user = self
            .find_db_user
            .execute(&user.username)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or(DomainError::InvalidCredentials)?;

        let deleted_files = self
            .find_deleted_files
            .execute(db_user.id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        for file in deleted_files {
            let rel_path = file
                .storage_path
                .strip_prefix('/')
                .unwrap_or(&file.storage_path);
            let _ = self.file_repo.delete_file(&user.username, rel_path).await;

            let _ = self.permanent_delete.execute(file.id).await;
        }

        Ok(())
    }
}
