
use crate::framework::context::Context;
use crate::middlewares::middleware::{Middleware, MiddlewareResult};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::warn;

pub struct RateLimitMiddleware {
    /// Nombre max de commandes par fenêtre.
    max_requests: usize,
    /// Taille de la fenêtre glissante.
    window: Duration,
    /// Historique par IP : timestamps des requêtes récentes.
    buckets: DashMap<IpAddr, Mutex<VecDeque<Instant>>>,
}

impl RateLimitMiddleware {
    /// Ex : RateLimitMiddleware::new(120, Duration::from_secs(60))
    /// = max 120 commandes par minute par IP
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            buckets: DashMap::new(),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn before(&self, ctx: &mut Context, _command: &str) -> MiddlewareResult {
        let ip = ctx.peer_addr.ip();
        let now = Instant::now();
        let window = self.window;
        let max = self.max_requests;

        let entry = self.buckets.entry(ip).or_insert_with(|| Mutex::new(VecDeque::new()));
        let mut deque = entry.lock().unwrap();

        while deque.front().map_or(false, |t: &Instant| now.duration_since(*t) > window) {
            deque.pop_front();
        }

        if deque.len() >= max {
            warn!("Rate limit atteint pour {} ({} req/{}s)", ip, max, window.as_secs());
            ctx.error(421, "Service not available, closing control connection.");
            return MiddlewareResult::Stop;
        }

        deque.push_back(now);
        MiddlewareResult::Continue
    }
}
