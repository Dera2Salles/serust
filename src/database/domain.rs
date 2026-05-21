use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub storage_quota_bytes: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbAdmin {
    pub user_id: Uuid,
    pub access_level: String,
    pub last_action_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbFileMetadata {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub filename: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbShareLink {
    pub id: Uuid,
    pub file_id: Uuid,
    pub created_by: Uuid,
    pub token: String,
    pub label: Option<String>,
    pub can_read: bool,
    pub can_write: bool,
    pub can_reshare: bool,
    pub max_reads: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbShareGrant {
    pub id: Uuid,
    pub file_id: Uuid,
    pub granted_by: Uuid,
    pub granted_to: Uuid,
    pub can_read: bool,
    pub can_write: bool,
    pub can_reshare: bool,
    pub max_reads: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbAccessLog {
    pub id: i64,
    pub file_id: Uuid,
    pub accessed_by: Option<Uuid>,
    pub share_link_id: Option<Uuid>,
    pub grant_id: Option<Uuid>,
    pub action: String,
    pub accessed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub bytes_transferred: Option<i64>,
}
