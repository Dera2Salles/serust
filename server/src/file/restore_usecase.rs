use crate::common::error::DomainError;
use crate::database::file_usecases::{FindFileUseCase, RestoreFileDbUseCase};
use crate::user::domain::User;
use std::sync::Arc;

pub struct RestoreUseCase {
    find_db_file: Arc<FindFileUseCase>,
    restore_db_file: Arc<RestoreFileDbUseCase>,
}

impl RestoreUseCase {
    pub fn new(
        find_db_file: Arc<FindFileUseCase>,
        restore_db_file: Arc<RestoreFileDbUseCase>,
    ) -> Self {
        Self {
            find_db_file,
            restore_db_file,
        }
    }

    pub async fn execute(&self, user: &User, id: uuid::Uuid) -> Result<(), DomainError> {
        let db_meta = self
            .find_db_file
            .execute(id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .ok_or(DomainError::FileNotFound)?;

        if db_meta.owner_id != user.id {
            return Err(DomainError::PermissionDenied);
        }

        if !db_meta.is_deleted {
            // Only files already in the trash can be restored
            return Err(DomainError::PermissionDenied);
        }

        self.restore_db_file
            .execute(id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}
