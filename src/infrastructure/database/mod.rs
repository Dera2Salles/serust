pub mod models;
pub mod repositories;

use sqlx::{sqlite::{SqlitePoolOptions, SqliteConnectOptions}, SqlitePool, Executor};
use std::sync::Arc;
use std::str::FromStr;
use anyhow::Result;

#[derive(Clone)]
pub struct Database {
    pub pool: Arc<SqlitePool>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options).await?;

        let schema = include_str!("schema.sql");
        pool.execute(schema).await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}
