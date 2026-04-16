use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::sync::Arc;

pub struct RenameUseCase {
    file_repo: Arc<dyn IFileRepository>,
}

impl RenameUseCase {
    pub fn new(file_repo: Arc<dyn IFileRepository>) -> Self {
        Self { file_repo }
    }

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DomainError> {
        let old_resolved = PermissionChecker::resolve_path(cwd, old_name);
        let new_resolved = PermissionChecker::resolve_path(cwd, new_name);

        if !PermissionChecker::is_safe_path(&old_resolved) || !PermissionChecker::is_safe_path(&new_resolved) {
            return Err(DomainError::UnsafePath);
        }

        if old_resolved.starts_with("shared/") || new_resolved.starts_with("shared/") {
            return Err(DomainError::PermissionDenied); 
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &old_resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else {
            return Err(DomainError::FileNotFound);
        }

        self.file_repo.rename(&user.username, &old_resolved, &new_resolved).await
    }
}
