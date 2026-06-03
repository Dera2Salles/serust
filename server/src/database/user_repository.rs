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
            "INSERT INTO users (id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.birth_date)
        .bind(&user.location)
        .bind(user.created_at)
        .bind(user.storage_quota_bytes)
        .bind(user.is_active)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<DbUser>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active FROM users WHERE username = ?"
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
                first_name: r.try_get("first_name")?,
                last_name: r.try_get("last_name")?,
                birth_date: r.try_get("birth_date")?,
                location: r.try_get("location")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<DbUser>> {
        let row = sqlx::query(
            "SELECT id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(&*self.db.pool)
        .await?;

        if let Some(r) = row {
            let id_str: String = r.try_get("id")?;
            Ok(Some(DbUser {
                id: Uuid::parse_str(&id_str)?,
                username: r.try_get("username")?,
                password_hash: r.try_get("password_hash")?,
                email: r.try_get("email")?,
                first_name: r.try_get("first_name")?,
                last_name: r.try_get("last_name")?,
                birth_date: r.try_get("birth_date")?,
                location: r.try_get("location")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DbUser>> {
        let id_str = id.to_string();
        let row = sqlx::query(
            "SELECT id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active FROM users WHERE id = ?"
        )
        .bind(id_str)
        .fetch_optional(&*self.db.pool)
        .await?;

        if let Some(r) = row {
            let id_str: String = r.try_get("id")?;
            Ok(Some(DbUser {
                id: Uuid::parse_str(&id_str)?,
                username: r.try_get("username")?,
                password_hash: r.try_get("password_hash")?,
                email: r.try_get("email")?,
                first_name: r.try_get("first_name")?,
                last_name: r.try_get("last_name")?,
                birth_date: r.try_get("birth_date")?,
                location: r.try_get("location")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn search_users(&self, query: &str) -> Result<Vec<DbUser>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            "SELECT id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active FROM users WHERE username LIKE ? OR email LIKE ? LIMIT 10"
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&*self.db.pool)
        .await?;

        let mut users = Vec::new();
        for r in rows {
            let id_str: String = r.try_get("id")?;
            users.push(DbUser {
                id: Uuid::parse_str(&id_str)?,
                username: r.try_get("username")?,
                password_hash: r.try_get("password_hash")?,
                email: r.try_get("email")?,
                first_name: r.try_get("first_name")?,
                last_name: r.try_get("last_name")?,
                birth_date: r.try_get("birth_date")?,
                location: r.try_get("location")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            });
        }
        Ok(users)
    }

    async fn update(&self, user: &DbUser) -> Result<()> {
        let id_str = user.id.to_string();
        sqlx::query(
            "UPDATE users SET username = ?, password_hash = ?, email = ?, first_name = ?, last_name = ?, birth_date = ?, location = ?, storage_quota_bytes = ?, is_active = ? WHERE id = ?"
        )
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.birth_date)
        .bind(&user.location)
        .bind(user.storage_quota_bytes)
        .bind(user.is_active)
        .bind(&id_str)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(&id_str)
            .execute(&*self.db.pool)
            .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<DbUser>> {
        let rows = sqlx::query(
            "SELECT id, username, password_hash, email, first_name, last_name, birth_date, location, created_at, storage_quota_bytes, is_active FROM users"
        )
        .fetch_all(&*self.db.pool)
        .await?;

        let mut users = Vec::new();
        for r in rows {
            let id_str: String = r.try_get("id")?;
            users.push(DbUser {
                id: Uuid::parse_str(&id_str)?,
                username: r.try_get("username")?,
                password_hash: r.try_get("password_hash")?,
                email: r.try_get("email")?,
                first_name: r.try_get("first_name")?,
                last_name: r.try_get("last_name")?,
                birth_date: r.try_get("birth_date")?,
                location: r.try_get("location")?,
                created_at: r.try_get("created_at")?,
                storage_quota_bytes: r.try_get("storage_quota_bytes")?,
                is_active: r.try_get("is_active")?,
            });
        }
        Ok(users)
    }
}
