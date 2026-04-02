use crate::application::file_service::FileService;
use crate::domain::user::User;
use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::error;

async fn get_data_stream(ctx: &mut Context) -> anyhow::Result<tokio::net::TcpStream> {
    if let Some(listener) = ctx.data_listener.take() {
        let (stream, _) = listener.accept().await?;
        Ok(stream)
    } else if let Some(addr) = ctx.data_address.take() {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(stream)
    } else {
        Err(anyhow::anyhow!("No data connection established."))
    }
}

pub struct StorHandler { files: Arc<FileService> }
impl StorHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for StorHandler {
    fn command(&self) -> &'static str { "STOR" }
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
        
        ctx.write_line("150 Ok to send data.");
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(_) => {
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        let mut data = Vec::new();
        if let Err(e) = data_stream.read_to_end(&mut data).await {
            error!("Data read failed: {}", e);
            ctx.error(426, "Connection closed; transfer aborted.");
            return Ok(());
        }
        
        let username = ctx.user().username.clone();
        let user = User { username, password_hash: String::new() };
        
        let full_path = if ctx.cwd == "/" {
            filename.to_string()
        } else {
            format!("{}/{}", ctx.cwd, filename)
        };
        
        if let Err(e) = self.files.upload(&user, &full_path, data.len() as u64, data).await {
            error!("Upload failed: {}", e);
            ctx.error(451, "Requested action aborted. Local error in processing.");
        } else {
            ctx.write_line("226 Transfer complete.");
        }
        Ok(())
    }
}

pub struct RetrHandler { files: Arc<FileService> }
impl RetrHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for RetrHandler {
    fn command(&self) -> &'static str { "RETR" }
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
        
        let username = ctx.user().username.clone();
        let user = User { username, password_hash: String::new() };
        
        let full_path = if ctx.cwd == "/" {
            filename.to_string()
        } else {
            format!("{}/{}", ctx.cwd, filename)
        };
        
        let data = match self.files.download(&user, &full_path).await {
            Ok(d) => d,
            Err(e) => {
                error!("Download lookup failed: {}", e);
                ctx.error(550, "Requested action not taken. File unavailable.");
                return Ok(());
            }
        };

        ctx.write_line(&format!("150 Opening BINARY mode data connection for {} ({} bytes).", filename, data.len()));
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(_) => {
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        if let Err(e) = data_stream.write_all(&data).await {
            error!("Data write failed: {}", e);
            ctx.error(426, "Connection closed; transfer aborted.");
        } else {
            ctx.write_line("226 Transfer complete.");
        }
        Ok(())
    }
}

pub struct ListDirHandler { files: Arc<FileService> }
impl ListDirHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for ListDirHandler {
    fn command(&self) -> &'static str { "LIST" }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let username = ctx.user().username.clone();
        let user = User { username, password_hash: String::new() };
        
        let files = match self.files.list(&user).await {
            Ok(f) => f,
            Err(e) => {
                error!("List failed: {}", e);
                ctx.error(550, "Requested action not taken. File unavailable.");
                return Ok(());
            }
        };

        ctx.write_line("150 Here comes the directory listing.");
        writer.write_all(&ctx.response).await?;
        ctx.response.clear();

        let mut data_stream = match get_data_stream(ctx).await {
            Ok(s) => s,
            Err(_) => {
                ctx.error(425, "Can't open data connection.");
                return Ok(());
            }
        };

        for f in &files {
            // Very simplified FTP LIST output format
            let line = format!("-rw-r--r-- 1 ftp ftp 1024 Jan 01 00:00 {}\r\n", f);
            if let Err(_) = data_stream.write_all(line.as_bytes()).await {
                ctx.error(426, "Connection closed; transfer aborted.");
                return Ok(());
            }
        }
        
        ctx.write_line("226 Directory send OK.");
        Ok(())
    }
}

pub struct MkdHandler { files: Arc<FileService> }
impl MkdHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for MkdHandler {
    fn command(&self) -> &'static str { "MKD" }
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
        let username = ctx.user().username.clone();
        let user = crate::domain::user::User { username, password_hash: String::new() };
        match self.files.mkdir(&user, dirname).await {
            Ok(_) => ctx.write_line(&format!("257 \"{}\" directory created.", dirname)),
            Err(e) => { error!("MKD failed: {}", e); ctx.error(550, "Failed to create directory."); }
        }
        Ok(())
    }
}

pub struct RmdHandler { files: Arc<FileService> }
impl RmdHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for RmdHandler {
    fn command(&self) -> &'static str { "RMD" }
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
        let username = ctx.user().username.clone();
        let user = crate::domain::user::User { username, password_hash: String::new() };
        match self.files.rmdir(&user, dirname).await {
            Ok(_) => ctx.write_line("250 Directory removed."),
            Err(e) => { error!("RMD failed: {}", e); ctx.error(550, "Failed to remove directory."); }
        }
        Ok(())
    }
}

pub struct DeleHandler { files: Arc<FileService> }
impl DeleHandler { pub fn new(files: Arc<FileService>) -> Self { Self { files } } }

#[async_trait]
impl Handler for DeleHandler {
    fn command(&self) -> &'static str { "DELE" }
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
        let username = ctx.user().username.clone();
        let user = crate::domain::user::User { username, password_hash: String::new() };
        match self.files.delete_file(&user, filename).await {
            Ok(_) => ctx.write_line("250 File deleted."),
            Err(e) => { error!("DELE failed: {}", e); ctx.error(550, "File not found or deletion failed."); }
        }
        Ok(())
    }
}
