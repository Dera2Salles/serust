use crate::database::models::{DbShareGrant, DbShareLink};
use crate::database::Database;
use anyhow::Result;

pub struct ShareRepository {
    db: Database,
}

impl ShareRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_link(&self, link: &DbShareLink) -> Result<()> {
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

    pub async fn create_grant(&self, grant: &DbShareGrant) -> Result<()> {
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
}
