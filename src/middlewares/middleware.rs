
use crate::framework::context::Context;
use async_trait::async_trait;

/// Décision d'un middleware.
pub enum MiddlewareResult {
    /// Continue vers le prochain middleware / handler.
    Continue,
    /// Stoppe la chaîne. La réponse d'erreur doit être écrite dans ctx avant.
    Stop,
}

/// Trait Middleware.
///
/// # Exemple — Middleware de logging :
/// ```rust
/// pub struct LoggingMiddleware;
///
/// #[async_trait]
/// impl Middleware for LoggingMiddleware {
///     async fn before(&self, ctx: &mut Context, command: &str) -> MiddlewareResult {
///         info!("[{}] → {}", ctx.peer_addr, command);
///         MiddlewareResult::Continue
///     }
/// }
/// ```
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Appelé AVANT le handler.
    async fn before(&self, ctx: &mut Context, command: &str) -> MiddlewareResult;
}
