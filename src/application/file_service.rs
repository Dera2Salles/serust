use crate::domain::{
    error::DomainError,
    file::FileMetadata,
    permission::{Permission, PermissionChecker},
    user::User,
};
use crate::application::share_service::ShareService;
use crate::infrastructure::file_repository::FileRepository;
use std::sync::Arc;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

pub struct FileService {
    file_repo: Arc<FileRepository>,
    shares: Arc<ShareService>,
}

impl FileService {
    pub fn new(file_repo: Arc<FileRepository>, shares: Arc<ShareService>) -> Self {
        Self { file_repo, shares }
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

    async fn list_shared_children(
        &self,
        user: &User,
        owner: &str,
        inner_dir: &str,
    ) -> Result<Vec<(String, bool)>, DomainError> {
        let grants = self.shares.list_incoming(&user.username).await;
        let mut children: Vec<String> = Vec::new();
        let prefix = if inner_dir.is_empty() {
            "".to_string()
        } else {
            format!("{}/", inner_dir.trim_end_matches('/'))
        };

        for g in grants.into_iter().filter(|g| g.owner == owner) {
            if !g.path.starts_with(&prefix) {
                continue;
            }
            let rest = &g.path[prefix.len()..];
            let child = rest.split('/').next().unwrap_or("").trim();
            if !child.is_empty() {
                children.push(child.to_string());
            }
        }

        children.sort();
        children.dedup();

        let mut result = Vec::new();
        for child in children {
            let child_path = if inner_dir.is_empty() {
                child.clone()
            } else {
                format!("{}/{}", inner_dir.trim_end_matches('/'), child)
            };
            let is_dir = match self.file_repo.stat(owner, &child_path).await? {
                Some((_size, is_dir)) => is_dir,
                None => false,
            };
            result.push((child, is_dir));
        }

        Ok(result)
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
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares.consume_write(&user.username, &owner, &inner).await?;
            let meta = FileMetadata::new(&inner, size, &owner);
            return self.file_repo.store(meta, data).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }

        let meta = FileMetadata::new(&resolved, size, &user.username);
        self.file_repo.store(meta, data).await
    }

    pub async fn download(&self, user: &User, cwd: &str, filename: &str) -> Result<Vec<u8>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_download(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            let data = self.file_repo.load(&owner, &inner).await?;
            self.shares.consume_download(&user.username, &owner, &inner).await?;
            return Ok(data);
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
        if resolved.is_empty() {
            let mut entries = self.file_repo.list_entries(&user.username, "").await?;
            if !entries.iter().any(|(n, is_dir)| n == "shared" && *is_dir) {
                entries.push(("shared".to_string(), true));
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            return Ok(entries);
        }

        if resolved == "shared" {
            let owners = self.shares.owners_shared_with(&user.username).await;
            return Ok(owners.into_iter().map(|o| (o, true)).collect());
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            let children = self.list_shared_children(user, &owner, &inner).await?;
            if !inner.is_empty() {
                self.shares.consume_read(&user.username, &owner, &inner).await?;
            }
            return Ok(children);
        }

        self.file_repo.list_entries(&user.username, &resolved).await
    }

    /// List only regular files (no directories) in the current working directory.
    pub async fn list_files(&self, user: &User, cwd: &str) -> Result<Vec<String>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, "");
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            return Ok(vec![]);
        }
        self.file_repo.list_files(&user.username, &resolved).await
    }

    /// Returns (size_bytes, is_dir) for a path, or `Ok(None)` if it doesn't exist.
    pub async fn stat(
        &self,
        user: &User,
        cwd: &str,
        target: &str,
    ) -> Result<Option<(u64, bool)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, target);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Ok(None);
            }
            return self.file_repo.stat(&owner, &inner).await;
        }

        self.file_repo.stat(&user.username, &resolved).await
    }

    pub async fn mkdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
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
            self.shares.consume_write(&user.username, &owner, &inner).await?;
            return self.file_repo.create_dir(&owner, &inner).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }
        self.file_repo.create_dir(&user.username, &resolved).await
    }

    pub async fn rmdir(&self, user: &User, cwd: &str, dirname: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, dirname);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares.consume_write(&user.username, &owner, &inner).await?;
            return self.file_repo.remove_dir(&owner, &inner).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }
        self.file_repo.remove_dir(&user.username, &resolved).await
    }

    pub async fn delete_file(&self, user: &User, cwd: &str, filename: &str) -> Result<(), DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, filename);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if !self.shares.can_write(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            self.shares.consume_write(&user.username, &owner, &inner).await?;
            return self.file_repo.delete_file(&owner, &inner).await;
        }

        if let Some(existing) = self.file_repo.get_metadata(&user.username, &resolved).await {
            if !PermissionChecker::can_access(user, &existing.owner, &Permission::Write) {
                return Err(DomainError::PermissionDenied);
            }
        }
        self.file_repo.delete_file(&user.username, &resolved).await
    }

    /// Returns true if the given path (resolved from cwd) is an existing directory.
    pub async fn dir_exists(&self, user: &User, cwd: &str, path: &str) -> bool {
        let resolved = PermissionChecker::resolve_path(cwd, path);
        if resolved.is_empty() || resolved == "shared" {
            return true;
        }
        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if inner.is_empty() {
                return self
                    .shares
                    .owners_shared_with(&user.username)
                    .await
                    .into_iter()
                    .any(|o| o == owner);
            }
            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return false;
            }
            return self.file_repo.dir_exists(&owner, &inner).await;
        }
        self.file_repo.dir_exists(&user.username, &resolved).await
    }
}
