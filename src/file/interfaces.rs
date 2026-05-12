use crate::common::error::DomainError;
use crate::file::domain::FileMetadata;
use async_trait::async_trait;

pub trait AsyncReadSeek: tokio::io::AsyncRead + tokio::io::AsyncSeek + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncSeek + Send + ?Sized> AsyncReadSeek for T {}

#[async_trait]
pub trait IFileRepository: Send + Sync {
    async fn store(&self, meta: FileMetadata, data: Vec<u8>) -> Result<(), DomainError>;
    async fn load(&self, username: &str, rel_path: &str) -> Result<Vec<u8>, DomainError>;
    async fn delete_file(&self, username: &str, rel_path: &str) -> Result<(), DomainError>;
    async fn rename(
        &self,
        username: &str,
        old_rel_path: &str,
        new_rel_path: &str,
    ) -> Result<(), DomainError>;
    async fn stat(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Option<(u64, bool)>, DomainError>;
    async fn create_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError>;
    async fn remove_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError>;

    async fn dir_exists(&self, username: &str, rel_path: &str) -> bool;

    async fn list_entries(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Vec<(String, bool)>, DomainError>;

    async fn get_metadata(&self, username: &str, rel_path: &str) -> Option<FileMetadata>;

    async fn get_reader(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<std::pin::Pin<Box<dyn AsyncReadSeek>>, DomainError>;

    async fn get_presigned_url(
        &self,
        _username: &str,
        _rel_path: &str,
    ) -> Result<Option<String>, DomainError> {
        Ok(None)
    }
}
