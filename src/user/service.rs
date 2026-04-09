use crate::common::error::DomainError;
use crate::database::domain::DbUser;
use crate::database::user_usecases::{CreateUserUseCase, FindUserUseCase};
use crate::user::domain::User;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    find_user: Arc<FindUserUseCase>,
    create_user: Arc<CreateUserUseCase>,
}

impl AuthService {
    pub fn new(find_user: Arc<FindUserUseCase>, create_user: Arc<CreateUserUseCase>) -> Self {
        Self { find_user, create_user }
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<User, DomainError> {
        let hash = Self::hash_password(password);
        match self.find_user.execute(username).await {
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
        self.create_user
            .execute(&dev_user)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))
    }
}
