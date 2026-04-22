use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::sync::Arc;

pub struct RemoveDirUseCase {
    file_repo: Arc<dyn IFileRepository>,
}

impl RemoveDirUseCase {
    pub fn new(file_repo: Arc<dyn IFileRepository>) -> Self {
        Self { file_repo }
    }

    pub async fn execute(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if resolved == "shared" || resolved.starts_with("shared/") {
            return Err(DomainError::PermissionDenied);
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else {
            return Err(DomainError::FileNotFound);
        }

        self.file_repo.remove_dir(&user.username, &resolved).await
    }
}
