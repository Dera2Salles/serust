use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::{
    CreateFileUseCase, FindFileByPathUseCase, SoftDeleteFileDbUseCase,
};
use crate::database::user_usecases::FindUserUseCase;
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct DeleteUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    find_db_file: Arc<FindFileByPathUseCase>,
    create_db_file: Arc<CreateFileUseCase>,
    soft_delete_db_file: Arc<SoftDeleteFileDbUseCase>,
    git_service: Arc<GitService>,
    find_user: Arc<FindUserUseCase>,
}

impl DeleteUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        find_db_file: Arc<FindFileByPathUseCase>,
        create_db_file: Arc<CreateFileUseCase>,
        soft_delete_db_file: Arc<SoftDeleteFileDbUseCase>,
        git_service: Arc<GitService>,
        find_user: Arc<FindUserUseCase>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            shares,
            find_db_file,
            create_db_file,
            soft_delete_db_file,
            git_service,
            find_user,
        }
    }

    pub async fn execute(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;

            let is_dir = self
                .file_repo
                .stat(&owner, &inner)
                .await?
                .map_or(false, |(_, d)| d);

            let res = if is_dir {
                self.file_repo.remove_dir(&owner, &inner).await
            } else {
                self.file_repo.delete_file(&owner, &inner).await
            };

            if res.is_ok() {
                let user_path = self.storage_root.join(&owner);
                let _ = self.git_service.commit_delete(
                    &user_path,
                    &inner,
                    &format!(
                        "Deleted {} (shared): {}",
                        if is_dir { "folder" } else { "file" },
                        inner
                    ),
                );

                // Soft-delete the owner's DB record now that the physical file is gone
                if let Ok(Some(owner_user)) = self.find_user.execute(&owner).await {
                    let owner_storage_path = format!("/{}", inner);
                    if let Ok(Some(meta)) = self
                        .find_db_file
                        .execute(owner_user.id, &owner_storage_path)
                        .await
                    {
                        let _ = self.soft_delete_db_file.execute(meta.id).await;
                    }
                }
            }
            return res;
        }

        let stat = self.file_repo.stat(&user.username, &resolved).await?;
        let is_dir = stat.as_ref().map_or(false, |(_, d)| *d);

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else if stat.is_none() {
            let storage_path = format!("/{}", resolved);
            if self
                .find_db_file
                .execute(user.id, &storage_path)
                .await
                .map_or(true, |o| o.is_none())
            {
                return Err(DomainError::FileNotFound);
            }
        }

        let storage_path = format!("/{}", resolved);

        let db_meta = self
            .find_db_file
            .execute(user.id, &storage_path)
            .await
            .ok()
            .flatten();

        if db_meta.is_none() && stat.is_some() {
            let new_meta = DbFileMetadata {
                id: uuid::Uuid::new_v4(),
                owner_id: user.id,
                filename: resolved.split('/').last().unwrap_or(filename).to_string(),
                storage_path: storage_path.clone(),
                size_bytes: stat.as_ref().map_or(0, |(s, _)| *s) as i64,
                mime_type: Some(if is_dir {
                    "inode/directory".into()
                } else {
                    "application/octet-stream".into()
                }),
                checksum: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                is_deleted: false,
            };
            let _ = self.create_db_file.execute(&new_meta).await;
        }

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_delete(
            &user_path,
            &resolved,
            &format!(
                "Deleted {}: {}",
                if is_dir { "folder" } else { "file" },
                filename
            ),
        );

        if let Ok(Some(meta)) = self.find_db_file.execute(user.id, &storage_path).await {
            self.soft_delete_db_file
                .execute(meta.id)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            Ok(())
        } else {
            if is_dir {
                self.file_repo.remove_dir(&user.username, &resolved).await
            } else {
                self.file_repo.delete_file(&user.username, &resolved).await
            }
        }
    }
}
