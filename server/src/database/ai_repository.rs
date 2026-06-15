use crate::database::entities::{prelude::*, ai_chats, ai_messages};
use crate::database::Database;
use anyhow::Result;
use sea_orm::*;
use chrono::Utc;

#[derive(Clone)]
pub struct AiRepository {
    db: Database,
}

impl AiRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_chat(&self, id: &str, user_id: &str, title: &str) -> Result<ai_chats::Model> {
        let now = Utc::now().fixed_offset();
        let active_model = ai_chats::ActiveModel {
            id: Set(id.to_string()),
            user_id: Set(user_id.to_string()),
            title: Set(title.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = AiChats::insert(active_model)
            .exec_with_returning(&self.db.connection)
            .await?;
        Ok(result)
    }

    pub async fn list_chats(&self, user_id: &str) -> Result<Vec<ai_chats::Model>> {
        let chats = AiChats::find()
            .filter(ai_chats::Column::UserId.eq(user_id))
            .order_by_desc(ai_chats::Column::UpdatedAt)
            .all(&self.db.connection)
            .await?;
        Ok(chats)
    }

    pub async fn delete_chat(&self, id: &str) -> Result<()> {
        AiChats::delete_by_id(id.to_string())
            .exec(&self.db.connection)
            .await?;
        Ok(())
    }

    pub async fn create_message(&self, id: &str, chat_id: &str, role: &str, text: &str) -> Result<ai_messages::Model> {
        let now = Utc::now().fixed_offset();
        let active_model = ai_messages::ActiveModel {
            id: Set(id.to_string()),
            chat_id: Set(chat_id.to_string()),
            role: Set(role.to_string()),
            text: Set(text.to_string()),
            created_at: Set(now),
        };

        let result = AiMessages::insert(active_model)
            .exec_with_returning(&self.db.connection)
            .await?;

        // Update the chat's updated_at field
        let chat_active = ai_chats::ActiveModel {
            id: Set(chat_id.to_string()),
            updated_at: Set(now),
            ..Default::default()
        };
        let _ = AiChats::update(chat_active).exec(&self.db.connection).await;

        Ok(result)
    }

    pub async fn list_messages(&self, chat_id: &str) -> Result<Vec<ai_messages::Model>> {
        let messages = AiMessages::find()
            .filter(ai_messages::Column::ChatId.eq(chat_id))
            .order_by_asc(ai_messages::Column::CreatedAt)
            .all(&self.db.connection)
            .await?;
        Ok(messages)
    }
}
