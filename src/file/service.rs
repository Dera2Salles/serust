use crate::common::error::DomainError;
use crate::file::compression_service::{CompressionFormat, CompressionService};
use crate::file::git_service::GitService;
use crate::file::interfaces::IFileRepository;
use crate::file::{
    DeleteUseCase, DirExistsUseCase, DownloadUseCase, ListUseCase, MkdirUseCase, PurgeUseCase,
    RemoveDirUseCase, RenameUseCase, RestoreUseCase, StatUseCase, UploadUseCase,
};
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
    rmdir: Arc<RemoveDirUseCase>,
    dir_exists: Arc<DirExistsUseCase>,
    restore: Arc<RestoreUseCase>,
    purge: Arc<PurgeUseCase>,
    pub git: Arc<GitService>,
    pub compression: Arc<CompressionService>,
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
        rmdir: Arc<RemoveDirUseCase>,
        dir_exists: Arc<DirExistsUseCase>,
        restore: Arc<RestoreUseCase>,
        purge: Arc<PurgeUseCase>,
        git: Arc<GitService>,
        compression: Arc<CompressionService>,
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
            rmdir,
            dir_exists,
            restore,
            purge,
            git,
            compression,
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
        self.git.restore_version(&user_path, &resolved, hash)
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

    pub async fn rename(
        &self,
        user: &User,
        cwd: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), DomainError> {
        self.rename.execute(user, cwd, old_name, new_name).await
    }

    pub async fn rmdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        self.rmdir.execute(user, cwd, dirname).await
    }

    pub async fn dir_exists(&self, user: &User, cwd: &str, dirname: &str) -> bool {
        self.dir_exists.execute(user, cwd, dirname).await
    }

    pub async fn restore(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        self.restore.execute(user, cwd, filename).await
    }

    pub async fn purge(&self, user: &User) -> Result<(), DomainError> {
        self.purge.execute(user).await
    }

    pub async fn get_reader(
        &self,
        user: &User,
        cwd: &str,
        filename: &str,
    ) -> Result<tokio::fs::File, DomainError> {
        let resolved = crate::common::permission::PermissionChecker::resolve_path(cwd, filename);
        if !crate::common::permission::PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        // For now, only allowing owner to get reader directly. 
        self.file_repo.get_reader(&user.username, &resolved).await
    }
}
