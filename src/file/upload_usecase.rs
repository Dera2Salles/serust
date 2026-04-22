use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::{CreateFileUseCase, FindFileByPathUseCase, UpdateFileUseCase};
use crate::database::user_usecases::FindUserUseCase;
use crate::file::domain::FileMetadata;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::sync::Arc;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

pub struct UploadUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    create_db_file: Arc<CreateFileUseCase>,
    update_db_file: Arc<UpdateFileUseCase>,
    find_db_file: Arc<FindFileByPathUseCase>,
    find_db_user: Arc<FindUserUseCase>,
}

impl UploadUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        create_db_file: Arc<CreateFileUseCase>,
        update_db_file: Arc<UpdateFileUseCase>,
        find_db_file: Arc<FindFileByPathUseCase>,
        find_db_user: Arc<FindUserUseCase>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            create_db_file,
            update_db_file,
            find_db_file,
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

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        size: u64,
        data: Vec<u8>,
    ) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if size > MAX_FILE_SIZE {
            return Err(DomainError::FileTooLarge);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;
            let meta = FileMetadata::new(&inner, size, &owner);
            return self.file_repo.store(meta, data).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        let checksum = hex::encode(Sha256::digest(&data));

        let mut final_data = data;
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        let compressible = matches!(
            ext.as_str(),
            "txt" | "md" | "json" | "csv" | "xml" | "html" | "sql" | "js" | "ts" | "rs" | "dart"
        );

        if compressible {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            if encoder.write_all(&final_data).is_ok() {
                if let Ok(compressed) = encoder.finish() {
                    if compressed.len() < final_data.len() {
                        final_data = compressed;
                    }
                }
            }
        }

        let meta = FileMetadata::new(&resolved, final_data.len() as u64, &user.username);
        self.file_repo.store(meta, final_data).await?;

        let owner_id = self.get_user_id(&user.username).await?;
        let storage_path = format!("/{}", resolved);

        let existing = self
            .find_db_file
            .execute(&storage_path)
            .await
            .ok()
            .flatten();

        if let Some(mut db_entry) = existing {
            db_entry.size_bytes = size as i64;
            db_entry.updated_at = chrono::Utc::now();
            db_entry.checksum = Some(checksum);
            let _ = self.update_db_file.execute(&db_entry).await;
        } else {
            let db_entry = DbFileMetadata {
                id: uuid::Uuid::new_v4(),
                owner_id,
                filename: filename.to_string(),
                storage_path,
                size_bytes: size as i64,
                mime_type: Some("application/octet-stream".into()),
                checksum: Some(checksum),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                is_deleted: false,
            };
            let _ = self.create_db_file.execute(&db_entry).await;
        }

        Ok(())
    }
}
