use crate::database::Database;
use anyhow::Result;
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct FileAccessStat {
    pub filename: String,
    pub storage_path: String,
    pub access_count: i64,
    pub total_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct BandwidthByDay {
    pub date: String,
    pub bytes_total: i64,
    pub access_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct RecentActivityRaw {
    pub action: String,
    pub filename: String,
    pub accessed_at: chrono::DateTime<chrono::FixedOffset>,
    pub ip_address: Option<String>,
    pub bytes_transferred: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub action: String,
    pub filename: String,
    pub accessed_at: String,
    pub ip_address: Option<String>,
    pub bytes_transferred: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
struct Totals {
    total_accesses: i64,
    total_bytes: i64,
    unique_files: i64,
    unique_ips: i64,
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

        let totals: Option<Totals> = Totals::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            SELECT
                COUNT(*) AS total_accesses,
                COALESCE(SUM(al.bytes_transferred), 0) AS total_bytes,
                COUNT(DISTINCT al.file_id) AS unique_files,
                COUNT(DISTINCT al.ip_address) AS unique_ips
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = $1
            "#,
            vec![username.into()],
        ))
        .one(&self.db.connection)
        .await?;

        let totals = totals.unwrap_or(Totals {
            total_accesses: 0,
            total_bytes: 0,
            unique_files: 0,
            unique_ips: 0,
        });

        Ok(AnalyticsSummary {
            total_accesses: totals.total_accesses,
            total_bytes_transferred: totals.total_bytes,
            unique_files_accessed: totals.unique_files,
            unique_ips: totals.unique_ips,
            top_files,
            bandwidth_by_day,
            recent_activity,
        })
    }

    pub async fn top_files(&self, username: &str, limit: i64) -> Result<Vec<FileAccessStat>> {
        let stats = FileAccessStat::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            SELECT
                f.filename,
                f.storage_path,
                COUNT(*) AS access_count,
                COALESCE(SUM(al.bytes_transferred), 0) AS total_bytes
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = $1
            GROUP BY f.id, f.filename, f.storage_path
            ORDER BY access_count DESC
            LIMIT $2
            "#,
            vec![username.into(), limit.into()],
        ))
        .all(&self.db.connection)
        .await?;

        Ok(stats)
    }

    pub async fn bandwidth_by_day(&self, username: &str, days: i64) -> Result<Vec<BandwidthByDay>> {
        let stats = BandwidthByDay::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
            SELECT
                DATE(al.accessed_at)::text AS date,
                COALESCE(SUM(al.bytes_transferred), 0) AS bytes_total,
                COUNT(*) AS access_count
            FROM access_log al
            JOIN files f ON f.id = al.file_id
            JOIN users u ON u.id = f.owner_id
            WHERE u.username = $1
              AND al.accessed_at >= NOW() - MAKE_INTERVAL(days => $2)
            GROUP BY DATE(al.accessed_at)
            ORDER BY date ASC
            "#,
            vec![username.into(), days.into()],
        ))
        .all(&self.db.connection)
        .await?;

        Ok(stats)
    }

    pub async fn recent_activity(&self, username: &str, limit: i64) -> Result<Vec<RecentActivity>> {
        let rows = RecentActivityRaw::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
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
            WHERE u.username = $1
            ORDER BY al.accessed_at DESC
            LIMIT $2
            "#,
            vec![username.into(), limit.into()],
        ))
        .all(&self.db.connection)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RecentActivity {
                action: r.action,
                filename: r.filename,
                accessed_at: r.accessed_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                ip_address: r.ip_address,
                bytes_transferred: r.bytes_transferred,
            })
            .collect())
    }
}
