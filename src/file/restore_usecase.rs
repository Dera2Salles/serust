use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::database::file_usecases::{FindFileByPathUseCase, RestoreFileDbUseCase};
use crate::file::git_service::GitService;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RestoreUseCase {
    storage_root: PathBuf,
    find_db_file: Arc<FindFileByPathUseCase>,
    restore_db_file: Arc<RestoreFileDbUseCase>,
    git_service: Arc<GitService>,
}

impl RestoreUseCase {
    pub fn new(
        storage_root: PathBuf,
        find_db_file: Arc<FindFileByPathUseCase>,
        restore_db_file: Arc<RestoreFileDbUseCase>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            find_db_file,
            restore_db_file,
            git_service,
        }
    }

    pub async fn execute(
        &self,
        user: &User,
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
            
            let user_path = self.storage_root.join(&user.username);
            let _ = self.git_service.commit_file(&user_path, &resolved, &format!("Restored file from recycle bin: {}", filename));

            Ok(())
        } else {
            Err(DomainError::FileNotFound)
        }
    }
}
