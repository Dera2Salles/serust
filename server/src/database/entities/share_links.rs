use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "share_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub file_id: String,
    pub created_by: String,
    #[sea_orm(unique)]
    pub token: String,
    pub label: Option<String>,
    pub can_read: Option<bool>,
    pub can_write: Option<bool>,
    pub can_reshare: Option<bool>,
    pub max_reads: Option<i64>,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub password_hash: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Files,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::CreatedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
    #[sea_orm(has_one = "super::read_counters::Entity")]
    ReadCounters,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::read_counters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReadCounters.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
