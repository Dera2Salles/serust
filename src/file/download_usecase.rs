use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::file::local_repository::FileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct DownloadUseCase {
    file_repo: Arc<FileRepository>,
    shares: Arc<ShareService>,
}

impl DownloadUseCase {
    pub fn new(
        file_repo: Arc<FileRepository>,
        shares: Arc<ShareService>,
    ) -> Self {
        Self {
            file_repo,
            shares,
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

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<Vec<u8>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self
                .shares
                .can_download(&user.username, &owner, &inner)
                .await
            {
                return Err(DomainError::PermissionDenied);
            }
            let data = self.file_repo.load(&owner, &inner).await?;
            self.shares
                .consume_download(&user.username, &owner, &inner)
                .await?;
            return Ok(data);
        }

        let meta = self
            .file_repo
            .get_metadata(&user.username, &resolved)
            .await
            .ok_or(DomainError::FileNotFound)?;

        if !PermissionChecker::can_access(user, &meta.owner, &Permission::Read) {
            return Err(DomainError::PermissionDenied);
        }

        let data = self.file_repo.load(&user.username, &resolved).await?;

        let log_event =
            crate::log::domain::AccessLog::new_download_event(uuid::Uuid::new_v4(), None);
        tracing::debug!("Domain access log recorded: {:?}", log_event);

        Ok(data)
    }
}
