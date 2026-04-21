use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

use crate::database::file_usecases::FindFileByPathUseCase;

pub struct StatUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    find_db_file: Arc<FindFileByPathUseCase>,
}

impl StatUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        find_db_file: Arc<FindFileByPathUseCase>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            find_db_file,
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
        target: &str,
    ) -> Result<Option<(u64, bool, Option<String>)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, target);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        let (size, is_dir) = if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Ok(None);
            }
            match self.file_repo.stat(&owner, &inner).await? {
                Some(s) => s,
                None => return Ok(None),
            }
        } else {
            // Check if deleted in DB
            let storage_path = format!("/{}", resolved);
            if let Ok(Some(db_meta)) = self.find_db_file.execute(&storage_path).await {
                if db_meta.is_deleted {
                    return Ok(None);
                }
            }

            match self.file_repo.stat(&user.username, &resolved).await? {
                Some(s) => s,
                None => return Ok(None),
            }
        };

        let mut checksum = None;
        if !is_dir {
            let storage_path = if let Some((owner, inner)) = Self::parse_shared(&resolved) {
                 format!("/shared/{}/{}", owner, inner)
            } else {
                 format!("/{}", resolved)
            };
            
            if let Ok(Some(db_meta)) = self.find_db_file.execute(&storage_path).await {
                checksum = db_meta.checksum;
            }
        }

        Ok(Some((size, is_dir, checksum)))
    }
}
