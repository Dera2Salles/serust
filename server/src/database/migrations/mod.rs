pub mod m0000001_create_table; 
pub mod m0000002_ai_chat;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m0000001_create_table::Migration),
            Box::new(m0000002_ai_chat::Migration),
        ]
    }
}
