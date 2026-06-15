pub mod access_log_repository;
pub mod admin_repository;
pub mod analytics_repository;
pub mod ai_repository;
pub mod domain;
pub mod file_repository;
pub mod file_usecases;
pub mod interfaces;
pub mod log_usecases;
pub mod share_repository;
pub mod share_usecases;
pub mod user_repository;
pub mod user_usecases;
pub mod entities;
pub mod migrations;

use anyhow::Result;
use ::sea_orm::{Database as SeaDatabase, DatabaseConnection};
use sea_orm_migration::prelude::*;
use crate::database::migrations::Migrator;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let connection = SeaDatabase::connect(database_url).await?;

        Migrator::up(&connection, None).await?;

        Ok(Self { connection })
    }
}
