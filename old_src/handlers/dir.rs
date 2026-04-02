use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
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

pub struct CwdHandler;
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
        if path.starts_with('/') {
            ctx.cwd = path.to_string();
        } else {
            if ctx.cwd.ends_with('/') {
                ctx.cwd = format!("{}{}", ctx.cwd, path);
            } else {
                ctx.cwd = format!("{}/{}", ctx.cwd, path);
            }
        }
        ctx.write_line("250 Directory successfully changed.");
        Ok(())
    }
}
