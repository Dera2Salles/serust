pub mod domain;
pub mod interfaces;
pub mod access_log_repository;
pub mod file_repository;
pub mod share_repository;
pub mod user_repository;

pub use access_log_repository::AccessLogRepository;
pub use file_repository::FileRepository as FileDatabaseRepository;
pub use share_repository::ShareRepository as ShareDatabaseRepository;
pub use user_repository::UserRepository as UserDatabaseRepository;
pub use interfaces::*;


use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Executor, SqlitePool,
};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pub pool: Arc<SqlitePool>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let schema = include_str!("schema.sql");
        pool.execute(schema).await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}
