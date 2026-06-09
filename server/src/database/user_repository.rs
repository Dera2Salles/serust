use crate::database::domain::DbUser;
use crate::database::entities::{prelude::*, users};
use crate::database::interfaces::IUserRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn model_to_db_user(model: users::Model) -> DbUser {
        DbUser {
            id: Uuid::parse_str(&model.id).unwrap_or_else(|_| Uuid::nil()),
            username: model.username,
            password_hash: model.password_hash,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
            birth_date: model.birth_date,
            location: model.location,
            profile_pic_path: model.profile_pic_path,
            created_at: model.created_at.into(),
            storage_quota_bytes: model.storage_quota_bytes,
            is_active: model.is_active,
        }
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn create(&self, user: &DbUser) -> Result<()> {
        let active_model = users::ActiveModel {
            id: Set(user.id.to_string()),
            username: Set(user.username.clone()),
            password_hash: Set(user.password_hash.clone()),
            email: Set(user.email.clone()),
            first_name: Set(user.first_name.clone()),
            last_name: Set(user.last_name.clone()),
            birth_date: Set(user.birth_date.clone()),
            location: Set(user.location.clone()),
            profile_pic_path: Set(user.profile_pic_path.clone()),
            created_at: Set(user.created_at.into()),
            storage_quota_bytes: Set(user.storage_quota_bytes),
            is_active: Set(user.is_active),
        };

        Users::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>> {
        let model = Users::find()
            .filter(users::Column::Username.eq(username))
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_db_user))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<DbUser>> {
        let model = Users::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_db_user))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbUser>> {
        let model = Users::find_by_id(id.to_string())
            .one(&self.db.connection)
            .await?;

        Ok(model.map(Self::model_to_db_user))
    }

    async fn search_users(&self, query: &str) -> Result<Vec<DbUser>> {
        let pattern = format!("%{}%", query);
        let models = Users::find()
            .filter(
                users::Column::Username
                    .like(&pattern)
                    .or(users::Column::Email.like(&pattern)),
            )
            .limit(10)
            .all(&self.db.connection)
            .await?;

        Ok(models.into_iter().map(Self::model_to_db_user).collect())
    }

    async fn update(&self, user: &DbUser) -> Result<()> {
        let model = Users::find_by_id(user.id.to_string())
            .one(&self.db.connection)
            .await?;

        if let Some(model) = model {
            let mut active_model: users::ActiveModel = model.into();
            active_model.username = Set(user.username.clone());
            active_model.password_hash = Set(user.password_hash.clone());
            active_model.email = Set(user.email.clone());
            active_model.first_name = Set(user.first_name.clone());
            active_model.last_name = Set(user.last_name.clone());
            active_model.birth_date = Set(user.birth_date.clone());
            active_model.location = Set(user.location.clone());
            active_model.profile_pic_path = Set(user.profile_pic_path.clone());
            active_model.storage_quota_bytes = Set(user.storage_quota_bytes);
            active_model.is_active = Set(user.is_active);

            active_model.update(&self.db.connection).await?;
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        Users::delete_by_id(id.to_string())
            .exec(&self.db.connection)
            .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<DbUser>> {
        let models = Users::find().all(&self.db.connection).await?;
        Ok(models.into_iter().map(Self::model_to_db_user).collect())
    }
}
