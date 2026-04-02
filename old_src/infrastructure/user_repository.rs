use crate::domain::{error::DomainError, user::User};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

pub struct UserRepository {
    users: RwLock<HashMap<String, User>>,
    db_path: PathBuf,
}

impl UserRepository {
    pub async fn new(db_path: impl Into<PathBuf>) -> Self {
        let db_path = db_path.into();
        let users = Self::load_from_disk(&db_path).await.unwrap_or_default();
        Self {
            users: RwLock::new(users),
            db_path,
        }
    }

    async fn load_from_disk(path: &PathBuf) -> Option<HashMap<String, User>> {
        let content = fs::read_to_string(path).await.ok()?;
        serde_json::from_str(&content).ok()
    }

    async fn persist(&self) -> Result<(), DomainError> {
        let users = self.users.read().await;
        let json = serde_json::to_string_pretty(&*users)
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        fs::write(&self.db_path, json).await?;
        Ok(())
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        self.users.read().await.get(username).cloned()
    }

    pub async fn save(&self, user: User) -> Result<(), DomainError> {
        {
            let mut users = self.users.write().await;
            users.insert(user.username.clone(), user);
        }
        self.persist().await
    }
}
