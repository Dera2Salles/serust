
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Default)]
pub struct Metrics {
    pub total_connections: AtomicU64,
    pub active_connections: AtomicU64,
    pub total_commands: AtomicU64,
    pub total_errors: AtomicU64,
    pub bytes_uploaded: AtomicU64,
    pub bytes_downloaded: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn connection_opened(&self) {
        self.total_connections.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn command_received(&self) {
        self.total_commands.fetch_add(1, Ordering::Relaxed);
    }

    pub fn error_occurred(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }



    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_connections: self.total_connections.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            total_commands: self.total_commands.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            bytes_uploaded: self.bytes_uploaded.load(Ordering::Relaxed),
            bytes_downloaded: self.bytes_downloaded.load(Ordering::Relaxed),
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

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_commands: u64,
    pub total_errors: u64,
    pub bytes_uploaded: u64,
    pub bytes_downloaded: u64,
}
