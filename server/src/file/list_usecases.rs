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

    async fn list_shared_children(
        &self,
        user: &User,
        owner: &str,
        inner_dir: &str,
    ) -> Result<Vec<(String, bool)>, DomainError> {
        let mut result_map: std::collections::HashMap<String, bool> = std::collections::HashMap::new();

        // 1. If this directory itself is accessible, list its actual contents from disk
        if self.shares.can_read(&user.username, owner, inner_dir).await {
            if let Ok(disk_entries) = self.file_repo.list_entries(owner, inner_dir).await {
                for (name, is_dir) in disk_entries {
                    result_map.insert(name, is_dir);
                }
            }
        }

        // 2. Also include explicitly shared items that might be deeper but whose immediate 
        // parent is the current directory (even if the current directory isn't shared).
        let grants = self.shares.list_incoming(&user.username).await;
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
            if rest.is_empty() { continue; }
            
            let child = rest.split('/').next().unwrap_or("").trim();
            if !child.is_empty() && !result_map.contains_key(child) {
                let child_path = if inner_dir.is_empty() {
                    child.to_string()
                } else {
                    format!("{}/{}", inner_dir.trim_end_matches('/'), child)
                };
                let is_dir = match self.file_repo.stat(owner, &child_path).await? {
                    Some((_size, is_dir)) => is_dir,
                    None => false,
                };
                result_map.insert(child.to_string(), is_dir);
            }
        }

        let mut result: Vec<(String, bool)> = result_map.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    pub async fn execute(
        &self,
        user: &User,
        cwd: &str,
    ) -> Result<Vec<(String, bool)>, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, "");

        if resolved == "shared" {
            let owners = self.shares.owners_shared_with(&user.username).await;
            let mut res: Vec<(String, bool)> = owners.into_iter().map(|o| (o, true)).collect();
            res.sort();
            return Ok(res);
        }

        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
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

            if !self.shares.can_discover(&user.username, &owner, &inner).await {
                return Err(DomainError::PermissionDenied);
            }
            let children = self.list_shared_children(user, &owner, &inner).await?;
            self.shares
                .consume_read(&user.username, &owner, &inner)
                .await?;
            return Ok(children);
        }

        // 1. List everything from disk
        let disk_entries = self.file_repo.list_entries(&user.username, &resolved).await?;
        
        // 2. Get metadata from DB for this parent path to check for deleted files
        let db_parent_path = format!("/{}", resolved).trim_end_matches('/').to_string();
        let db_entries = self.list_db_files.execute(user.id, &db_parent_path).await.unwrap_or_default();
        
        let mut filtered = Vec::new();
        for (name, is_dir) in disk_entries {
            // Check if this specific entry is marked as deleted in DB
            let is_deleted = db_entries.iter().any(|meta| meta.filename == name && meta.is_deleted);
            if !is_deleted {
                filtered.push((name, is_dir));
            }
        }

        // 3. Add virtual folders if at root
        if resolved.is_empty() {
            if !filtered.iter().any(|(n, is_dir)| n == "shared" && *is_dir) {
                filtered.push(("shared".to_string(), true));
            }
        }

        filtered.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(filtered)
    }
}
