use crate::database::domain::DbUser;
use crate::database::interfaces::IUserRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct CreateUserUseCase {
    repo: Arc<dyn IUserRepository>,
}

impl CreateUserUseCase {
    pub fn new(repo: Arc<dyn IUserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user: &DbUser) -> Result<()> {
        self.repo.create(user).await
    }
}

pub struct FindUserUseCase {
    repo: Arc<dyn IUserRepository>,
}

impl FindUserUseCase {
    pub fn new(repo: Arc<dyn IUserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, username: &str) -> Result<Option<DbUser>> {
        self.repo.find_by_username(username).await
    }
}
