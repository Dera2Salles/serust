use crate::common::error::DomainError;
use crate::common::permission::PermissionChecker;
use crate::database::domain::{DbShareGrant, DbShareLink};
use crate::database::entities::{access_log, users};
use crate::database::share_usecases::{
    CreateGrantUseCase, CreateLinkUseCase, ListMyGrantsUseCase, ListMyLinksUseCase,
    RevokeGrantUseCase, RevokeLinkUseCase,
};
use crate::share::domain::ShareGrant;
use crate::share::local_repository::ShareRepository;
use sea_orm::{
    prelude::*, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, Set, Statement,
};
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
            self.repo.replace_all(grants).await?;
        } else {
            self.repo
                .push(ShareGrant {
                    owner: owner.to_string(),
                    path: resolved.clone(),
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
                .await?;
        }

        // Sync to DB so that SQL permission checks (can_read / can_write) reflect
        // this grant.  If the file or either user is not yet in the DB the insert
        // is skipped silently — the JSON store remains the authoritative source
        // for legacy paths until migration is complete.
        let storage_path = if resolved.starts_with('/') {
            resolved.clone()
        } else {
            format!("/{}", resolved)
        };
        let db_row = self
            .db
            .connection
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT f.id AS file_id, u_grantor.id AS grantor_uuid, u_grantee.id AS grantee_uuid
                   FROM files f
                   JOIN users u_owner  ON u_owner.id  = f.owner_id
                   JOIN users u_grantor ON u_grantor.username = $1
                   JOIN users u_grantee ON u_grantee.username = $2
                   WHERE u_owner.username = $3 AND f.storage_path = $4 AND f.is_deleted = FALSE"#,
                vec![
                    granted_by.into(),
                    grantee.into(),
                    owner.into(),
                    storage_path.into(),
                ],
            ))
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(row) = db_row {
            let file_id_str: String = row
                .try_get::<String>("", "file_id")
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grantor_uuid_str: String = row
                .try_get::<String>("", "grantor_uuid")
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grantee_uuid_str: String = row
                .try_get::<String>("", "grantee_uuid")
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let file_id = uuid::Uuid::parse_str(&file_id_str)
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grantor_uuid = uuid::Uuid::parse_str(&grantor_uuid_str)
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grantee_uuid = uuid::Uuid::parse_str(&grantee_uuid_str)
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let db_grant = DbShareGrant {
                id: uuid::Uuid::new_v4(),
                file_id,
                granted_by: grantor_uuid,
                granted_to: grantee_uuid,
                can_read,
                can_write,
                can_reshare,
                max_reads: remaining_reads.map(|r| r as i64),
                expires_at: expires_at
                    .and_then(|ts| chrono::DateTime::from_timestamp(ts as i64, 0)),
                granted_at: chrono::Utc::now(),
            };
            self.create_grant_usecase
                .execute(&db_grant)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        Ok(())
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

    pub async fn list_my_links(
        &self,
        owner_id: uuid::Uuid,
    ) -> Result<Vec<DbShareLink>, DomainError> {
        self.list_links_usecase
            .execute(owner_id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    pub async fn list_my_grants(
        &self,
        owner_id: uuid::Uuid,
    ) -> Result<Vec<DbShareGrant>, DomainError> {
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
        let rows = self.db.connection.query_all(
            Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT f.storage_path, u_owner.username as owner_username, u_grantor.username as grantor_username, p.can_read, p.can_write, p.is_expired, p.reads_remaining
                 FROM v_effective_permissions p
                 JOIN files f ON f.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f.owner_id
                 JOIN users u_grantee ON u_grantee.id = p.user_id
                 LEFT JOIN share_grants g ON g.id = p.grant_id
                 LEFT JOIN users u_grantor ON u_grantor.id = g.granted_by
                 WHERE u_grantee.username = $1 AND p.source = 'grant' AND p.is_valid = TRUE"#,
                vec![grantee_username.into()],
            )
        )
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(|r| ShareGrant {
                owner: r
                    .try_get::<String>("", "owner_username")
                    .unwrap_or_default(),
                path: r
                    .try_get::<String>("", "storage_path")
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string(),
                grantee: grantee_username.to_string(),
                can_read: r.try_get::<bool>("", "can_read").unwrap_or(false),
                can_write: r.try_get::<bool>("", "can_write").unwrap_or(false),
                can_download: r.try_get::<bool>("", "can_read").unwrap_or(false),
                remaining_reads: r
                    .try_get::<Option<i64>>("", "reads_remaining")
                    .unwrap_or(None)
                    .map(|v| v as u64),
                remaining_writes: None,
                remaining_downloads: None,
                can_reshare: false,
                granted_by: r
                    .try_get::<Option<String>>("", "grantor_username")
                    .unwrap_or_else(|_| None)
                    .unwrap_or_else(|| {
                        r.try_get::<String>("", "owner_username")
                            .unwrap_or_default()
                    }),
                expires_at: None,
            })
            .collect()
    }

    pub async fn owners_shared_with(&self, grantee_username: &str) -> Vec<String> {
        let rows = self
            .db
            .connection
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT DISTINCT u_owner.username
                 FROM v_effective_permissions p
                 JOIN files f ON f.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f.owner_id
                 JOIN users u_grantee ON u_grantee.id = p.user_id
                 WHERE u_grantee.username = $1 AND p.source = 'grant' AND p.is_valid = TRUE"#,
                vec![grantee_username.into()],
            ))
            .await
            .unwrap_or_default();

        rows.into_iter()
            .map(|r| r.try_get_by_index::<String>(0).unwrap_or_default())
            .collect()
    }

    pub async fn can_read(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let storage_path = if owner_rel_path.starts_with('/') {
            owner_rel_path.to_string()
        } else {
            format!("/{}", owner_rel_path)
        };

        let row = self
            .db
            .connection
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT 1 FROM v_effective_permissions p
                 JOIN files f_shared ON f_shared.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f_shared.owner_id
                 JOIN users u_actor ON u_actor.id = p.user_id
                 WHERE u_actor.username = $1
                   AND u_owner.username = $2
                   AND p.can_read = TRUE
                   AND p.is_valid = TRUE
                   AND (
                       f_shared.storage_path = $3
                       OR ($3 || '/') LIKE (f_shared.storage_path || '/%')
                   )
                 LIMIT 1"#,
                vec![
                    actor.into(),
                    owner.into(),
                    storage_path.clone().into(),
                    storage_path.clone().into(),
                ],
            ))
            .await
            .unwrap_or_default();

        row.is_some()
    }

    pub async fn can_discover(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let storage_path = if owner_rel_path.starts_with('/') {
            owner_rel_path.to_string()
        } else {
            format!("/{}", owner_rel_path)
        };

        let row = self
            .db
            .connection
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT 1 FROM v_effective_permissions p
                 JOIN files f_shared ON f_shared.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f_shared.owner_id
                 JOIN users u_actor ON u_actor.id = p.user_id
                 WHERE u_actor.username = $1
                   AND u_owner.username = $2
                   AND p.can_read = TRUE
                   AND p.is_valid = TRUE
                   AND (
                       f_shared.storage_path = $3
                       OR ($3 || '/') LIKE (f_shared.storage_path || '/%')
                       OR (f_shared.storage_path || '/') LIKE ($3 || '/%')
                   )
                 LIMIT 1"#,
                vec![
                    actor.into(),
                    owner.into(),
                    storage_path.clone().into(),
                    storage_path.clone().into(),
                    storage_path.clone().into(),
                ],
            ))
            .await
            .unwrap_or_default();

        row.is_some()
    }

    pub async fn can_write(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        let storage_path = if owner_rel_path.starts_with('/') {
            owner_rel_path.to_string()
        } else {
            format!("/{}", owner_rel_path)
        };

        let row = self
            .db
            .connection
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT 1 FROM v_effective_permissions p
                 JOIN files f_shared ON f_shared.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f_shared.owner_id
                 JOIN users u_actor ON u_actor.id = p.user_id
                 WHERE u_actor.username = $1
                   AND u_owner.username = $2
                   AND p.can_write = TRUE
                   AND p.is_valid = TRUE
                   AND (
                       f_shared.storage_path = $3
                       OR ($3 || '/') LIKE (f_shared.storage_path || '/%')
                   )
                 LIMIT 1"#,
                vec![
                    actor.into(),
                    owner.into(),
                    storage_path.clone().into(),
                    storage_path.clone().into(),
                ],
            ))
            .await
            .unwrap_or_default();

        row.is_some()
    }

    pub async fn can_download(&self, actor: &str, owner: &str, owner_rel_path: &str) -> bool {
        if actor == owner {
            return true;
        }
        // Download requires read access; the DB check enforces validity and expiry.
        if !self.can_read(actor, owner, owner_rel_path).await {
            return false;
        }
        // The DB schema has no dedicated can_download column, so we look up the
        // explicit flag from the JSON grant store.
        let path_norm = owner_rel_path.trim_start_matches('/');
        let grants = self.repo.all().await;
        grants.iter().any(|g| {
            g.owner == owner && g.grantee == actor && g.can_download && {
                let stored = g.path.trim_start_matches('/');
                stored == path_norm || path_norm.starts_with(&format!("{}/", stored))
            }
        })
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
        let storage_path = if owner_rel_path.starts_with('/') {
            owner_rel_path.to_string()
        } else {
            format!("/{}", owner_rel_path)
        };
        let row = self.db.connection.query_one(
            Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT p.file_id, p.grant_id FROM v_effective_permissions p
                 JOIN files f ON f.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f.owner_id
                 JOIN users u_actor ON u_actor.id = p.user_id
                 WHERE u_actor.username = $1 AND u_owner.username = $2 AND f.storage_path = $3 AND p.can_read = TRUE AND p.is_valid = TRUE"#,
                vec![actor.into(), owner.into(), storage_path.into()],
            )
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(r) = row {
            let file_id: String = r
                .try_get::<String>("", "file_id")
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grant_id: Option<String> = r
                .try_get::<Option<String>>("", "grant_id")
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let actor_model = users::Entity::find()
                .filter(users::Column::Username.eq(actor))
                .one(&self.db.connection)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?
                .ok_or(DomainError::Internal("Actor not found".to_string()))?;
            let actor_id: String = actor_model.id;

            let active_model = access_log::ActiveModel {
                file_id: Set(file_id),
                accessed_by: Set(Some(actor_id)),
                share_link_id: Set(None),
                grant_id: Set(grant_id),
                action: Set(Some("read".to_string())),
                accessed_at: Set(chrono::Utc::now().into()),
                ip_address: Set(None),
                user_agent: Set(None),
                bytes_transferred: Set(None),
                ..Default::default()
            };
            access_log::Entity::insert(active_model)
                .exec(&self.db.connection)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn consume_write(
        &self,
        actor: &str,
        owner: &str,
        owner_rel_path: &str,
    ) -> Result<(), DomainError> {
        if actor == owner {
            return Ok(());
        }
        let storage_path = if owner_rel_path.starts_with('/') {
            owner_rel_path.to_string()
        } else {
            format!("/{}", owner_rel_path)
        };
        let row = self.db.connection.query_one(
            Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT p.file_id, p.grant_id FROM v_effective_permissions p
                 JOIN files f ON f.id = p.file_id
                 JOIN users u_owner ON u_owner.id = f.owner_id
                 JOIN users u_actor ON u_actor.id = p.user_id
                 WHERE u_actor.username = $1 AND u_owner.username = $2 AND f.storage_path = $3 AND p.can_write = TRUE AND p.is_valid = TRUE"#,
                vec![actor.into(), owner.into(), storage_path.into()],
            )
        )
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        if let Some(r) = row {
            let file_id: String = r
                .try_get::<String>("", "file_id")
                .map_err(|e| DomainError::Internal(e.to_string()))?;
            let grant_id: Option<String> = r
                .try_get::<Option<String>>("", "grant_id")
                .map_err(|e| DomainError::Internal(e.to_string()))?;

            let actor_model = users::Entity::find()
                .filter(users::Column::Username.eq(actor))
                .one(&self.db.connection)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?
                .ok_or(DomainError::Internal("Actor not found".to_string()))?;
            let actor_id: String = actor_model.id;

            let active_model = access_log::ActiveModel {
                file_id: Set(file_id),
                accessed_by: Set(Some(actor_id)),
                share_link_id: Set(None),
                grant_id: Set(grant_id),
                action: Set(Some("write".to_string())),
                accessed_at: Set(chrono::Utc::now().into()),
                ip_address: Set(None),
                user_agent: Set(None),
                bytes_transferred: Set(None),
                ..Default::default()
            };
            access_log::Entity::insert(active_model)
                .exec(&self.db.connection)
                .await
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }

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
        false
    }
}
