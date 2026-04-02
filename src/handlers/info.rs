use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use crate::application::file_service::FileService;
use crate::domain::user::User;
use std::sync::Arc;
use tokio::io::BufReader;
use tracing::error;

fn make_user(ctx: &Context) -> User {
    User { username: ctx.user().username.clone(), password_hash: String::new() }
}

pub struct SystHandler;
#[async_trait]
impl Handler for SystHandler {
    fn command(&self) -> &'static str { "SYST" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        ctx.write_line("215 UNIX Type: L8");
        Ok(())
    }
}

pub struct FeatHandler;
#[async_trait]
impl Handler for FeatHandler {
    fn command(&self) -> &'static str { "FEAT" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        // Multi-line response: must end with "211 End"
        ctx.write_line("211-Features:");
        ctx.write_line(" PASV");
        ctx.write_line(" SIZE");
        ctx.write_line(" MLST");
        ctx.write_line(" UTF8");
        ctx.write_line("211 End");
        Ok(())
    }
}

pub struct TypeHandler;
#[async_trait]
impl Handler for TypeHandler {
    fn command(&self) -> &'static str { "TYPE" }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        ctx.write_line("200 Command okay.");
        Ok(())
    }
}

pub struct NoopHandler;
#[async_trait]
impl Handler for NoopHandler {
    fn command(&self) -> &'static str { "NOOP" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        ctx.write_line("200 OK.");
        Ok(())
    }
}

pub struct QuitHandler;
#[async_trait]
impl Handler for QuitHandler {
    fn command(&self) -> &'static str { "QUIT" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        ctx.write_line("221 Goodbye.");
        Ok(())
    }
}

// ── MLST ─────────────────────────────────────────────────────────────────────
// Minimal RFC 3659-style facts (type + size). Modified time is not tracked by this server.
pub struct MlstHandler { files: Arc<FileService> }
impl MlstHandler {
    pub fn new(files: Arc<FileService>) -> Self { Self { files } }
}

#[async_trait]
impl Handler for MlstHandler {
    fn command(&self) -> &'static str { "MLST" }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let target = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.stat(&user, &cwd, target).await {
            Ok(Some((size, is_dir))) => {
                let kind = if is_dir { "dir" } else { "file" };
                ctx.write_line(&format!("250-{};type={};size={}", target, kind, size));
                ctx.write_line("250 End");
            }
            Ok(None) => ctx.error(550, "Requested action not taken. File unavailable."),
            Err(e) => {
                error!("MLST: {}", e);
                ctx.error(550, "Requested action not taken. File unavailable.");
            }
        }

        Ok(())
    }
}
