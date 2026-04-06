use crate::domain::{error::DomainError, file::FileMetadata};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct FileRepository {
    storage_root: PathBuf,
}

impl FileRepository {
    pub fn new(storage_root: impl Into<PathBuf>) -> Self {
        Self {
            storage_root: storage_root.into(),
        }
    }

    fn user_dir(&self, username: &str) -> PathBuf {
        self.storage_root.join(username)
    }

    /// Resolves a path (relative to the user's root) safely.
    /// `rel_path` is already normalised by PermissionChecker::resolve_path.
    fn user_path(&self, username: &str, rel_path: &str) -> PathBuf {
        if rel_path.is_empty() {
            self.user_dir(username)
        } else {
            self.user_dir(username).join(rel_path)
        }
    }

    pub async fn store(&self, meta: FileMetadata, data: Vec<u8>) -> Result<(), DomainError> {
        let path = self.user_path(&meta.owner, &meta.filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;
        file.flush().await?;
        Ok(())
    }

    pub async fn load(&self, username: &str, rel_path: &str) -> Result<Vec<u8>, DomainError> {
        let path = self.user_path(username, rel_path);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        let data = fs::read(&path).await?;
        Ok(data)
    }

    pub async fn get_metadata(&self, username: &str, rel_path: &str) -> Option<FileMetadata> {
        let path = self.user_path(username, rel_path);
        if !path.exists() {
            return None;
        }
        let meta = fs::metadata(&path).await.ok()?;
        Some(FileMetadata::new(rel_path, meta.len(), username))
    }

    /// List files AND directories in a given sub-directory (rel_path = "" means root).
    pub async fn list_entries(&self, username: &str, rel_path: &str) -> Result<Vec<(String, bool)>, DomainError> {
        let dir = self.user_path(username, rel_path);
        if !dir.exists() {
            fs::create_dir_all(&dir).await?;
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(&dir).await?;
        let mut result = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let is_dir = path.is_dir();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                result.push((name.to_string(), is_dir));
            }
        }

        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    /// Legacy: list only file names (kept for backward compat).
    pub async fn list_files(&self, username: &str, rel_path: &str) -> Result<Vec<String>, DomainError> {
        let entries = self.list_entries(username, rel_path).await?;
        Ok(entries.into_iter().filter(|(_, is_dir)| !is_dir).map(|(n, _)| n).collect())
    }

    /// Returns (size_bytes, is_dir) for an entry, or `Ok(None)` if it doesn't exist.
    pub async fn stat(
        &self,
        username: &str,
        rel_path: &str,
    ) -> Result<Option<(u64, bool)>, DomainError> {
        let path = self.user_path(username, rel_path);
        if !path.exists() {
            return Ok(None);
        }
        let meta = fs::metadata(&path).await?;
        Ok(Some((meta.len(), meta.is_dir())))
    }

    pub async fn create_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        let path = self.user_path(username, rel_path);
        fs::create_dir_all(&path).await?;
        Ok(())
    }

    pub async fn remove_dir(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        let path = self.user_path(username, rel_path);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        fs::remove_dir_all(&path).await?;
        Ok(())
    }

    pub async fn delete_file(&self, username: &str, rel_path: &str) -> Result<(), DomainError> {
        let path = self.user_path(username, rel_path);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        fs::remove_file(&path).await?;
        Ok(())
    }

    /// Returns true if the given path is an existing directory.
    pub async fn dir_exists(&self, username: &str, rel_path: &str) -> bool {
        let path = self.user_path(username, rel_path);
        path.exists() && path.is_dir()
    }
}
