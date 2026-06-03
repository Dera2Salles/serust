use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::{CreateFileUseCase, FindFileByPathUseCase, RenameFileDbUseCase, UpdatePathPrefixDbUseCase};
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct RenameUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    find_db_file: Arc<FindFileByPathUseCase>,
    create_db_file: Arc<CreateFileUseCase>,
    rename_db_file: Arc<RenameFileDbUseCase>,
    update_path_prefix: Arc<UpdatePathPrefixDbUseCase>,
    git_service: Arc<GitService>,
}

impl RenameUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        find_db_file: Arc<FindFileByPathUseCase>,
        create_db_file: Arc<CreateFileUseCase>,
        rename_db_file: Arc<RenameFileDbUseCase>,
        update_path_prefix: Arc<UpdatePathPrefixDbUseCase>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            find_db_file,
            create_db_file,
            rename_db_file,
            update_path_prefix,
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

        // 1. Determine if target is a directory
        let stat = self.file_repo.stat(&user.username, &old_resolved).await?;
        let is_dir = stat.as_ref().map_or(false, |(_, d)| *d);

        if let Some(existing) = self
            .file_repo
            .get_metadata(&user.username, &old_resolved)
            .await
        {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else if stat.is_none() {
            return Err(DomainError::FileNotFound);
        }

        // 2. Perform physical rename
        self.file_repo
            .rename(&user.username, &old_resolved, &new_resolved)
            .await?;

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(&user_path, &old_resolved, &format!("Renamed file (old): {}", old_name));
        let _ = self.git_service.commit_file(&user_path, &new_resolved, &format!("Renamed file (new): {}", new_name));

        // 3. Update database records
        let old_storage_path = format!("/{}", old_resolved);
        let new_storage_path = format!("/{}", new_resolved);
        let new_filename = new_resolved
            .split('/')
            .last()
            .unwrap_or(new_name)
            .to_string();

        let db_meta = self.find_db_file.execute(user.id, &old_storage_path).await.ok().flatten();

        if let Some(meta) = db_meta {
            // Update the record itself
            let _ = self
                .rename_db_file
                .execute(meta.id, &new_storage_path, &new_filename)
                .await;

            if is_dir {
                // Recursively update all children in DB
                let _ = self.update_path_prefix.execute(user.id, &old_storage_path, &new_storage_path).await;
            }
        } else if stat.is_some() {
            // Missing DB record but exists on disk, create it now
            let new_meta = DbFileMetadata {
                id: uuid::Uuid::new_v4(),
                owner_id: user.id,
                filename: new_filename,
                storage_path: new_storage_path,
                size_bytes: stat.as_ref().map_or(0, |(s, _)| *s) as i64,
                mime_type: Some(if is_dir { "inode/directory".into() } else { "application/octet-stream".into() }),
                checksum: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                is_deleted: false,
            };
            let _ = self.create_db_file.execute(&new_meta).await;
        }

        Ok(())
    }
}
