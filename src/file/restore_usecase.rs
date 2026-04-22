use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::database::file_usecases::{FindFileByPathUseCase, RestoreFileDbUseCase};
use crate::user::domain::User;
use std::sync::Arc;

pub struct RestoreUseCase {
    find_db_file: Arc<FindFileByPathUseCase>,
    restore_db_file: Arc<RestoreFileDbUseCase>,
}

impl RestoreUseCase {
    pub fn new(
        find_db_file: Arc<FindFileByPathUseCase>,
        restore_db_file: Arc<RestoreFileDbUseCase>,
    ) -> Self {
        Self {
            find_db_file,
            restore_db_file,
        }
    }

    pub async fn execute(
        &self,
        _user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        let storage_path = format!("/{}", resolved);
        if let Ok(Some(db_meta)) = self.find_db_file.execute(&storage_path).await {
            self.restore_db_file
                .execute(db_meta.id)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            Ok(())
        } else {
            Err(DomainError::FileNotFound)
        }
    }
}
