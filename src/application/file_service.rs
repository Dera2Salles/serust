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
        filename: &str,
        size: u64,
        data: Vec<u8>,
    ) -> Result<(), DomainError> {
        if !PermissionChecker::is_safe_path(filename) {
            return Err(DomainError::UnsafePath);
        }
        if size > MAX_FILE_SIZE {
            return Err(DomainError::FileTooLarge);
        }

        let meta = FileMetadata::new(filename, size, &user.username);
        self.file_repo.store(meta, data).await
    }

    pub async fn download(&self, user: &User, filename: &str) -> Result<Vec<u8>, DomainError> {
        if !PermissionChecker::is_safe_path(filename) {
            return Err(DomainError::UnsafePath);
        }

        let meta = self
            .file_repo
            .get_metadata(&user.username, filename)
            .await
            .ok_or(DomainError::FileNotFound)?;

        if !PermissionChecker::can_access(user, &meta.owner, &Permission::Read) {
            return Err(DomainError::PermissionDenied);
        }

        self.file_repo.load(&user.username, filename).await
    }

    pub async fn list(&self, user: &User) -> Result<Vec<String>, DomainError> {
        self.file_repo.list_files(&user.username).await
    }
}
