use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
