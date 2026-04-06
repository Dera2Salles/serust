use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Share,
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Read => "read".to_string(),
            Action::Write => "write".to_string(),
            Action::Share => "share".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
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

impl AccessLog {
    pub fn new_download_event(file_id: Uuid, user_id: Option<Uuid>) -> Self {
        Self {
            id: 0,
            file_id,
            accessed_by: user_id,
            share_link_id: None,
            grant_id: None,
            action: Action::Read.to_string(),
            accessed_at: Utc::now(),
            ip_address: None,
            user_agent: None,
            bytes_transferred: None,
        }
    }
}
