use crate::common::permission::PermissionChecker;
use crate::file::local_repository::FileRepository;
use crate::user::domain::User;
use std::sync::Arc;

pub struct DirExistsUseCase {
    file_repo: Arc<FileRepository>,
}

impl DirExistsUseCase {
    pub fn new(file_repo: Arc<FileRepository>) -> Self {
        Self { file_repo }
    }

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        dirname: &str,
    ) -> bool {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);

        if !PermissionChecker::is_safe_path(&resolved) {
            return false;
        }

        if resolved == "shared" || resolved.starts_with("shared/") {
            // Check if it's a valid shared path
            // (Simplified: just return false for now as shared exists check is complex)
            return false;
        }

        self.file_repo.dir_exists(&user.username, &resolved).await
    }
}
