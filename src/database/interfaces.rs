use crate::database::domain::{DbAccessLog, DbFileMetadata, DbShareGrant, DbShareLink, DbUser, DbAdmin};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IUserRepository: Send + Sync {
    async fn create(&self, user: &DbUser) -> Result<()>;
    async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<DbUser>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbUser>>;
    async fn search_users(&self, query: &str) -> Result<Vec<DbUser>>;
    async fn update(&self, user: &DbUser) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<DbUser>>;
}

#[allow(dead_code)]
#[async_trait]
pub trait IFileDatabaseRepository: Send + Sync {
    async fn create(&self, file: &DbFileMetadata) -> Result<()>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbFileMetadata>>;
    async fn find_by_storage_path(&self, owner_id: Uuid, path: &str) -> Result<Option<DbFileMetadata>>;
    async fn update(&self, file: &DbFileMetadata) -> Result<()>;
    async fn rename(&self, id: Uuid, new_storage_path: &str, new_filename: &str) -> Result<()>;
    async fn soft_delete(&self, id: Uuid) -> Result<()>;
    async fn restore(&self, id: Uuid) -> Result<()>;
    async fn find_deleted_by_owner(&self, owner_id: Uuid) -> Result<Vec<DbFileMetadata>>;
    async fn delete_permanently(&self, id: Uuid) -> Result<()>;
    async fn find_by_parent_path(&self, owner_id: Uuid, parent_path: &str) -> Result<Vec<DbFileMetadata>>;
}

#[async_trait]
pub trait IShareDatabaseRepository: Send + Sync {
    async fn create_link(&self, link: &DbShareLink) -> Result<()>;
    async fn create_grant(&self, grant: &DbShareGrant) -> Result<()>;
    async fn find_link_by_token(&self, token: &str) -> Result<Option<DbShareLink>>;
}

#[async_trait]
pub trait IAccessLogRepository: Send + Sync {
    async fn create(&self, log: &DbAccessLog) -> Result<()>;
}

#[allow(dead_code)]
#[async_trait]
pub trait IAdminRepository: Send + Sync {
    async fn create(&self, admin: &DbAdmin) -> Result<()>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<DbAdmin>>;
    async fn update_last_action(&self, user_id: Uuid) -> Result<()>;
    async fn is_admin(&self, user_id: Uuid) -> Result<bool>;
    async fn list_all(&self) -> Result<Vec<DbAdmin>>;
}

