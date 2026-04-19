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

pub struct UpdateFileUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl UpdateFileUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, file: &DbFileMetadata) -> Result<()> {
        self.repo.update(file).await
    }
}

pub struct FindFileByPathUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl FindFileByPathUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, path: &str) -> Result<Option<DbFileMetadata>> {
        self.repo.find_by_storage_path(path).await
    }
}
