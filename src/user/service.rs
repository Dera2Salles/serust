use crate::common::error::DomainError;
use crate::database::domain::DbUser;
use crate::database::{IUserRepository, UserDatabaseRepository as DbUserRepository};
use crate::user::domain::User;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    user_repo: Arc<DbUserRepository>,
}

impl AuthService {
    pub fn new(user_repo: Arc<DbUserRepository>) -> Self {
        Self { user_repo }
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<User, DomainError> {
        let hash = Self::hash_password(password);
        match self.user_repo.find_by_username(username).await {
            Ok(Some(db_user)) if db_user.password_hash == hash => Ok(User {
                username: db_user.username,
                password_hash: db_user.password_hash,
            }),
            _ => Err(DomainError::InvalidCredentials),
        }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<(), DomainError> {
        let hash = Self::hash_password(password);
        let dev_user = DbUser {
            id: Uuid::new_v4(),
            username: username.to_string(),
            password_hash: hash,
            email: "".to_string(),
            created_at: Utc::now(),
            storage_quota_bytes: 0,
            is_active: true,
        };
        self.user_repo
            .create(&dev_user)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}
