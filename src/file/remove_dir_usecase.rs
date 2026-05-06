use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RemoveDirUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    git_service: Arc<GitService>,
}

impl RemoveDirUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            git_service,
        }
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

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(&user_path, &resolved, &format!("Removing directory: {}", dirname));

        self.file_repo.remove_dir(&user.username, &resolved).await
    }
}
