use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::file::local_repository::FileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct StatUseCase {
    file_repo: Arc<FileRepository>,
    shares: Arc<ShareService>,
}

impl StatUseCase {
    pub fn new(file_repo: Arc<FileRepository>, shares: Arc<ShareService>) -> Self {
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

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        target: &str,
    ) -> Result<Option<(u64, bool)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, target);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Ok(None);
            }
            return self.file_repo.stat(&owner, &inner).await;
        }

        self.file_repo.stat(&user.username, &resolved).await
    }
}
