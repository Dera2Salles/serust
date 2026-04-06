use crate::framework::context::Context;
use crate::framework::middleware::{Middleware, MiddlewareResult};
use async_trait::async_trait;

pub struct AuthMiddleware;

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, ctx: &mut Context, command: &str) -> MiddlewareResult {
        if command == "USER"
            || command == "PASS"
            || command == "QUIT"
            || command == "FEAT"
            || command == "SYST"
            || command == "AUTH"
        {
            return MiddlewareResult::Continue;
        }
        if !ctx.is_authenticated() {
            ctx.error(530, "Not logged in.");
            return MiddlewareResult::Stop;
        }
        MiddlewareResult::Continue
    }
}
