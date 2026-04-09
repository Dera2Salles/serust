use crate::database::domain::DbFileMetadata;
use crate::database::interfaces::IFileDatabaseRepository;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateFileUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl CreateFileUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, file: &DbFileMetadata) -> Result<()> {
        self.repo.create(file).await
    }
}

pub struct FindFileUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl FindFileUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Option<DbFileMetadata>> {
        self.repo.find_by_id(id).await
    }
}
