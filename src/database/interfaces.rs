use crate::database::domain::{DbAccessLog, DbFileMetadata, DbShareGrant, DbShareLink, DbUser};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IUserRepository: Send + Sync {
    async fn create(&self, user: &DbUser) -> Result<()>;
    async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>>;
}

#[async_trait]
pub trait IFileDatabaseRepository: Send + Sync {
    async fn create(&self, file: &DbFileMetadata) -> Result<()>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbFileMetadata>>;
}

#[async_trait]
pub trait IShareDatabaseRepository: Send + Sync {
    async fn create_link(&self, link: &DbShareLink) -> Result<()>;
    async fn create_grant(&self, grant: &DbShareGrant) -> Result<()>;
}

#[async_trait]
pub trait IAccessLogRepository: Send + Sync {
    async fn create(&self, log: &DbAccessLog) -> Result<()>;
}
