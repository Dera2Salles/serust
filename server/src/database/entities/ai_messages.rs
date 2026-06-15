use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub chat_id: String,
    pub role: String, // 'user' or 'model'
    #[sea_orm(column_type = "Text")]
    pub text: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ai_chats::Entity",
        from = "Column::ChatId",
        to = "super::ai_chats::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AiChats,
}

impl Related<super::ai_chats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AiChats.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
