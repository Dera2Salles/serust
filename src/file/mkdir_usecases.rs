use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::CreateFileUseCase;

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
    git_service: Arc<GitService>,
}

impl MkdirUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        create_db_file: Arc<CreateFileUseCase>,
        git_service: Arc<GitService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            shares,
            create_db_file,
            git_service,
        }
    }

    fn parse_shared(resolved: &str) -> Option<(String, String)> {
        let rest = resolved.strip_prefix("shared/")?;
        let mut parts = rest.splitn(2, '/');
        let owner = parts.next()?.to_string();
        let inner = parts.next().unwrap_or("").to_string();
        if owner.is_empty() {
            return None;
        }
        Some((owner, inner))
    }

    pub async fn execute(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if resolved == "shared" {
            return Err(DomainError::PermissionDenied);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;
            let res = self.file_repo.create_dir(&owner, &inner).await;
            if res.is_ok() {
                let user_path = self.storage_root.join(&owner);
                let _ = self.git_service.commit_file(&user_path, &inner, &format!("Created folder (shared): {}", inner));
            }
            return res;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        self.file_repo.create_dir(&user.username, &resolved).await?;

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(&user_path, &resolved, &format!("Created folder: {}", dirname));

        let owner_id = user.id;
        let db_entry = DbFileMetadata {
            id: uuid::Uuid::new_v4(),
            owner_id,
            filename: dirname.to_string(),
            storage_path: format!("/{}", resolved),
            size_bytes: 0,
            mime_type: Some("inode/directory".into()),
            checksum: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            is_deleted: false,
        };
        let _ = self.create_db_file.execute(&db_entry).await;

        Ok(())
    }
}
