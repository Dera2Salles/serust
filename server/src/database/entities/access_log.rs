use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "access_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub file_id: String,
    pub accessed_by: Option<String>,
    pub share_link_id: Option<String>,
    pub grant_id: Option<String>,
    pub action: Option<String>,
    pub accessed_at: DateTimeWithTimeZone,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub bytes_transferred: Option<i64>,
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
        belongs_to = "super::share_grants::Entity",
        from = "Column::GrantId",
        to = "super::share_grants::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ShareGrants,
    #[sea_orm(
        belongs_to = "super::share_links::Entity",
        from = "Column::ShareLinkId",
        to = "super::share_links::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ShareLinks,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AccessedBy",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Users,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl Related<super::share_grants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareGrants.def()
    }
}

impl Related<super::share_links::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareLinks.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
