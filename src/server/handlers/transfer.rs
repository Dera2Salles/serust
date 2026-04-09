use crate::common::error::DomainError;
use crate::file::service::FileService;
use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use crate::user::domain::User;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tracing::error;

async fn get_data_stream(ctx: &mut Context) -> anyhow::Result<tokio::net::TcpStream> {
    if let Some(listener) = ctx.data_listener.take() {
        let result = timeout(Duration::from_secs(30), listener.accept()).await;
        match result {
            Ok(Ok((stream, _))) => Ok(stream),
            Ok(Err(e)) => Err(anyhow::anyhow!("Accept error: {}", e)),
            Err(_) => Err(anyhow::anyhow!("Timeout waiting for data connection")),
        }
    } else if let Some(addr) = ctx.data_address.take() {
        let stream = timeout(
            Duration::from_secs(30),
            tokio::net::TcpStream::connect(addr),
        )
        .await??;
        Ok(stream)
    } else {
        Err(anyhow::anyhow!(
            "No data connection established. Use PASV or PORT first."
        ))
    }
}

fn make_user(ctx: &Context) -> User {
    User {
        username: ctx.user().username.clone(),
        password_hash: String::new(),
    }
}

pub struct StorHandler {
    files: Arc<FileService>,
}
impl StorHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for StorHandler {
    fn command(&self) -> &'static str {
        "STOR"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }
        let filename = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        ctx.write_line("150 Ok to send data.");
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(e) => {
                error!("STOR data connect: {}", e);
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        let mut data = Vec::new();
        if let Err(e) = data_stream.read_to_end(&mut data).await {
            error!("STOR read: {}", e);
            ctx.error(426, "Connection closed; transfer aborted.");
            return Ok(());
        }
        let size = data.len() as u64;

        match self.files.upload(&user, &cwd, filename, size, data).await {
            Ok(_) => ctx.write_line("226 Transfer complete."),
            Err(e) => {
                error!("STOR upload: {}", e);
                ctx.error(451, "Requested action aborted. Local error in processing.");
            }
        }
        Ok(())
    }
}

pub struct RetrHandler {
    files: Arc<FileService>,
}
impl RetrHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for RetrHandler {
    fn command(&self) -> &'static str {
        "RETR"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }
        let filename = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        let data = match self.files.download(&user, &cwd, filename).await {
            Ok(d) => d,
            Err(e) => {
                error!("RETR: {}", e);
                ctx.error(550, "Requested action not taken. File unavailable.");
                return Ok(());
            }
        };

        ctx.write_line(&format!(
            "150 Opening BINARY mode data connection for {} ({} bytes).",
            filename,
            data.len()
        ));
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(e) => {
                error!("RETR data connect: {}", e);
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        if let Err(e) = data_stream.write_all(&data).await {
            error!("RETR write: {}", e);
            ctx.error(426, "Connection closed; transfer aborted.");
        } else {
            ctx.write_line("226 Transfer complete.");
        }
        Ok(())
    }
}

pub struct ListDirHandler {
    files: Arc<FileService>,
}
impl ListDirHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for ListDirHandler {
    fn command(&self) -> &'static str {
        "LIST"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let user = make_user(ctx);
        let target = args.first().copied().unwrap_or(".");
        let cwd = if target == "." || target == "-a" || target == "-la" {
            ctx.cwd.clone()
        } else {
            target.to_string()
        };

        let entries = match self.files.list(&user, &cwd).await {
            Ok(e) => e,
            Err(e) => {
                error!("LIST: {}", e);
                match e {
                    DomainError::PermissionDenied => ctx.error(550, "Permission denied."),
                    _ => ctx.error(550, "Requested action not taken. File unavailable."),
                }
                return Ok(());
            }
        };

        ctx.write_line("150 Here comes the directory listing.");
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(e) => {
                error!("LIST data connect: {}", e);
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        for (name, is_dir) in &entries {
            let type_char = if *is_dir { 'd' } else { '-' };
            let perms = if *is_dir { "rwxr-xr-x" } else { "rw-r--r--" };
            let line = format!(
                "{}{} 1 ftp ftp {:>10} Jan 01 00:00 {}\r\n",
                type_char,
                perms,
                if *is_dir { 4096u64 } else { 0u64 },
                name
            );
            if let Err(_) = data_stream.write_all(line.as_bytes()).await {
                ctx.error(426, "Connection closed; transfer aborted.");
                return Ok(());
            }
        }

        ctx.write_line("226 Directory send OK.");
        Ok(())
    }
}

pub struct NlstHandler {
    files: Arc<FileService>,
}
impl NlstHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for NlstHandler {
    fn command(&self) -> &'static str {
        "NLST"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        let files = match self.files.list(&user, &cwd).await {
            Ok(e) => e,
            Err(e) => {
                error!("NLST: {}", e);
                match e {
                    DomainError::PermissionDenied => ctx.error(550, "Permission denied."),
                    _ => ctx.error(550, "Requested action not taken."),
                }
                return Ok(());
            }
        };

        ctx.write_line("150 Here comes the file list.");
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(e) => {
                error!("NLST data connect: {}", e);
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        for (name, is_dir) in &files {
            if !is_dir {
                let line = format!("{}\r\n", name);
                if let Err(_) = data_stream.write_all(line.as_bytes()).await {
                    ctx.error(426, "Connection closed; transfer aborted.");
                    return Ok(());
                }
            }
        }

        ctx.write_line("226 Directory send OK.");
        Ok(())
    }
}

pub struct MkdHandler {
    files: Arc<FileService>,
}
impl MkdHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for MkdHandler {
    fn command(&self) -> &'static str {
        "MKD"
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
        let dirname = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.mkdir(&user, &cwd, dirname).await {
            Ok(_) => {
                let abs = format!("{}/{}", if cwd == "/" { "" } else { &cwd }, dirname)
                    .replace("//", "/");
                ctx.write_line(&format!("257 \"{}\" directory created.", abs));
            }
            Err(e) => {
                error!("MKD: {}", e);
                ctx.error(550, "Failed to create directory.");
            }
        }
        Ok(())
    }
}

pub struct RmdHandler {
    files: Arc<FileService>,
}
impl RmdHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for RmdHandler {
    fn command(&self) -> &'static str {
        "RMD"
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
        let dirname = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();
        match self.files.rmdir(&user, &cwd, dirname).await {
            Ok(_) => ctx.write_line("250 Directory removed."),
            Err(e) => {
                error!("RMD: {}", e);
                ctx.error(550, "Failed to remove directory.");
            }
        }
        Ok(())
    }
}

pub struct DeleHandler {
    files: Arc<FileService>,
}
impl DeleHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for DeleHandler {
    fn command(&self) -> &'static str {
        "DELE"
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
        match self.files.delete(&user, &cwd, filename).await {
            Ok(_) => ctx.write_line("250 File deleted."),
            Err(e) => {
                error!("DELE: {}", e);
                ctx.error(550, "File not found or deletion failed.");
            }
        }
        Ok(())
    }
}

pub struct RnfrHandler;
#[async_trait]
impl Handler for RnfrHandler {
    fn command(&self) -> &'static str {
        "RNFR"
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
        ctx.extensions.set(RenameFrom(args[0].to_string()));
        ctx.write_line("350 Ready for RNTO.");
        Ok(())
    }
}

#[derive(Clone)]
pub struct RenameFrom(pub String);

pub struct RntoHandler {
    files: Arc<FileService>,
}
impl RntoHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for RntoHandler {
    fn command(&self) -> &'static str {
        "RNTO"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let from = match ctx.extensions.get::<RenameFrom>() {
            Some(r) => r.0.clone(),
            None => {
                ctx.error(503, "Bad sequence of commands. Send RNFR first.");
                return Ok(());
            }
        };
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }
        let to = args[0];
        let user = make_user(ctx);
        let cwd = ctx.cwd.clone();

        match self.files.rename(&user, &cwd, &from, to).await {
            Ok(_) => {
                ctx.write_line("250 Rename successful.");
            }
            Err(e) => {
                error!("RNTO rename: {}", e);
                ctx.error(550, "Rename failed.");
            }
        }
        Ok(())
    }
}

pub struct SizeHandler {
    files: Arc<FileService>,
}
impl SizeHandler {
    pub fn new(files: Arc<FileService>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl Handler for SizeHandler {
    fn command(&self) -> &'static str {
        "SIZE"
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
        match self.files.download(&user, &cwd, filename).await {
            Ok(data) => ctx.write_line(&format!("213 {}", data.len())),
            Err(_) => ctx.error(550, "File not found."),
        }
        Ok(())
    }
}
