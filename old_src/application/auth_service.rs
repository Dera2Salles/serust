use crate::domain::{error::DomainError, user::User};
use crate::infrastructure::user_repository::UserRepository;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct AuthService {
    user_repo: Arc<UserRepository>,
}

impl AuthService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
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
            Some(user) if user.password_hash == hash => Ok(user),
            Some(_) => Err(DomainError::InvalidCredentials),
            None => Err(DomainError::InvalidCredentials),
        }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<(), DomainError> {
        let hash = Self::hash_password(password);
        let user = User::new(username, hash);
        self.user_repo.save(user).await
    }
}
