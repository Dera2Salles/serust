use crate::database::domain::{DbShareGrant, DbShareLink};
use crate::database::interfaces::IShareDatabaseRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct ShareRepository {
    db: Database,
}

impl ShareRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IShareDatabaseRepository for ShareRepository {
    async fn create_link(&self, link: &DbShareLink) -> Result<()> {
        let id_str = link.id.to_string();
        let file_str = link.file_id.to_string();
        let created_str = link.created_by.to_string();

        sqlx::query(
            "INSERT INTO share_links (id, file_id, created_by, token, label, can_read, can_write, can_reshare, max_reads, expires_at, password_hash, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&file_str)
        .bind(&created_str)
        .bind(&link.token)
        .bind(&link.label)
        .bind(link.can_read)
        .bind(link.can_write)
        .bind(link.can_reshare)
        .bind(link.max_reads)
        .bind(link.expires_at)
        .bind(&link.password_hash)
        .bind(link.is_active)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn create_grant(&self, grant: &DbShareGrant) -> Result<()> {
        let id_str = grant.id.to_string();
        let file_str = grant.file_id.to_string();
        let granted_by_str = grant.granted_by.to_string();
        let granted_to_str = grant.granted_to.to_string();

        sqlx::query(
            "INSERT INTO share_grants (id, file_id, granted_by, granted_to, can_read, can_write, can_reshare, max_reads, expires_at, granted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&file_str)
        .bind(&granted_by_str)
        .bind(&granted_to_str)
        .bind(grant.can_read)
        .bind(grant.can_write)
        .bind(grant.can_reshare)
        .bind(grant.max_reads)
        .bind(grant.expires_at)
        .bind(grant.granted_at)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn find_link_by_token(&self, token: &str) -> Result<Option<DbShareLink>> {
        let row = sqlx::query(
            "SELECT id, file_id, created_by, token, label, can_read, can_write, can_reshare, max_reads, expires_at, password_hash, is_active FROM share_links WHERE token = ? AND is_active = 1"
        )
        .bind(token)
        .fetch_optional(&*self.db.pool)
        .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let id_str: String = r.try_get("id")?;
            let file_str: String = r.try_get("file_id")?;
            let created_str: String = r.try_get("created_by")?;
            Ok(Some(DbShareLink {
                id: uuid::Uuid::parse_str(&id_str)?,
                file_id: uuid::Uuid::parse_str(&file_str)?,
                created_by: uuid::Uuid::parse_str(&created_str)?,
                token: r.try_get("token")?,
                label: r.try_get("label")?,
                can_read: r.try_get("can_read")?,
                can_write: r.try_get("can_write")?,
                can_reshare: r.try_get("can_reshare")?,
                max_reads: r.try_get("max_reads")?,
                expires_at: r.try_get("expires_at")?,
                password_hash: r.try_get("password_hash")?,
                is_active: r.try_get("is_active")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_links_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbShareLink>> {
        let owner_str = owner_id.to_string();
        let rows = sqlx::query(
            "SELECT id, file_id, created_by, token, label, can_read, can_write, can_reshare, max_reads, expires_at, password_hash, is_active FROM share_links WHERE created_by = ?"
        )
        .bind(&owner_str)
        .fetch_all(&*self.db.pool)
        .await?;

        let mut results = Vec::new();
        for r in rows {
            use sqlx::Row;
            let id_str: String = r.try_get("id")?;
            let file_str: String = r.try_get("file_id")?;
            let created_str: String = r.try_get("created_by")?;
            results.push(DbShareLink {
                id: uuid::Uuid::parse_str(&id_str)?,
                file_id: uuid::Uuid::parse_str(&file_str)?,
                created_by: uuid::Uuid::parse_str(&created_str)?,
                token: r.try_get("token")?,
                label: r.try_get("label")?,
                can_read: r.try_get("can_read")?,
                can_write: r.try_get("can_write")?,
                can_reshare: r.try_get("can_reshare")?,
                max_reads: r.try_get("max_reads")?,
                expires_at: r.try_get("expires_at")?,
                password_hash: r.try_get("password_hash")?,
                is_active: r.try_get("is_active")?,
            });
        }
        Ok(results)
    }

    async fn list_grants_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbShareGrant>> {
        let owner_str = owner_id.to_string();
        let rows = sqlx::query(
            "SELECT id, file_id, granted_by, granted_to, can_read, can_write, can_reshare, max_reads, expires_at, granted_at FROM share_grants WHERE granted_by = ?"
        )
        .bind(&owner_str)
        .fetch_all(&*self.db.pool)
        .await?;

        let mut results = Vec::new();
        for r in rows {
            use sqlx::Row;
            let id_str: String = r.try_get("id")?;
            let file_str: String = r.try_get("file_id")?;
            let granted_by_str: String = r.try_get("granted_by")?;
            let granted_to_str: String = r.try_get("granted_to")?;
            results.push(DbShareGrant {
                id: uuid::Uuid::parse_str(&id_str)?,
                file_id: uuid::Uuid::parse_str(&file_str)?,
                granted_by: uuid::Uuid::parse_str(&granted_by_str)?,
                granted_to: uuid::Uuid::parse_str(&granted_to_str)?,
                can_read: r.try_get("can_read")?,
                can_write: r.try_get("can_write")?,
                can_reshare: r.try_get("can_reshare")?,
                max_reads: r.try_get("max_reads")?,
                expires_at: r.try_get("expires_at")?,
                granted_at: r.try_get("granted_at")?,
            });
        }
        Ok(results)
    }

    async fn delete_link(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM share_links WHERE id = ?")
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }

    async fn delete_grant(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM share_grants WHERE id = ?")
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }
}
