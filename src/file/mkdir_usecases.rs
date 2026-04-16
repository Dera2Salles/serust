use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::CreateFileUseCase;
use crate::database::user_usecases::FindUserUseCase;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct MkdirUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    create_db_file: Arc<CreateFileUseCase>,
    find_db_user: Arc<FindUserUseCase>,
}

impl MkdirUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        create_db_file: Arc<CreateFileUseCase>,
        find_db_user: Arc<FindUserUseCase>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            create_db_file,
            find_db_user,
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

    async fn get_user_id(&self, username: &str) -> Result<uuid::Uuid, DomainError> {
        self.find_db_user
            .execute(username)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .map(|u| u.id)
            .ok_or(DomainError::InvalidCredentials)
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
            return self.file_repo.create_dir(&owner, &inner).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        self.file_repo.create_dir(&user.username, &resolved).await?;

        let owner_id = self.get_user_id(&user.username).await?;
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
