use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub id: String,
    pub peer_addr: String,
    pub connected_at: DateTime<Utc>,
    pub last_command: Option<String>,
    pub username: Option<String>,
}

pub struct SessionRegistry {
    sessions: DashMap<String, ActiveSession>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn add_session(&self, session: ActiveSession) {
        self.sessions.insert(session.id.clone(), session);
    }

    pub fn remove_session(&self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn update_last_command(&self, id: &str, command: String, username: Option<String>) {
        if let Some(mut session) = self.sessions.get_mut(id) {
            session.last_command = Some(command);
            if username.is_some() {
                session.username = username;
            }
        }
    }

    pub fn get_all_sessions(&self) -> Vec<ActiveSession> {
        self.sessions.iter().map(|r| r.value().clone()).collect()
    }
}

pub type SharedSessionRegistry = Arc<SessionRegistry>;
