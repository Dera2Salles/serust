use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::{CreateFileUseCase, FindFileByPathUseCase, UpdateFileUseCase};

use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct MkdirUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    create_db_file: Arc<CreateFileUseCase>,
    find_db_file: Arc<FindFileByPathUseCase>,
    update_db_file: Arc<UpdateFileUseCase>,
    git_service: Arc<GitService>,
}

impl MkdirUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        create_db_file: Arc<CreateFileUseCase>,
        find_db_file: Arc<FindFileByPathUseCase>,
        update_db_file: Arc<UpdateFileUseCase>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            shares,
            create_db_file,
            find_db_file,
            update_db_file,
            git_service,
        }
    }

    async fn ensure_db_parents(&self, user: &User, path: &str) -> Result<(), DomainError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = String::new();

        for segment in segments {
            current_path.push_str("/");
            current_path.push_str(segment);

            let existing = self
                .find_db_file
                .execute(user.id, &current_path)
                .await
                .ok()
                .flatten();

            match existing {
                None => {
                    let db_entry = DbFileMetadata {
                        id: uuid::Uuid::new_v4(),
                        owner_id: user.id,
                        filename: segment.to_string(),
                        storage_path: current_path.clone(),
                        size_bytes: 0,
                        mime_type: Some("inode/directory".into()),
                        checksum: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        is_deleted: false,
                    };
                    let _ = self.create_db_file.execute(&db_entry).await;
                }
                Some(ref e) if e.is_deleted => {
                    // Reactivate the soft-deleted directory record
                    let mut reactivated = e.clone();
                    reactivated.is_deleted = false;
                    reactivated.updated_at = chrono::Utc::now();
                    let _ = self.update_db_file.execute(&reactivated).await;
                }
                _ => {} // already exists and not deleted, nothing to do
            }
        }
        Ok(())
    }

    pub async fn execute(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if resolved == "shared" {
            return Err(DomainError::PermissionDenied);
        }

        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;
            let res = self.file_repo.create_dir(&owner, &inner).await;
            if res.is_ok() {
                let user_path = self.storage_root.join(&owner);
                let _ = self.git_service.commit_file(
                    &user_path,
                    &inner,
                    &format!("Created folder (shared): {}", inner),
                );
            }
            return res;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        if self.file_repo.dir_exists(&user.username, &resolved).await {
            return Err(DomainError::AlreadyExists);
        }

        self.file_repo.create_dir(&user.username, &resolved).await?;

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(
            &user_path,
            &resolved,
            &format!("Created folder: {}", dirname),
        );

        self.ensure_db_parents(user, &resolved).await?;

        Ok(())
    }
}
