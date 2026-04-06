use crate::application::file_service::FileService;
use crate::domain::user::User;
use crate::domain::permission::PermissionChecker;
use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::BufReader;

pub struct DirHandler;
#[async_trait]
impl Handler for DirHandler {
    fn command(&self) -> &'static str { "PWD" }
    fn requires_auth(&self) -> bool { true }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        ctx.write_line(&format!("257 \"{}\"", ctx.cwd));
        Ok(())
    }
}

pub struct CwdHandler { files: Arc<FileService> }
impl CwdHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for CwdHandler {
    fn command(&self) -> &'static str { "CWD" }
    fn requires_auth(&self) -> bool { true }
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

        let path = args[0];
        let new_cwd = PermissionChecker::resolve_path(&ctx.cwd, path);
        let new_cwd_abs = if new_cwd.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", new_cwd)
        };

        let username = ctx.user().username.clone();
        let user = User { username, password_hash: String::new() };
        let exists = if new_cwd.is_empty() {
            true 
        } else {
            self.files.dir_exists(&user, "/", &new_cwd).await
        };

        if exists {
            ctx.cwd = new_cwd_abs;
            ctx.write_line("250 Directory successfully changed.");
        } else {
            ctx.error(550, "No such file or directory.");
        }
        Ok(())
    }
}

pub struct CdupHandler;
#[async_trait]
impl Handler for CdupHandler {
    fn command(&self) -> &'static str { "CDUP" }
    fn requires_auth(&self) -> bool { true }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let new_cwd = PermissionChecker::resolve_path(&ctx.cwd, "..");
        ctx.cwd = if new_cwd.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", new_cwd)
        };
        ctx.write_line("250 Directory successfully changed.");
        Ok(())
    }
}
