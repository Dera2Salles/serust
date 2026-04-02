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

    fn file_path(&self, username: &str, filename: &str) -> PathBuf {
        self.user_dir(username).join(filename)
    }

    pub async fn store(&self, meta: FileMetadata, data: Vec<u8>) -> Result<(), DomainError> {
        let dir = self.user_dir(&meta.owner);
        fs::create_dir_all(&dir).await?;

        let path = self.file_path(&meta.owner, &meta.filename);
        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        Ok(())
    }

    /// Charge un fichier depuis le disque.
    pub async fn load(&self, username: &str, filename: &str) -> Result<Vec<u8>, DomainError> {
        let path = self.file_path(username, filename);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        let data = fs::read(&path).await?;
        Ok(data)
    }

    /// Retourne les métadonnées d'un fichier (si existant).
    pub async fn get_metadata(&self, username: &str, filename: &str) -> Option<FileMetadata> {
        let path = self.file_path(username, filename);
        if !path.exists() {
            return None;
        }
        let meta = fs::metadata(&path).await.ok()?;
        Some(FileMetadata::new(filename, meta.len(), username))
    }

    pub async fn list_files(&self, username: &str) -> Result<Vec<String>, DomainError> {
        let dir = self.user_dir(username);
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(&dir).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(name.to_string());
            }
        }

        files.sort();
        Ok(files)
    }

    pub async fn create_dir(&self, username: &str, dirname: &str) -> Result<(), DomainError> {
        let path = self.user_dir(username).join(dirname);
        fs::create_dir_all(&path).await?;
        Ok(())
    }

    pub async fn remove_dir(&self, username: &str, dirname: &str) -> Result<(), DomainError> {
        let path = self.user_dir(username).join(dirname);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        fs::remove_dir_all(&path).await?;
        Ok(())
    }

    pub async fn delete_file(&self, username: &str, filename: &str) -> Result<(), DomainError> {
        let path = self.file_path(username, filename);
        if !path.exists() {
            return Err(DomainError::FileNotFound);
        }
        fs::remove_file(&path).await?;
        Ok(())
    }
}
