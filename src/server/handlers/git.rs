use crate::file::service::FileService;
use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use crate::user::domain::User;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::BufReader;

fn make_user(ctx: &Context) -> User {
    User {
        username: ctx.user().username.clone(),
        password_hash: String::new(),
    }
}

pub struct GitHistoryHandler {
    files: Arc<FileService>,
}
impl GitHistoryHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for GitHistoryHandler {
    fn command(&self) -> &'static str {
        "GITH"
    }
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

        let filename = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.git_history(&user, &cwd, filename).await {
            Ok(history) => {
                ctx.write_line("211-History:");
                for (hash, date, msg) in history {
                    ctx.write_line(&format!("{}|{}|{}", hash, date, msg));
                }
                ctx.write_line("211 End");
            }
            Err(e) => {
                ctx.error(550, &format!("Failed to get history: {}", e));
            }
        }
        Ok(())
    }
}

pub struct GitRestoreHandler {
    files: Arc<FileService>,
}
impl GitRestoreHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for GitRestoreHandler {
    fn command(&self) -> &'static str {
        "GITR"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.len() < 2 {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let filename = args[0];
        let hash = args[1];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.git_restore(&user, &cwd, filename, hash).await {
            Ok(_) => {
                ctx.write_line("200 Version restored.");
            }
            Err(e) => {
                ctx.error(550, &format!("Failed to restore version: {}", e));
            }
        }
        Ok(())
    }
}

pub struct GitDiffHandler {
    files: Arc<FileService>,
}
impl GitDiffHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for GitDiffHandler {
    fn command(&self) -> &'static str {
        "GITD"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.len() < 2 {
            ctx.error(501, "Syntax error in parameters or arguments. Use: GITD <path> <hash>");
            return Ok(());
        }

        let filename = args[0];
        let hash = args[1];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.git_diff(&user, &cwd, filename, hash).await {
            Ok(diff) => {
                ctx.write_line("211-Diff:");
                for line in diff.lines() {
                    ctx.write_line(line);
                }
                ctx.write_line("211 End");
            }
            Err(e) => {
                ctx.error(550, &format!("Failed to get diff: {}", e));
            }
        }
        Ok(())
    }
}
