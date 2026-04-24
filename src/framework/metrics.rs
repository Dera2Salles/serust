use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::info;
use dashmap::DashMap;
use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SessionInfo {
    pub peer_addr: String,
    pub connected_at: DateTime<Utc>,
    pub last_command: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug)]
pub struct Metrics {
    pub total_connections: AtomicU64,
    pub total_commands: AtomicU64,
    pub total_errors: AtomicU64,
    pub bytes_uploaded: AtomicU64,
    pub bytes_downloaded: AtomicU64,
    pub active_sessions: DashMap<SocketAddr, SessionInfo>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            total_commands: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            bytes_uploaded: AtomicU64::new(0),
            bytes_downloaded: AtomicU64::new(0),
            active_sessions: DashMap::new(),
        }
    }
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn connection_opened(&self, addr: SocketAddr) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        self.active_sessions.insert(addr, SessionInfo {
            peer_addr: addr.to_string(),
            connected_at: Utc::now(),
            last_command: None,
            username: None,
        });
    }

    pub fn connection_closed(&self, addr: SocketAddr) {
        self.active_sessions.remove(&addr);
    }

    pub fn command_received(&self, addr: SocketAddr, command: String) {
        self.total_commands.fetch_add(1, Ordering::Relaxed);
        if let Some(mut session) = self.active_sessions.get_mut(&addr) {
            session.last_command = Some(command);
        }
    }

    pub fn user_authenticated(&self, addr: SocketAddr, username: String) {
        if let Some(mut session) = self.active_sessions.get_mut(&addr) {
            session.username = Some(username);
        }
    }

    pub fn error_occurred(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_connections: self.total_connections.load(Ordering::Relaxed),
            active_connections: self.active_sessions.len() as u64,
            total_commands: self.total_commands.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            bytes_uploaded: self.bytes_uploaded.load(Ordering::Relaxed),
            bytes_downloaded: self.bytes_downloaded.load(Ordering::Relaxed),
            sessions: self.active_sessions.iter().map(|r| r.value().clone()).collect(),
        }
    }

    pub fn log_snapshot(&self) {
        let s = self.snapshot();
        info!(
            "METRICS | connexions: {} actives / {} total | cmds: {} | erreurs: {} | up: {} B | down: {} B",
            s.active_connections,
            s.total_connections,
            s.total_commands,
            s.total_errors,
            s.bytes_uploaded,
            s.bytes_downloaded,
        );
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_commands: u64,
    pub total_errors: u64,
    pub bytes_uploaded: u64,
    pub bytes_downloaded: u64,
    pub sessions: Vec<SessionInfo>,
}
