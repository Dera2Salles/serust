use crate::common::error::DomainError;
use crate::file::compression_service::{CompressionFormat, CompressionService};
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::file::{
    DeleteUseCase, DownloadUseCase, ListUseCase, MkdirUseCase,
    RenameUseCase, StatUseCase, UploadUseCase,
    RestoreUseCase, PurgeUseCase,
};
use crate::database::file_usecases::{FindDeletedFilesDbUseCase, FindFileByPathUseCase};
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::path::PathBuf;
use std::sync::Arc;

pub struct FileService {
    storage_root: PathBuf,
    file_repo: Arc<dyn IFileRepository>,
    download: Arc<DownloadUseCase>,
    upload: Arc<UploadUseCase>,
    list: Arc<ListUseCase>,
    mkdir: Arc<MkdirUseCase>,
    delete: Arc<DeleteUseCase>,
    stat: Arc<StatUseCase>,
    rename: Arc<RenameUseCase>,
    restore: Arc<RestoreUseCase>,
    purge: Arc<PurgeUseCase>,
    find_deleted: Arc<FindDeletedFilesDbUseCase>,
    find_file_by_path: Arc<FindFileByPathUseCase>,

    pub git: Arc<GitService>,
    pub compression: Arc<CompressionService>,
    pub shares: Arc<ShareService>,
}

impl FileService {
    pub fn new(
        storage_root: PathBuf,
        file_repo: Arc<dyn IFileRepository>,
        download: Arc<DownloadUseCase>,
        upload: Arc<UploadUseCase>,
        list: Arc<ListUseCase>,
        mkdir: Arc<MkdirUseCase>,
        delete: Arc<DeleteUseCase>,
        stat: Arc<StatUseCase>,
        rename: Arc<RenameUseCase>,
        restore: Arc<RestoreUseCase>,
        purge: Arc<PurgeUseCase>,
        find_deleted: Arc<FindDeletedFilesDbUseCase>,
        find_file_by_path: Arc<FindFileByPathUseCase>,

        git: Arc<GitService>,
        compression: Arc<CompressionService>,
        shares: Arc<ShareService>,
    ) -> Self {
        Self {
            storage_root,
            file_repo,
            download,
            upload,
            list,
            mkdir,
            delete,
            stat,
            rename,
            restore,
            purge,
            find_deleted,
            find_file_by_path,
            git,
            compression,
            shares,
        }
    }

    fn user_path(&self, username: &str) -> PathBuf {
        self.storage_root.join(username)
    }

    pub async fn git_history(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<Vec<(String, i64, String)>, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        let user_path = self.user_path(&user.username);
        self.git.get_history(&user_path, &resolved)
    }

    pub async fn git_restore(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        hash: &str,
    ) -> Result<(), DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        let user_path = self.user_path(&user.username);
        let (historical_name, content) = self.git.restore_version(&user_path, &resolved, hash)?;
        
        let old_dir = resolved.split('/').collect::<Vec<_>>()[..resolved.split('/').count() - 1].join("/");
        let new_resolved = if old_dir.is_empty() {
            historical_name
        } else {
            format!("{}/{}", old_dir, historical_name)
        };
        
        self.file_repo.rename(&user.username, &resolved, &new_resolved).await?;
        
        let meta = crate::file::domain::FileMetadata::new(&new_resolved, content.len() as u64, &user.username);
        self.file_repo.store(meta, content).await?;

        Ok(())
    }

    pub async fn git_diff(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        hash: &str,
    ) -> Result<String, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        let user_path = self.user_path(&user.username);
        self.git.get_diff(&user_path, &resolved, hash)
    }

    pub async fn compress(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        format_str: &str,
    ) -> Result<String, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        let format = match format_str.to_uppercase().as_str() {
            "ZIP" => CompressionFormat::Zip,
            "TAR.GZ" | "TGZ" => CompressionFormat::TarGz,
            _ => return Err(DomainError::Internal("Unsupported format".into())),
        };
        let user_path = self.user_path(&user.username);
        self.compression.compress(&user_path, &resolved, format)
    }

    pub async fn decompress(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<(), DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        let user_path = self.user_path(&user.username);
        self.compression.decompress(&user_path, &resolved)
    }

    pub async fn download(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<Vec<u8>, DomainError> {
        self.download.execute(user, cwd, filename).await
    }

    pub async fn upload(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
        size: u64,
        data: Vec<u8>,
    ) -> Result<(), DomainError> {
        self.upload.execute(user, cwd, filename, size, data).await
    }

    pub async fn list(&self, user: &User, cwd: &str) -> Result<Vec<(String, bool)>, DomainError> {
        self.list.execute(user, cwd).await
    }

    pub async fn mkdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        self.mkdir.execute(user, cwd, dirname).await
    }

    pub async fn delete(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        self.delete.execute(user, cwd, filename).await
    }

    pub async fn stat(
        &self,
        user: &User,
        cwd: &str,
        target: &str,
    ) -> Result<Option<(u64, bool, Option<String>)>, DomainError> {
        self.stat.execute(user, cwd, target).await
    }

    pub async fn find_db_file_by_path(
        &self,
        user_id: uuid::Uuid,
        path: &str,
    ) -> Result<Option<crate::database::domain::DbFileMetadata>, DomainError> {
        let storage_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        self.find_file_by_path
            .execute(user_id, &storage_path)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn rename(
        &self,
        user: &User,
        cwd: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DomainError> {
        self.rename.execute(user, cwd, old_name, new_name).await
    }

    pub async fn restore(&self, user: &User, id: uuid::Uuid) -> Result<(), DomainError> {
        self.restore.execute(user, id).await
    }

    pub async fn purge(&self, user: &User, id: uuid::Uuid) -> Result<(), DomainError> {
        self.purge.execute(user, id).await
    }

    pub async fn list_deleted(&self, user: &User) -> Result<Vec<crate::database::domain::DbFileMetadata>, DomainError> {
        self.find_deleted.execute(user.id).await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn find_db_file(&self, user_id: uuid::Uuid, path: &str) -> Result<Option<crate::database::domain::DbFileMetadata>, DomainError> {
        let storage_path = if path.starts_with('/') { path.to_string() } else { format!("/{}", path) };
        self.find_file_by_path.execute(user_id, &storage_path).await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn get_reader(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<std::pin::Pin<Box<dyn crate::file::interfaces::AsyncReadSeek>>, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        if !crate::common::permission::PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = crate::common::permission::PermissionChecker::parse_shared(&resolved) {
            if !self.shares.can_download(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            return self.file_repo.get_reader(&owner, &inner).await;
        }

        self.file_repo.get_reader(&user.username, &resolved).await
    }

    pub async fn get_presigned_url(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<Option<String>, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        if !crate::common::permission::PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = crate::common::permission::PermissionChecker::parse_shared(&resolved) {
            if !self.shares.can_download(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            return self.file_repo.get_presigned_url(&owner, &inner).await;
        }

        self.file_repo.get_presigned_url(&user.username, &resolved).await
    }
}
