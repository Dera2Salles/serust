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

pub struct CompressHandler {
    files: Arc<FileService>,
}
impl CompressHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for CompressHandler {
    fn command(&self) -> &'static str {
        "COMP"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.len() < 2 {
            ctx.error(501, "Syntax error in parameters or arguments. Use: COMP <format> <path>");
            return Ok(());
        }

        let format = args[0];
        let filename = args[1];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.compress(&user, &cwd, filename, format).await {
            Ok(archive_name) => {
                ctx.write_line(&format!("200 Archive created: {}", archive_name));
            }
            Err(e) => {
                ctx.error(550, &format!("Compression failed: {}", e));
            }
        }
        Ok(())
    }
}

pub struct DecompressHandler {
    files: Arc<FileService>,
}
impl DecompressHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for DecompressHandler {
    fn command(&self) -> &'static str {
        "DECO"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments. Use: DECO <path>");
            return Ok(());
        }

        let filename = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.decompress(&user, &cwd, filename).await {
            Ok(_) => {
                ctx.write_line("200 Decompression successful.");
            }
            Err(e) => {
                ctx.error(550, &format!("Decompression failed: {}", e));
            }
        }
        Ok(())
    }
}
