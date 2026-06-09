use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub owner_id: String,
    pub filename: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub is_deleted: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::OwnerId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
    #[sea_orm(has_many = "super::share_links::Entity")]
    ShareLinks,
    #[sea_orm(has_many = "super::share_grants::Entity")]
    ShareGrants,
    #[sea_orm(has_many = "super::access_log::Entity")]
    AccessLog,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::share_links::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareLinks.def()
    }
}

impl Related<super::share_grants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareGrants.def()
    }
}

impl Related<super::access_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccessLog.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
