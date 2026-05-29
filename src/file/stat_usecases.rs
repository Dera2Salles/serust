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

        if resolved == "shared" {
            return Ok(Some((0, true, None)));
        }

        let (size, is_dir) = if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            if inner.is_empty() {
                let allowed = self
                    .shares
                    .owners_shared_with(&user.username)
                    .await
                    .into_iter()
                    .any(|o| o == owner);
                if allowed {
                    return Ok(Some((0, true, None)));
                } else {
                    return Ok(None);
                }
            }

            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Ok(None);
            }
            match self.file_repo.stat(&owner, &inner).await? {
                Some(s) => s,
                None => return Ok(None),
            }
        } else {
            let storage_path = format!("/{}", resolved);
            if let Ok(Some(db_meta)) = self.find_db_file.execute(user.id, &storage_path).await {
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
            let storage_path = if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
                format!("/shared/{}/{}", owner, inner)
            } else {
                format!("/{}", resolved)
            };

            let owner_id = if let Some((_owner, _)) = PermissionChecker::parse_shared(&resolved) {
                // For shared files, we need the owner's ID. 
                // But v_effective_permissions view handles this better usually.
                // For now, let's just use user.id if it's not shared, or skip if shared for simplicity
                // since StatUseCase currently assumes finding in current user's DB entries if not shared.
                None
            } else {
                Some(user.id)
            };

            if let Some(oid) = owner_id {
                if let Ok(Some(db_meta)) = self.find_db_file.execute(oid, &storage_path).await {
                    checksum = db_meta.checksum;
                }
            }
        }

        Ok(Some((size, is_dir, checksum)))
    }
}
