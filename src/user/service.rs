use crate::common::error::DomainError;
use crate::database::domain::DbUser;
use crate::database::user_usecases::{CreateUserUseCase, FindUserByEmailUseCase, FindUserUseCase};
use crate::user::domain::User;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    find_user_by_email: Arc<FindUserByEmailUseCase>,
    find_user_by_username: Arc<FindUserUseCase>,
    create_user: Arc<CreateUserUseCase>,
}

impl AuthService {
    pub fn new(
        find_user_by_email: Arc<FindUserByEmailUseCase>,
        find_user_by_username: Arc<FindUserUseCase>,
        create_user: Arc<CreateUserUseCase>,
    ) -> Self {
        Self {
            find_user_by_email,
            find_user_by_username,
            create_user,
        }
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<User, DomainError> {
        let hash = Self::hash_password(password);
        let normalized_email = email.replace(' ', "");
        match self.find_user_by_email.execute(&normalized_email).await {
            Ok(Some(db_user)) if db_user.password_hash == hash => Ok(User {
                id: db_user.id,
                username: db_user.username,
                password_hash: db_user.password_hash,
                email: db_user.email,
                first_name: db_user.first_name,
                last_name: db_user.last_name,
                birth_date: db_user.birth_date,
                location: db_user.location,
            }),
            _ => Err(DomainError::InvalidCredentials),
        }
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        match self.find_user_by_username.execute(username).await {
            Ok(Some(db_user)) => Ok(Some(User {
                id: db_user.id,
                username: db_user.username,
                password_hash: db_user.password_hash,
                email: db_user.email,
                first_name: db_user.first_name,
                last_name: db_user.last_name,
                birth_date: db_user.birth_date,
                location: db_user.location,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(DomainError::Internal(e.to_string())),
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
        first_name: Option<String>,
        last_name: Option<String>,
        birth_date: Option<String>,
        location: Option<String>,
    ) -> Result<(), DomainError> {
        let hash = Self::hash_password(password);
        let normalized_username = username.replace(' ', "");
        let normalized_email = email.replace(' ', "");
        
        let dev_user = DbUser {
            id: Uuid::new_v4(),
            username: normalized_username,
            password_hash: hash,
            email: normalized_email,
            first_name,
            last_name,
            birth_date,
            location,
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
