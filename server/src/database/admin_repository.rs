use crate::database::domain::DbAdmin;
use crate::database::entities::{prelude::*, admins};
use crate::database::interfaces::IAdminRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminRepository {
    db: Database,
}

impl AdminRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn model_to_db_admin(model: admins::Model) -> DbAdmin {
        DbAdmin {
            user_id: Uuid::parse_str(&model.user_id).unwrap_or_else(|_| Uuid::nil()),
            access_level: model.access_level,
            last_action_at: model.last_action_at.map(|dt| dt.into()),
        }
    }
}

#[async_trait]
impl IAdminRepository for AdminRepository {
    async fn create(&self, admin: &DbAdmin) -> Result<()> {
        let active_model = admins::ActiveModel {
            user_id: Set(admin.user_id.to_string()),
            access_level: Set(admin.access_level.clone()),
            last_action_at: Set(admin.last_action_at.map(|dt| dt.into())),
        };

        Admins::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<DbAdmin>> {
        let model = Admins::find_by_id(user_id.to_string())
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_db_admin))
    }

    async fn update_last_action(&self, user_id: Uuid) -> Result<()> {
        let model = Admins::find_by_id(user_id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: admins::ActiveModel = model.into();
            active_model.last_action_at = Set(Some(chrono::Utc::now().into()));
            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn is_admin(&self, user_id: Uuid) -> Result<bool> {
        let model = Admins::find_by_id(user_id.to_string())
            .one(&self.db.connection)
            .await?;
        Ok(model.is_some())
    }

    async fn list_all(&self) -> Result<Vec<DbAdmin>> {
        let models = Admins::find().all(&self.db.connection).await?;
        Ok(models.into_iter().map(Self::model_to_db_admin).collect())
    }
}
