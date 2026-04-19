use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessStat {
    pub filename: String,
    pub storage_path: String,
    pub access_count: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthByDay {
    pub date: String,
    pub bytes_total: i64,
    pub access_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub action: String,
    pub filename: String,
    pub accessed_at: String,
    pub ip_address: Option<String>,
    pub bytes_transferred: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_accesses: i64,
    pub total_bytes_transferred: i64,
    pub unique_files_accessed: i64,
    pub unique_ips: i64,
    pub top_files: Vec<FileAccessStat>,
    pub bandwidth_by_day: Vec<BandwidthByDay>,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Clone)]
pub struct AnalyticsRepository {
    db: Database,
}

impl AnalyticsRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_summary(&self, username: &str) -> Result<AnalyticsSummary> {
        let top_files = self.top_files(username, 10).await?;
        let bandwidth_by_day = self.bandwidth_by_day(username, 30).await?;
        let recent_activity = self.recent_activity(username, 20).await?;

        // Global counters for this user
        let totals = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS total_accesses,
                COALESCE(SUM(al.bytes_transferred), 0) AS total_bytes,
                COUNT(DISTINCT al.file_id) AS unique_files,
                COUNT(DISTINCT al.ip_address) AS unique_ips
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&*self.db.pool)
        .await?;

        Ok(AnalyticsSummary {
            total_accesses: totals.try_get("total_accesses").unwrap_or(0),
            total_bytes_transferred: totals.try_get("total_bytes").unwrap_or(0),
            unique_files_accessed: totals.try_get("unique_files").unwrap_or(0),
            unique_ips: totals.try_get("unique_ips").unwrap_or(0),
            top_files,
            bandwidth_by_day,
            recent_activity,
        })
    }

    pub async fn top_files(&self, username: &str, limit: i64) -> Result<Vec<FileAccessStat>> {
        let rows = sqlx::query(
            r#"
            SELECT
                f.filename,
                f.storage_path,
                COUNT(*) AS access_count,
                COALESCE(SUM(al.bytes_transferred), 0) AS total_bytes
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = ?
            GROUP BY f.id
            ORDER BY access_count DESC
            LIMIT ?
            "#,
        )
        .bind(username)
        .bind(limit)
        .fetch_all(&*self.db.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| FileAccessStat {
                filename: r.try_get("filename").unwrap_or_default(),
                storage_path: r.try_get("storage_path").unwrap_or_default(),
                access_count: r.try_get("access_count").unwrap_or(0),
                total_bytes: r.try_get("total_bytes").unwrap_or(0),
            })
            .collect())
    }

    pub async fn bandwidth_by_day(&self, username: &str, days: i64) -> Result<Vec<BandwidthByDay>> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(al.accessed_at) AS date,
                COALESCE(SUM(al.bytes_transferred), 0) AS bytes_total,
                COUNT(*) AS access_count
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = ?
              AND al.accessed_at >= DATE('now', '-' || ? || ' days')
            GROUP BY DATE(al.accessed_at)
            ORDER BY date ASC
            "#,
        )
        .bind(username)
        .bind(days)
        .fetch_all(&*self.db.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| BandwidthByDay {
                date: r.try_get("date").unwrap_or_default(),
                bytes_total: r.try_get("bytes_total").unwrap_or(0),
                access_count: r.try_get("access_count").unwrap_or(0),
            })
            .collect())
    }

    pub async fn recent_activity(&self, username: &str, limit: i64) -> Result<Vec<RecentActivity>> {
        let rows = sqlx::query(
            r#"
            SELECT
                al.action,
                f.filename,
                al.accessed_at,
                al.ip_address,
                al.bytes_transferred
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = ?
            ORDER BY al.accessed_at DESC
            LIMIT ?
            "#,
        )
        .bind(username)
        .bind(limit)
        .fetch_all(&*self.db.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                let accessed_at: chrono::DateTime<chrono::Utc> =
                    r.try_get("accessed_at").unwrap_or_else(|_| chrono::Utc::now());
                RecentActivity {
                    action: r.try_get("action").unwrap_or_default(),
                    filename: r.try_get("filename").unwrap_or_default(),
                    accessed_at: accessed_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    ip_address: r.try_get("ip_address").ok().flatten(),
                    bytes_transferred: r.try_get("bytes_transferred").ok().flatten(),
                }
            })
            .collect())
    }
}
