pub mod m0000001_create_table; // Update this with the actual migration file name

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m0000001_create_table::Migration),
        ]
    }
}
