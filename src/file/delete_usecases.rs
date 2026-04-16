use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct DeleteUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
}

impl DeleteUseCase {
    pub fn new(file_repo: Arc<dyn IFileRepository>, shares: Arc<ShareService>) -> Self {
        Self { file_repo, shares }
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
            return self.file_repo.delete_file(&owner, &inner).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        self.file_repo.delete_file(&user.username, &resolved).await
    }
}
