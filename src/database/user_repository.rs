use crate::database::domain::DbUser;
use crate::database::interfaces::IUserRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn create(&self, user: &DbUser) -> Result<()> {
        let id_str = user.id.to_string();
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, email, created_at, storage_quota_bytes, is_active) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(user.created_at)
        .bind(user.storage_quota_bytes)
        .bind(user.is_active)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, email, created_at, storage_quota_bytes, is_active FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&*self.db.pool)
        .await?;

        if let Some(r) = row {
            let id_str: String = r.try_get("id")?;
            Ok(Some(DbUser {
                id: Uuid::parse_str(&id_str)?,
                username: r.try_get("username")?,
                password_hash: r.try_get("password_hash")?,
                email: r.try_get("email")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            }))
        } else {
            Ok(None)
        }
    }
}
