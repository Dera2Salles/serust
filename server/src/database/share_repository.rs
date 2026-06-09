use crate::database::domain::{DbShareGrant, DbShareLink};
use crate::database::entities::{prelude::*, share_links, share_grants};
use crate::database::interfaces::IShareDatabaseRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct ShareRepository {
    db: Database,
}

impl ShareRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn model_to_link(model: share_links::Model) -> DbShareLink {
        DbShareLink {
            id: Uuid::parse_str(&model.id).unwrap_or_else(|_| Uuid::nil()),
            file_id: Uuid::parse_str(&model.file_id).unwrap_or_else(|_| Uuid::nil()),
            created_by: Uuid::parse_str(&model.created_by).unwrap_or_else(|_| Uuid::nil()),
            token: model.token,
            label: model.label,
            can_read: model.can_read.unwrap_or(true),
            can_write: model.can_write.unwrap_or(false),
            can_reshare: model.can_reshare.unwrap_or(false),
            max_reads: model.max_reads,
            expires_at: model.expires_at.map(|dt| dt.into()),
            password_hash: model.password_hash,
            is_active: model.is_active.unwrap_or(true),
        }
    }

    fn model_to_grant(model: share_grants::Model) -> DbShareGrant {
        DbShareGrant {
            id: Uuid::parse_str(&model.id).unwrap_or_else(|_| Uuid::nil()),
            file_id: Uuid::parse_str(&model.file_id).unwrap_or_else(|_| Uuid::nil()),
            granted_by: Uuid::parse_str(&model.granted_by).unwrap_or_else(|_| Uuid::nil()),
            granted_to: Uuid::parse_str(&model.granted_to).unwrap_or_else(|_| Uuid::nil()),
            can_read: model.can_read,
            can_write: model.can_write,
            can_reshare: model.can_reshare,
            max_reads: model.max_reads,
            expires_at: model.expires_at.map(|dt| dt.into()),
            granted_at: model.granted_at.into(),
        }
    }
}

#[async_trait]
impl IShareDatabaseRepository for ShareRepository {
    async fn create_link(&self, link: &DbShareLink) -> Result<()> {
        let active_model = share_links::ActiveModel {
            id: Set(link.id.to_string()),
            file_id: Set(link.file_id.to_string()),
            created_by: Set(link.created_by.to_string()),
            token: Set(link.token.clone()),
            label: Set(link.label.clone()),
            can_read: Set(Some(link.can_read)),
            can_write: Set(Some(link.can_write)),
            can_reshare: Set(Some(link.can_reshare)),
            max_reads: Set(link.max_reads),
            expires_at: Set(link.expires_at.map(|dt| dt.into())),
            password_hash: Set(link.password_hash.clone()),
            is_active: Set(Some(link.is_active)),
        };

        ShareLinks::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }

    async fn create_grant(&self, grant: &DbShareGrant) -> Result<()> {
        let active_model = share_grants::ActiveModel {
            id: Set(grant.id.to_string()),
            file_id: Set(grant.file_id.to_string()),
            granted_by: Set(grant.granted_by.to_string()),
            granted_to: Set(grant.granted_to.to_string()),
            can_read: Set(grant.can_read),
            can_write: Set(grant.can_write),
            can_reshare: Set(grant.can_reshare),
            max_reads: Set(grant.max_reads),
            expires_at: Set(grant.expires_at.map(|dt| dt.into())),
            granted_at: Set(grant.granted_at.into()),
        };

        ShareGrants::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }

    async fn find_link_by_token(&self, token: &str) -> Result<Option<DbShareLink>> {
        let model = ShareLinks::find()
            .filter(share_links::Column::Token.eq(token))
            .filter(share_links::Column::IsActive.eq(Some(true)))
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_link))
    }

    async fn list_links_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbShareLink>> {
        let models = ShareLinks::find()
            .filter(share_links::Column::CreatedBy.eq(owner_id.to_string()))
            .all(&self.db.connection)
            .await?;

        Ok(models.into_iter().map(Self::model_to_link).collect())
    }

    async fn list_grants_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbShareGrant>> {
        let models = ShareGrants::find()
            .filter(share_grants::Column::GrantedBy.eq(owner_id.to_string()))
            .all(&self.db.connection)
            .await?;

        Ok(models.into_iter().map(Self::model_to_grant).collect())
    }

    async fn delete_link(&self, id: Uuid) -> Result<()> {
        ShareLinks::delete_by_id(id.to_string())
            .exec(&self.db.connection)
            .await?;
        Ok(())
    }

    async fn delete_grant(&self, id: Uuid) -> Result<()> {
        ShareGrants::delete_by_id(id.to_string())
            .exec(&self.db.connection)
            .await?;
        Ok(())
    }
}
