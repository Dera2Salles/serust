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
