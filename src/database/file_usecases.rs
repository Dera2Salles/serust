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

pub struct RenameFileDbUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl RenameFileDbUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid, new_storage_path: &str, new_filename: &str) -> Result<()> {
        self.repo.rename(id, new_storage_path, new_filename).await
    }
}

pub struct SoftDeleteFileDbUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl SoftDeleteFileDbUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<()> {
        self.repo.soft_delete(id).await
    }
}

pub struct RestoreFileDbUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl RestoreFileDbUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<()> {
        self.repo.restore(id).await
    }
}

pub struct FindDeletedFilesDbUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl FindDeletedFilesDbUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, owner_id: Uuid) -> Result<Vec<DbFileMetadata>> {
        self.repo.find_deleted_by_owner(owner_id).await
    }
}

pub struct PermanentDeleteFileDbUseCase {
    repo: Arc<dyn IFileDatabaseRepository>,
}

impl PermanentDeleteFileDbUseCase {
    pub fn new(repo: Arc<dyn IFileDatabaseRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<()> {
        self.repo.delete_permanently(id).await
    }
}
