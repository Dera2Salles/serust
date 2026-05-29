use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::database::domain::{DbShareGrant, DbShareLink};
use crate::database::share_usecases::{
    CreateGrantUseCase, CreateLinkUseCase, ListMyGrantsUseCase, ListMyLinksUseCase,
    RevokeGrantUseCase, RevokeLinkUseCase,
};
use crate::share::domain::ShareGrant;
use crate::share::local_repository::ShareRepository;
use std::sync::Arc;

pub struct ShareService {
    repo: Arc<ShareRepository>,
    db: crate::database::Database,
    create_link_usecase: Arc<CreateLinkUseCase>,
    create_grant_usecase: Arc<CreateGrantUseCase>,
    list_links_usecase: Arc<ListMyLinksUseCase>,
    list_grants_usecase: Arc<ListMyGrantsUseCase>,
    revoke_link_usecase: Arc<RevokeLinkUseCase>,
    revoke_grant_usecase: Arc<RevokeGrantUseCase>,
}

impl ShareService {
    pub fn new(
        repo: Arc<ShareRepository>,
        db: crate::database::Database,
        create_link_usecase: Arc<CreateLinkUseCase>,
        create_grant_usecase: Arc<CreateGrantUseCase>,
        list_links_usecase: Arc<ListMyLinksUseCase>,
        list_grants_usecase: Arc<ListMyGrantsUseCase>,
        revoke_link_usecase: Arc<RevokeLinkUseCase>,
        revoke_grant_usecase: Arc<RevokeGrantUseCase>,
    ) -> Self {
        Self {
            repo,
            db,
            create_link_usecase,
            create_grant_usecase,
            list_links_usecase,
            list_grants_usecase,
            revoke_link_usecase,
            revoke_grant_usecase,
        }
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
    #[allow(dead_code)]
    pub fn resolve_owner_path(
        actor: &str,
        cwd: &str,
        path: &str,
    ) -> Result<(String, String), DomainError> {
        let resolved = Self::normalize_path(cwd, path)?;
        if let Some((owner, inner)) = PermissionChecker::parse_shared(&resolved) {
            return Ok((owner, inner));
        }
        Ok((actor.to_string(), resolved))
    }

    pub async fn create_direct_share(
        &self,
        owner_id: uuid::Uuid,
        file_id: uuid::Uuid,
        grantee_id: uuid::Uuid,
        can_read: bool,
        can_write: bool,
        can_reshare: bool,
        max_reads: Option<i64>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DomainError> {
        let grant = DbShareGrant {
            id: uuid::Uuid::new_v4(),
            file_id,
            granted_by: owner_id,
            granted_to: grantee_id,
            can_read,
            can_write,
            can_reshare,
            max_reads,
            expires_at,
            granted_at: chrono::Utc::now(),
        };
        self.create_grant_usecase
            .execute(&grant)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
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

        if let Some(existing) = grants
            .iter_mut()
            .find(|g| g.owner == owner && g.path == resolved && g.grantee == grantee)
        {
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

    #[allow(dead_code)]
    pub async fn revoke(
        &self,
        owner: &str,
        cwd: &str,
        path: &str,
        grantee: &str,
    ) -> Result<(), DomainError> {
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

    pub async fn list_my_links(&self, owner_id: uuid::Uuid) -> Result<Vec<DbShareLink>, DomainError> {
        self.list_links_usecase
            .execute(owner_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn list_my_grants(&self, owner_id: uuid::Uuid) -> Result<Vec<DbShareGrant>, DomainError> {
        self.list_grants_usecase
            .execute(owner_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn create_public_link(
        &self,
        owner_id: uuid::Uuid,
        file_id: uuid::Uuid,
        token: String,
        label: Option<String>,
        can_read: bool,
        can_write: bool,
        can_reshare: bool,
        max_reads: Option<i64>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        password: Option<String>,
    ) -> Result<(), DomainError> {
        let password_hash = password.map(|p| crate::user::service::AuthService::hash_password(&p));
        let link = DbShareLink {
            id: uuid::Uuid::new_v4(),
            file_id,
            created_by: owner_id,
            token,
            label,
            can_read,
            can_write,
            can_reshare,
            max_reads,
            expires_at,
            password_hash,
            is_active: true,
        };
        self.create_link_usecase
            .execute(&link)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn revoke_link(&self, id: uuid::Uuid) -> Result<(), DomainError> {
        self.revoke_link_usecase
            .execute(id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn revoke_grant(&self, id: uuid::Uuid) -> Result<(), DomainError> {
        self.revoke_grant_usecase
            .execute(id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn list_incoming(&self, grantee_username: &str) -> Vec<ShareGrant> {
        let rows = sqlx::query(
            "SELECT f.storage_path, u_owner.username as owner_username, u_grantor.username as grantor_username, p.can_read, p.can_write, p.is_expired, p.reads_remaining
             FROM v_effective_permissions p
             JOIN files f ON f.id = p.file_id
             JOIN users u_owner ON u_owner.id = f.owner_id
             JOIN users u_grantee ON u_grantee.id = p.user_id
             LEFT JOIN share_grants g ON g.id = p.grant_id
             LEFT JOIN users u_grantor ON u_grantor.id = g.granted_by
             WHERE u_grantee.username = ? AND p.source = 'grant' AND p.is_valid = 1"
        )
        .bind(grantee_username)
        .fetch_all(&*self.db.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(|r| {
            use sqlx::Row;
            ShareGrant {
                owner: r.get("owner_username"),
                path: r.get::<String, _>("storage_path").trim_start_matches('/').to_string(),
                grantee: grantee_username.to_string(),
                can_read: r.get::<i32, _>("can_read") != 0,
                can_write: r.get::<i32, _>("can_write") != 0,
                can_download: true,
                remaining_reads: r.get::<Option<i32>, _>("reads_remaining").map(|v| v as u64),
                remaining_writes: None,
                remaining_downloads: None,
                can_reshare: false,
                granted_by: r.get::<Option<String>, _>("grantor_username").unwrap_or_else(|| r.get("owner_username")),
                expires_at: None,
            }
        }).collect()
    }

    pub async fn owners_shared_with(&self, grantee_username: &str) -> Vec<String> {
        let rows = sqlx::query(
            "SELECT DISTINCT u_owner.username
             FROM v_effective_permissions p
             JOIN files f ON f.id = p.file_id
             JOIN users u_owner ON u_owner.id = f.owner_id
             JOIN users u_grantee ON u_grantee.id = p.user_id
             WHERE u_grantee.username = ? AND p.source = 'grant' AND p.is_valid = 1"
        )
        .bind(grantee_username)
        .fetch_all(&*self.db.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(|r| {
            use sqlx::Row;
            r.get::<String, _>(0)
        }).collect()
    }

    pub async fn can_read(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let storage_path = if owner_rel_path.starts_with('/') { owner_rel_path.to_string() } else { format!("/{}", owner_rel_path) };
        
        // A path is readable if:
        // 1. It is directly shared.
        // 2. A parent folder is shared (inherited permission).
        // 3. It is a parent of a shared item (discovery permission for virtual folders).
        let query = "
            SELECT 1 FROM v_effective_permissions p
            JOIN files f_shared ON f_shared.id = p.file_id
            JOIN users u_owner ON u_owner.id = f_shared.owner_id
            JOIN users u_actor ON u_actor.id = p.user_id
            WHERE u_actor.username = ? 
              AND u_owner.username = ? 
              AND p.can_read = 1 
              AND p.is_valid = 1
              AND (
                  f_shared.storage_path = ? 
                  OR (? || '/') LIKE (f_shared.storage_path || '/%')
                  OR (f_shared.storage_path || '/') LIKE (? || '/%')
              )
            LIMIT 1";

        let row = sqlx::query(query)
            .bind(actor)
            .bind(owner)
            .bind(&storage_path)
            .bind(&storage_path)
            .bind(&storage_path)
            .fetch_optional(&*self.db.pool)
            .await
            .unwrap_or(None);

        row.is_some()
    }

    pub async fn can_write(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let storage_path = if owner_rel_path.starts_with('/') { owner_rel_path.to_string() } else { format!("/{}", owner_rel_path) };
        
        let query = "
            SELECT 1 FROM v_effective_permissions p
            JOIN files f_shared ON f_shared.id = p.file_id
            JOIN users u_owner ON u_owner.id = f_shared.owner_id
            JOIN users u_actor ON u_actor.id = p.user_id
            WHERE u_actor.username = ? 
              AND u_owner.username = ? 
              AND p.can_write = 1 
              AND p.is_valid = 1
              AND (
                  f_shared.storage_path = ? 
                  OR (? || '/') LIKE (f_shared.storage_path || '/%')
              )
            LIMIT 1";

        let row = sqlx::query(query)
            .bind(actor)
            .bind(owner)
            .bind(&storage_path)
            .bind(&storage_path)
            .fetch_optional(&*self.db.pool)
            .await
            .unwrap_or(None);

        row.is_some()
    }

    pub async fn can_download(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        self.can_read(actor, owner, owner_rel_path).await
    }

    pub async fn consume_read(
        &self,
        actor: &str,
        owner: &str,
        owner_rel_path: &str,
    ) -> Result<(), DomainError> {
        if actor == owner {
            return Ok(());
        }
        // Log access to trigger read counter
        let storage_path = if owner_rel_path.starts_with('/') { owner_rel_path.to_string() } else { format!("/{}", owner_rel_path) };
        let row = sqlx::query(
            "SELECT p.file_id, p.grant_id FROM v_effective_permissions p
             JOIN files f ON f.id = p.file_id
             JOIN users u_owner ON u_owner.id = f.owner_id
             JOIN users u_actor ON u_actor.id = p.user_id
             WHERE u_actor.username = ? AND u_owner.username = ? AND f.storage_path = ? AND p.can_read = 1 AND p.is_valid = 1"
        )
        .bind(actor)
        .bind(owner)
        .bind(&storage_path)
        .fetch_optional(&*self.db.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let file_id: String = r.get("file_id");
            let grant_id: Option<String> = r.get("grant_id");
            let actor_row = sqlx::query("SELECT id FROM users WHERE username = ?")
                .bind(actor)
                .fetch_one(&*self.db.pool).await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let actor_id: String = actor_row.get(0);

            sqlx::query(
                "INSERT INTO access_log (file_id, accessed_by, grant_id, action, accessed_at) VALUES (?, ?, ?, 'read', CURRENT_TIMESTAMP)"
            )
            .bind(file_id)
            .bind(actor_id)
            .bind(grant_id)
            .execute(&*self.db.pool).await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn consume_write(
        &self,
        _actor: &str,
        _owner: &str,
        _owner_rel_path: &str,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    pub async fn consume_download(
        &self,
        actor: &str,
        owner: &str,
        owner_rel_path: &str,
    ) -> Result<(), DomainError> {
        self.consume_read(actor, owner, owner_rel_path).await
    }

    #[allow(dead_code)]
    pub async fn can_reshare(&self, actor: &str, owner: &str, _owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        // Simplified for now, could also check DB
        false
    }
}
