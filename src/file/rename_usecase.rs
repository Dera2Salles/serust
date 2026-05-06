use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::file_usecases::{FindFileByPathUseCase, RenameFileDbUseCase};
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RenameUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    find_db_file: Arc<FindFileByPathUseCase>,
    rename_db_file: Arc<RenameFileDbUseCase>,
    git_service: Arc<GitService>,
}

impl RenameUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        find_db_file: Arc<FindFileByPathUseCase>,
        rename_db_file: Arc<RenameFileDbUseCase>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            find_db_file,
            rename_db_file,
            git_service,
        }
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

        if !PermissionChecker::is_safe_path(&old_resolved)
            || !PermissionChecker::is_safe_path(&new_resolved)
        {
            return Err(DomainError::UnsafePath);
        }

        if old_resolved.starts_with("shared/") || new_resolved.starts_with("shared/") {
            return Err(DomainError::PermissionDenied);
        }

        if let Some(existing) = self
            .file_repo
            .get_metadata(&user.username, &old_resolved)
            .await
        {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else {
            return Err(DomainError::FileNotFound);
        }

        self.file_repo
            .rename(&user.username, &old_resolved, &new_resolved)
            .await?;

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(&user_path, &old_resolved, &format!("Renamed file (old): {}", old_name));
        let _ = self.git_service.commit_file(&user_path, &new_resolved, &format!("Renamed file (new): {}", new_name));

        let old_storage_path = format!("/{}", old_resolved);
        if let Ok(Some(db_meta)) = self.find_db_file.execute(&old_storage_path).await {
            let new_filename = new_resolved
                .split('/')
                .last()
                .unwrap_or(new_name)
                .to_string();
            let new_storage_path = format!("/{}", new_resolved);

            let _ = self
                .rename_db_file
                .execute(db_meta.id, &new_storage_path, &new_filename)
                .await;
        }

        Ok(())
    }
}
