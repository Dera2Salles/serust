use crate::domain::{error::DomainError, permission::PermissionChecker, share::ShareGrant};
use crate::infrastructure::share_repository::ShareRepository;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ShareService {
    repo: Arc<ShareRepository>,
}

impl ShareService {
    pub fn new(repo: Arc<ShareRepository>) -> Self {
        Self { repo }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn normalize_path(cwd: &str, path: &str) -> Result<String, DomainError> {
        let resolved = PermissionChecker::resolve_path(cwd, path);
        if !PermissionChecker::is_safe_path(&resolved) {
            return Err(DomainError::UnsafePath);
        }
        Ok(resolved)
    }

    /// Resolves a path possibly referring to shared namespace:
    /// - "shared/<owner>/<path...>" → (owner, inner_path)
    /// - otherwise it belongs to `actor` → (actor, resolved_path)
    pub fn resolve_owner_path(actor: &str, cwd: &str, path: &str) -> Result<(String, String), DomainError> {
        let resolved = Self::normalize_path(cwd, path)?;
        if let Some(rest) = resolved.strip_prefix("shared/") {
            let mut parts = rest.splitn(2, '/');
            let owner = parts.next().unwrap_or("").to_string();
            let inner = parts.next().unwrap_or("").to_string();
            if owner.is_empty() {
                return Err(DomainError::FileNotFound);
            }
            return Ok((owner, inner));
        }
        Ok((actor.to_string(), resolved))
    }

    pub async fn grant(
        &self,
        owner: &str,
        cwd: &str,
        path: &str,
        grantee: &str,
        can_read: bool,
        can_write: bool,
        can_download: bool,
        remaining_reads: Option<u64>,
        remaining_writes: Option<u64>,
        remaining_downloads: Option<u64>,
        can_reshare: bool,
        granted_by: &str,
        expires_at: Option<u64>,
    ) -> Result<(), DomainError> {
        let resolved = Self::normalize_path(cwd, path)?;
        let mut grants = self.repo.all().await;

        // Upsert
        if let Some(existing) = grants.iter_mut().find(|g| g.owner == owner && g.path == resolved && g.grantee == grantee) {
            existing.can_read = can_read;
            existing.can_write = can_write;
            existing.can_download = can_download;
            existing.remaining_reads = remaining_reads;
            existing.remaining_writes = remaining_writes;
            existing.remaining_downloads = remaining_downloads;
            existing.can_reshare = can_reshare;
            existing.granted_by = granted_by.to_string();
            existing.expires_at = expires_at;
            return self.repo.replace_all(grants).await;
        }

        self.repo
            .push(ShareGrant {
                owner: owner.to_string(),
                path: resolved,
                grantee: grantee.to_string(),
                can_read,
                can_write,
                can_download,
                remaining_reads,
                remaining_writes,
                remaining_downloads,
                can_reshare,
                granted_by: granted_by.to_string(),
                expires_at,
            })
            .await
    }

    pub async fn revoke(&self, owner: &str, cwd: &str, path: &str, grantee: &str) -> Result<(), DomainError> {
        let resolved = Self::normalize_path(cwd, path)?;
        let mut grants = self.repo.all().await;
        let before = grants.len();
        grants.retain(|g| !(g.owner == owner && g.path == resolved && g.grantee == grantee));
        if grants.len() == before {
            return Err(DomainError::FileNotFound);
        }
        self.repo.replace_all(grants).await
    }

    pub async fn list_outgoing(&self, owner: &str) -> Vec<ShareGrant> {
        self.repo
            .all()
            .await
            .into_iter()
            .filter(|g| g.owner == owner)
            .collect()
    }

    pub async fn list_incoming(&self, grantee: &str) -> Vec<ShareGrant> {
        let now = Self::now_secs();
        self.repo
            .all()
            .await
            .into_iter()
            .filter(|g| g.grantee == grantee && g.expires_at.map_or(true, |exp| exp > now))
            .collect()
    }

    pub async fn owners_shared_with(&self, grantee: &str) -> Vec<String> {
        let mut owners: Vec<String> = self
            .list_incoming(grantee)
            .await
            .into_iter()
            .map(|g| g.owner)
            .collect();
        owners.sort();
        owners.dedup();
        owners
    }

    fn grant_allows_path(grant_path: &str, target: &str) -> bool {
        if grant_path == target {
            return true;
        }
        if grant_path.is_empty() {
            // Sharing root: everything
            return true;
        }
        let prefix = format!("{}/", grant_path.trim_end_matches('/'));
        target.starts_with(&prefix)
    }

    fn is_expired(g: &ShareGrant, now: u64) -> bool {
        g.expires_at.map_or(false, |exp| exp <= now)
    }

    pub async fn can_read(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let now = Self::now_secs();
        self.repo
            .all()
            .await
            .into_iter()
            .any(|g| g.owner == owner && g.grantee == actor && g.can_read && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path))
    }

    pub async fn can_write(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let now = Self::now_secs();
        self.repo
            .all()
            .await
            .into_iter()
            .any(|g| g.owner == owner && g.grantee == actor && g.can_write && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path))
    }

    pub async fn can_download(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let now = Self::now_secs();
        self.repo
            .all()
            .await
            .into_iter()
            .any(|g| g.owner == owner && g.grantee == actor && g.can_download && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path))
    }

    pub async fn consume_read(&self, actor: &str, owner: &str, owner_rel_path: &str) -> Result<(), DomainError> {
        if actor == owner {
            return Ok(());
        }
        let now = Self::now_secs();
        let mut grants = self.repo.all().await;
        // Consume the most specific matching grant (longest path)
        let mut idx: Option<usize> = None;
        let mut best_len = 0usize;
        for (i, g) in grants.iter().enumerate() {
            if g.owner == owner && g.grantee == actor && g.can_read && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path) {
                let l = g.path.len();
                if l >= best_len {
                    best_len = l;
                    idx = Some(i);
                }
            }
        }
        let Some(i) = idx else { return Err(DomainError::PermissionDenied); };
        if let Some(rem) = grants[i].remaining_reads.as_mut() {
            if *rem == 0 {
                return Err(DomainError::PermissionDenied);
            }
            *rem -= 1;
        }
        self.repo.replace_all(grants).await
    }

    pub async fn consume_write(&self, actor: &str, owner: &str, owner_rel_path: &str) -> Result<(), DomainError> {
        if actor == owner {
            return Ok(());
        }
        let now = Self::now_secs();
        let mut grants = self.repo.all().await;
        let mut idx: Option<usize> = None;
        let mut best_len = 0usize;
        for (i, g) in grants.iter().enumerate() {
            if g.owner == owner && g.grantee == actor && g.can_write && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path) {
                let l = g.path.len();
                if l >= best_len {
                    best_len = l;
                    idx = Some(i);
                }
            }
        }
        let Some(i) = idx else { return Err(DomainError::PermissionDenied); };
        if let Some(rem) = grants[i].remaining_writes.as_mut() {
            if *rem == 0 {
                return Err(DomainError::PermissionDenied);
            }
            *rem -= 1;
        }
        self.repo.replace_all(grants).await
    }

    pub async fn consume_download(&self, actor: &str, owner: &str, owner_rel_path: &str) -> Result<(), DomainError> {
        if actor == owner {
            return Ok(());
        }
        let now = Self::now_secs();
        let mut grants = self.repo.all().await;
        let mut idx: Option<usize> = None;
        let mut best_len = 0usize;
        for (i, g) in grants.iter().enumerate() {
            if g.owner == owner && g.grantee == actor && g.can_download && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path) {
                let l = g.path.len();
                if l >= best_len {
                    best_len = l;
                    idx = Some(i);
                }
            }
        }
        let Some(i) = idx else { return Err(DomainError::PermissionDenied); };
        if let Some(rem) = grants[i].remaining_downloads.as_mut() {
            if *rem == 0 {
                return Err(DomainError::PermissionDenied);
            }
            *rem -= 1;
        }
        self.repo.replace_all(grants).await
    }

    pub async fn can_reshare(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let now = Self::now_secs();
        self.repo
            .all()
            .await
            .into_iter()
            .any(|g| g.owner == owner && g.grantee == actor && g.can_reshare && !Self::is_expired(&g, now) && Self::grant_allows_path(&g.path, owner_rel_path))
    }
}

