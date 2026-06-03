use crate::database::domain::DbAdmin;
use crate::database::interfaces::IAdminRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminRepository {
    db: Database,
}

impl AdminRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IAdminRepository for AdminRepository {
    async fn create(&self, admin: &DbAdmin) -> Result<()> {
        let user_id_str = admin.user_id.to_string();
        sqlx::query(
            "INSERT INTO admins (user_id, access_level, last_action_at) VALUES (?, ?, ?)"
        )
        .bind(&user_id_str)
        .bind(&admin.access_level)
        .bind(admin.last_action_at)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<DbAdmin>> {
        let user_id_str = user_id.to_string();
        let row = sqlx::query(
            "SELECT user_id, access_level, last_action_at FROM admins WHERE user_id = ?"
        )
        .bind(&user_id_str)
        .fetch_optional(&*self.db.pool)
        .await?;

        if let Some(r) = row {
            let id_str: String = r.try_get("user_id")?;
            Ok(Some(DbAdmin {
                user_id: Uuid::parse_str(&id_str)?,
                access_level: r.try_get("access_level")?,
                last_action_at: r.try_get("last_action_at")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_last_action(&self, user_id: Uuid) -> Result<()> {
        let user_id_str = user_id.to_string();
        let now = chrono::Utc::now();
        sqlx::query(
            "UPDATE admins SET last_action_at = ? WHERE user_id = ?"
        )
        .bind(now)
        .bind(&user_id_str)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }

    async fn is_admin(&self, user_id: Uuid) -> Result<bool> {
        let user_id_str = user_id.to_string();
        let row = sqlx::query(
            "SELECT 1 FROM admins WHERE user_id = ?"
        )
        .bind(&user_id_str)
        .fetch_optional(&*self.db.pool)
        .await?;
        Ok(row.is_some())
    }

    async fn list_all(&self) -> Result<Vec<DbAdmin>> {
        let rows = sqlx::query(
            "SELECT user_id, access_level, last_action_at FROM admins"
        )
        .fetch_all(&*self.db.pool)
        .await?;

        let mut admins = Vec::new();
        for r in rows {
            let id_str: String = r.try_get("user_id")?;
            admins.push(DbAdmin {
                user_id: Uuid::parse_str(&id_str)?,
                access_level: r.try_get("access_level")?,
                last_action_at: r.try_get("last_action_at")?,
            });
        }
        Ok(admins)
    }
}
