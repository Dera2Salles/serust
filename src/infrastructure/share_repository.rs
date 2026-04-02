use crate::domain::{error::DomainError, share::ShareGrant};
use serde_json;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

pub struct ShareRepository {
    grants: RwLock<Vec<ShareGrant>>,
    db_path: PathBuf,
}

impl ShareRepository {
    pub async fn new(db_path: impl Into<PathBuf>) -> Self {
        let db_path = db_path.into();
        let grants = Self::load_from_disk(&db_path).await.unwrap_or_default();
        Self {
            grants: RwLock::new(grants),
            db_path,
        }
    }

    async fn load_from_disk(path: &PathBuf) -> Option<Vec<ShareGrant>> {
        let content = fs::read_to_string(path).await.ok()?;
        serde_json::from_str(&content).ok()
    }

    async fn persist(&self) -> Result<(), DomainError> {
        let grants = self.grants.read().await;
        let json = serde_json::to_string_pretty(&*grants)
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        fs::write(&self.db_path, json).await?;
        Ok(())
    }

    pub async fn all(&self) -> Vec<ShareGrant> {
        self.grants.read().await.clone()
    }

    pub async fn replace_all(&self, new_grants: Vec<ShareGrant>) -> Result<(), DomainError> {
        {
            let mut w = self.grants.write().await;
            *w = new_grants;
        }
        self.persist().await
    }

    pub async fn push(&self, grant: ShareGrant) -> Result<(), DomainError> {
        {
            let mut w = self.grants.write().await;
            w.push(grant);
        }
        self.persist().await
    }
}

