use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::file::local_repository::FileRepository;
use crate::share::service::ShareService;
use crate::user::domain::User;
use std::sync::Arc;

pub struct ListUseCase {
    file_repo: Arc<FileRepository>,
    shares: Arc<ShareService>,
}

impl ListUseCase {
    pub fn new(
        file_repo: Arc<FileRepository>,
        shares: Arc<ShareService>,
    ) -> Self {
        Self {
            file_repo,
            shares,
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

        self.file_repo.list_entries(&user.username, &resolved).await
    }
}
