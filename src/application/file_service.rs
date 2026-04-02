use crate::domain::{
    error::DomainError,
    file::FileMetadata,
    permission::{Permission, PermissionChecker},
    user::User,
};
use crate::infrastructure::file_repository::FileRepository;
use std::sync::Arc;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

pub struct FileService {
    file_repo: Arc<FileRepository>,
}

impl FileService {
    pub fn new(file_repo: Arc<FileRepository>) -> Self {
        Self { file_repo }
    }

    pub async fn upload(
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
        let meta = FileMetadata::new(&resolved, size, &user.username);
        self.file_repo.store(meta, data).await
    }

    pub async fn download(&self, user: &User, cwd: &str, filename: &str) -> Result<Vec<u8>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        let meta = self
            .file_repo
            .get_metadata(&user.username, &resolved)
            .await
            .ok_or(DomainError::FileNotFound)?;

        if !PermissionChecker::can_access(user, &meta.owner, &Permission::Read) {
            return Err(DomainError::PermissionDenied);
        }
        self.file_repo.load(&user.username, &resolved).await
    }

    /// List entries (files + dirs) in the current working directory.
    pub async fn list(&self, user: &User, cwd: &str) -> Result<Vec<(String, bool)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, "");
        self.file_repo.list_entries(&user.username, &resolved).await
    }

    pub async fn mkdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        self.file_repo.create_dir(&user.username, &resolved).await
    }

    pub async fn rmdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        self.file_repo.remove_dir(&user.username, &resolved).await
    }

    pub async fn delete_file(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        self.file_repo.delete_file(&user.username, &resolved).await
    }

    /// Returns true if the given path (resolved from cwd) is an existing directory.
    pub async fn dir_exists(&self, user: &User, cwd: &str, path: &str) -> bool {
        let resolved = PermissionChecker::resolve_path(cwd, path);
        self.file_repo.dir_exists(&user.username, &resolved).await
    }
}
