use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::database::file_usecases::{FindFileByPathUseCase, ListFilesByParentUseCase};
use crate::file::interfaces::IFileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct ListUseCase {
    file_repo: Arc<dyn IFileRepository>,
    shares: Arc<ShareService>,
    _find_db_file: Arc<FindFileByPathUseCase>,
    list_db_files: Arc<ListFilesByParentUseCase>,
}

impl ListUseCase {
    pub fn new(
        file_repo: Arc<dyn IFileRepository>,
        shares: Arc<ShareService>,
        find_db_file: Arc<FindFileByPathUseCase>,
        list_db_files: Arc<ListFilesByParentUseCase>,
    ) -> Self {
        Self {
            file_repo,
            shares,
            _find_db_file: find_db_file,
            list_db_files,
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

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
    ) -> Result<Vec<(String, bool)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, "");

        let db_parent_path = format!("/{}", resolved).trim_end_matches('/').to_string();
        let db_entries = self.list_db_files.execute(user.id, &db_parent_path).await.unwrap_or_default();
        
        if resolved.is_empty() {
            let mut filtered = Vec::new();
            for meta in db_entries {
                if meta.is_deleted { continue; }
                let is_dir = meta.mime_type.as_deref() == Some("inode/directory");
                filtered.push((meta.filename, is_dir));
            }

            if !filtered.iter().any(|(n, is_dir)| n == "shared" && *is_dir) {
                filtered.push(("shared".to_string(), true));
            }
            filtered.sort_by(|a, b| a.0.cmp(&b.0));
            return Ok(filtered);
        }

        if resolved == "shared" {
            let owners = self.shares.owners_shared_with(&user.username).await;
            return Ok(owners.into_iter().map(|o| (o, true)).collect());
        }

        if let Some((owner, inner)) = Self::parse_shared(&resolved) {
            if inner.is_empty() {
                let allowed = self
                    .shares
                    .owners_shared_with(&user.username)
                    .await
                    .into_iter()
                    .any(|o| o == owner);
                if !allowed {
                    return Err(DomainError::PermissionDenied);
                }
                return Ok(self.list_shared_children(user, &owner, "").await?);
            }

            if !self.shares.can_read(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            let children = self.list_shared_children(user, &owner, &inner).await?;
            self.shares
                .consume_read(&user.username, &owner, &inner)
                .await?;
            return Ok(children);
        }

        let mut filtered = Vec::new();
        for meta in db_entries {
            if meta.is_deleted { continue; }
            let is_dir = meta.mime_type.as_deref() == Some("inode/directory");
            filtered.push((meta.filename, is_dir));
        }
        
        if filtered.is_empty() {
            let entries = self.file_repo.list_entries(&user.username, &resolved).await?;
            for (name, is_dir) in entries {
                filtered.push((name, is_dir));
            }
        }

        filtered.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(filtered)
    }
}
