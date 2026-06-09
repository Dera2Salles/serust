use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "read_counters")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub share_link_id: Option<String>,
    #[sea_orm(unique)]
    pub grant_id: Option<String>,
    pub read_count: i64,
    pub last_read_at: Option<DateTimeWithTimeZone>,
    pub is_exhausted: bool,
    pub refreshed_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::share_grants::Entity",
        from = "Column::GrantId",
        to = "super::share_grants::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ShareGrants,
    #[sea_orm(
        belongs_to = "super::share_links::Entity",
        from = "Column::ShareLinkId",
        to = "super::share_links::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ShareLinks,
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

impl ActiveModelBehavior for ActiveModel {}
