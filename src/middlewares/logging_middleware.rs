
use crate::framework::context::Context;
use crate::middlewares::middleware::{Middleware, MiddlewareResult};
use async_trait::async_trait;
use tracing::info;

pub struct LoggingMiddleware;

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, ctx: &mut Context, command: &str) -> MiddlewareResult {
        let user = ctx
            .extensions
            .get::<crate::framework::context::AuthenticatedUser>()
            .map(|u| u.username.as_str())
            .unwrap_or("anonymous");

        info!("[{}] {} → {}", ctx.peer_addr, user, command);
        MiddlewareResult::Continue
    }
}
