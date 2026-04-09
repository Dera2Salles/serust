use crate::database::domain::DbAccessLog;
use crate::database::interfaces::IAccessLogRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct LogAccessUseCase {
    repo: Arc<dyn IAccessLogRepository>,
}

impl LogAccessUseCase {
    pub fn new(repo: Arc<dyn IAccessLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, log: &DbAccessLog) -> Result<()> {
        self.repo.create(log).await
    }
}
