use crate::infrastructure::database::models::DbAccessLog;
use crate::infrastructure::database::Database;
use anyhow::Result;

pub struct AccessLogRepository {
    db: Database,
}

impl AccessLogRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, log: &DbAccessLog) -> Result<()> {
        let file_str = log.file_id.to_string();
        let accessed_str = log.accessed_by.map(|u| u.to_string());
        let link_str = log.share_link_id.map(|u| u.to_string());
        let grant_str = log.grant_id.map(|u| u.to_string());

        sqlx::query(
            "INSERT INTO access_log (file_id, accessed_by, share_link_id, grant_id, action, accessed_at, ip_address, user_agent, bytes_transferred) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&file_str)
        .bind(&accessed_str)
        .bind(&link_str)
        .bind(&grant_str)
        .bind(&log.action)
        .bind(log.accessed_at)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(log.bytes_transferred)
        .execute(&*self.db.pool)
        .await?;
        Ok(())
    }
}
