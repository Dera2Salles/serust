use crate::application::auth_service::AuthService;
use crate::framework::{
    context::{AuthenticatedUser, Context},
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::BufReader;
use tracing::warn;

// Stocke temporairement le username entre USER et PASS
struct PendingAuth {
    username: String,
}

pub struct UserHandler;

#[async_trait]
impl Handler for UserHandler {
    fn command(&self) -> &'static str { "USER" }
    fn requires_auth(&self) -> bool { false }

    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _reader: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }
        ctx.extensions.set(PendingAuth { username: args[0].to_string() });
        ctx.write_line("331 User name okay, need password.");
        Ok(())
    }
}

pub struct PassHandler {
    auth: Arc<AuthService>,
}

impl PassHandler {
    pub fn new(auth: Arc<AuthService>) -> Self {
        Self { auth }
    }
}

#[async_trait]
impl Handler for PassHandler {
    fn command(&self) -> &'static str { "PASS" }
    fn requires_auth(&self) -> bool { false }

    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _reader: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let username = match ctx.extensions.get::<PendingAuth>() {
            Some(p) => p.username.clone(),
            None => {
                ctx.error(503, "Bad sequence of commands. Send USER first.");
                return Ok(());
            }
        };

        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let password = args[0];

        match self.auth.login(&username, password).await {
            Ok(user) => {
                ctx.extensions.set(AuthenticatedUser {
                    username: user.username.clone(),
                });
                ctx.write_line("230 User logged in, proceed.");
            }
            Err(e) => {
                warn!("Échec login '{}': {}", username, e);
                ctx.error(530, "Not logged in.");
            }
        }
        Ok(())
    }
}
