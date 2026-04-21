use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::file_usecases::{FindFileByPathUseCase, SoftDeleteFileDbUseCase};
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct DeleteUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    find_db_file: Arc<FindFileByPathUseCase>,
    soft_delete_db_file: Arc<SoftDeleteFileDbUseCase>,
}

impl DeleteUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        find_db_file: Arc<FindFileByPathUseCase>,
        soft_delete_db_file: Arc<SoftDeleteFileDbUseCase>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            find_db_file,
            soft_delete_db_file,
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

    pub async fn execute(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;
            // Soft delete shared files? Usually we only allow owner to manage recycle bin.
            // For shared files, we'll keep hard delete for now or implement "shared recycle bin" later.
            return self.file_repo.delete_file(&owner, &inner).await;
        }

        if let Some(existing) = self
            .file_repo
            .get_metadata(&user.username, &resolved)
            .await
        {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        } else {
             return Err(DomainError::FileNotFound);
        }

        let storage_path = format!("/{}", resolved);
        if let Ok(Some(db_meta)) = self.find_db_file.execute(&storage_path).await {
            println!("DEBUG: Found file in DB for soft delete: {}", storage_path);
            self.soft_delete_db_file.execute(db_meta.id).await.map_err(|e| DomainError::Internal(e.to_string()))?;
            Ok(())
        } else {
            println!("DEBUG: File not found in DB for soft delete, falling back to hard delete: {}", storage_path);
            // Fallback to hard delete if not in DB (should not happen with new sync logic)
            self.file_repo.delete_file(&user.username, &resolved).await
        }
    }
}
