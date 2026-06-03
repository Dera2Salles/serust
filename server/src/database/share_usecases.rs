use crate::database::domain::{DbShareGrant, DbShareLink};
use crate::database::interfaces::IShareDatabaseRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct CreateLinkUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl CreateLinkUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, link: &DbShareLink) -> Result<()> {
        self.repo.create_link(link).await
    }
}

pub struct CreateGrantUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl CreateGrantUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, grant: &DbShareGrant) -> Result<()> {
        self.repo.create_grant(grant).await
    }
}

pub struct ListMyLinksUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl ListMyLinksUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, owner_id: uuid::Uuid) -> Result<Vec<DbShareLink>> {
        self.repo.list_links_by_owner(owner_id).await
    }
}

pub struct ListMyGrantsUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl ListMyGrantsUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, owner_id: uuid::Uuid) -> Result<Vec<DbShareGrant>> {
        self.repo.list_grants_by_owner(owner_id).await
    }
}

pub struct RevokeLinkUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl RevokeLinkUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<()> {
        self.repo.delete_link(id).await
    }
}

pub struct RevokeGrantUseCase {
    repo: Arc<dyn IShareDatabaseRepository>,
}

impl RevokeGrantUseCase {
    pub fn new(repo: Arc<dyn IShareDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<()> {
        self.repo.delete_grant(id).await
    }
}
