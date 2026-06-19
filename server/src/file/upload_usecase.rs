use crate::common::error::DomainError;
use crate::common::permission::{Permission, PermissionChecker};
use crate::database::domain::DbFileMetadata;
use crate::database::file_usecases::{CreateFileUseCase, FindFileByPathUseCase, UpdateFileUseCase};
use crate::database::user_usecases::FindUserUseCase;

use crate::file::domain::FileMetadata;
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

pub struct UploadUseCase {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    create_db_file: Arc<CreateFileUseCase>,
    update_db_file: Arc<UpdateFileUseCase>,
    find_db_file: Arc<FindFileByPathUseCase>,
    git_service: Arc<GitService>,
    find_user: Arc<FindUserUseCase>,
}

impl UploadUseCase {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        create_db_file: Arc<CreateFileUseCase>,
        update_db_file: Arc<UpdateFileUseCase>,
        find_db_file: Arc<FindFileByPathUseCase>,
        git_service: Arc<GitService>,
        find_user: Arc<FindUserUseCase>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            shares,
            create_db_file,
            update_db_file,
            find_db_file,
            git_service,
            find_user,
        }
    }

    async fn ensure_db_parents(&self, user: &User, path: &str) -> Result<(), DomainError> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = String::new();

        let count = segments.len();
        if count <= 1 {
            return Ok(());
        }

        for i in 0..(count - 1) {
            let segment = segments[i];
            current_path.push_str("/");
            current_path.push_str(segment);

            let existing = self
                .find_db_file
                .execute(user.id, &current_path)
                .await
                .ok()
                .flatten();

            if existing.is_none() {
                let db_entry = DbFileMetadata {
                    id: uuid::Uuid::new_v4(),
                    owner_id: user.id,
                    filename: segment.to_string(),
                    storage_path: current_path.clone(),
                    size_bytes: 0,
                    mime_type: Some("inode/directory".into()),
                    checksum: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    is_deleted: false,
                };
                self.create_db_file
                    .execute(&db_entry)
                    .await
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        _size: u64,
        data: Vec<u8>,
        overwrite: bool,
    ) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);

        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        let actual_size = data.len() as u64;
        if actual_size > MAX_FILE_SIZE {
            return Err(DomainError::FileTooLarge);
        }

        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares
                .consume_write(&user.username, &owner, &inner)
                .await?;
            let meta = FileMetadata::new(&inner, actual_size, &owner);
            self.file_repo.store(meta, data.clone()).await?;

            // Sync DB record for the owner so the file appears in their namespace
            if let Ok(Some(owner_db)) = self.find_user.execute(&owner).await {
                let storage_path = format!("/{}", inner);
                let existing = self
                    .find_db_file
                    .execute(owner_db.id, &storage_path)
                    .await
                    .ok()
                    .flatten();
                let checksum = hex::encode(Sha256::digest(&data));
                if let Some(mut db_entry) = existing {
                    db_entry.size_bytes = actual_size as i64;
                    db_entry.updated_at = chrono::Utc::now();
                    db_entry.checksum = Some(checksum);
                    db_entry.is_deleted = false;
                    self.update_db_file
                        .execute(&db_entry)
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                } else {
                    let db_entry = DbFileMetadata {
                        id: uuid::Uuid::new_v4(),
                        owner_id: owner_db.id,
                        filename: inner.split('/').last().unwrap_or(&inner).to_string(),
                        storage_path,
                        size_bytes: actual_size as i64,
                        mime_type: Some("application/octet-stream".into()),
                        checksum: Some(checksum),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        is_deleted: false,
                    };
                    self.create_db_file
                        .execute(&db_entry)
                        .await
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                }
            }
            return Ok(());
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
            if !overwrite {
                return Err(DomainError::AlreadyExists);
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

        let final_size = final_data.len();
        let meta = FileMetadata::new(&resolved, final_size as u64, &user.username);
        self.file_repo.store(meta, final_data).await?;

        let user_path = self.storage_root.join(&user.username);
        let _ = self.git_service.commit_file(
            &user_path,
            &resolved,
            &format!("Uploaded file: {}", filename),
        );

        self.ensure_db_parents(user, &resolved).await?;

        let owner_id = user.id;
        let storage_path = format!("/{}", resolved);

        let existing = self
            .find_db_file
            .execute(user.id, &storage_path)
            .await
            .ok()
            .flatten();

        if let Some(mut db_entry) = existing {
            db_entry.size_bytes = final_size as i64;
            db_entry.updated_at = chrono::Utc::now();
            db_entry.checksum = Some(checksum);
            db_entry.is_deleted = false;
            self.update_db_file
                .execute(&db_entry)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        } else {
            let db_entry = DbFileMetadata {
                id: uuid::Uuid::new_v4(),
                owner_id,
                filename: filename.to_string(),
                storage_path,
                size_bytes: final_size as i64,
                mime_type: Some("application/octet-stream".into()),
                checksum: Some(checksum),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                is_deleted: false,
            };
            self.create_db_file
                .execute(&db_entry)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        Ok(())
    }
}
