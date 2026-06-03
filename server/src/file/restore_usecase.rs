use crate::common::error::DomainError;
use crate::database::file_usecases::RestoreFileDbUseCase;
use crate::user::domain::User;
use std::sync::Arc;

pub struct RestoreUseCase {
    restore_db_file: Arc<RestoreFileDbUseCase>,
}

impl RestoreUseCase {
    pub fn new(restore_db_file: Arc<RestoreFileDbUseCase>) -> Self {
        Self { restore_db_file }
    }

    pub async fn execute(&self, _user: &User, id: uuid::Uuid) -> Result<(), DomainError> {
        // We restore by ID since it's in the recycle bin and we have its DB ID
        self.restore_db_file
            .execute(id)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}
