mod common;
mod database;
mod file;
mod injection;
mod log;
mod mcp;
mod server;
mod share;
mod user;
mod webdav;

use mcp::{
    registry::McpRegistry,
    server::{run_mcp_server, McpServerState},
};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

pub async fn run_server() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::never(".", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("Démarrage du framework TCP...");

    let services = injection::setup_injection().await?;
    let auth_service = services.auth_service;
    let share_service = services.share_service;
    let file_service = services.file_service;
    let db = services.db;


    let mcp_registry = Arc::new(McpRegistry::new(
        Arc::clone(&file_service),
        Arc::new(crate::database::user_repository::UserRepository::new(
            db.clone(),
        )),
        Arc::clone(&share_service),
    ));
    let mcp_state = Arc::new(McpServerState {
        registry: Arc::clone(&mcp_registry),
        file_service: Arc::clone(&file_service),
        auth_service: Arc::clone(&auth_service),
        db: db.clone(),
    });
    tokio::spawn(async move {
        if let Err(e) = run_mcp_server(mcp_state, "0.0.0.0:8081").await {
            tracing::error!("MCP server error: {}", e);
        }
    });
    info!("MCP server spawned on 0.0.0.0:8081");

    let webdav_auth = Arc::clone(&auth_service);
    let webdav_files = Arc::clone(&file_service);
    tokio::spawn(async move {
        let addr: std::net::SocketAddr = "0.0.0.0:8083".parse().unwrap();
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            tracing::info!("WebDAV HTTP server listening on {}", addr);
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    let auth = Arc::clone(&webdav_auth);
                    let files = Arc::clone(&webdav_files);
                    tokio::task::spawn(async move {
                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(
                                io,
                                hyper::service::service_fn(move |req| {
                                    crate::webdav::handler::serve_webdav(
                                        req,
                                        Arc::clone(&auth),
                                        Arc::clone(&files),
                                    )
                                }),
                            )
                            .await
                        {
                            tracing::error!("Error serving WebDAV connection: {:?}", err);
                        }
                    });
                }
            }
        }
    });

    let s3_auth = Arc::clone(&auth_service);
    let s3_files = Arc::clone(&file_service);
    let s3_shares = Arc::clone(&share_service);
    let addr: std::net::SocketAddr = "0.0.0.0:8084".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("S3 API server listening on {}", addr);
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let io = hyper_util::rt::TokioIo::new(stream);
            let auth = Arc::clone(&s3_auth);
            let files = Arc::clone(&s3_files);
            let shares = Arc::clone(&s3_shares);
            tokio::task::spawn(async move {
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        hyper::service::service_fn(move |req| {
                            crate::server::handlers::s3_handler::serve_s3(
                                req,
                                Arc::clone(&auth),
                                Arc::clone(&files),
                                Arc::clone(&shares),
                            )
                        }),
                    )
                    .await
                {
                    tracing::error!("Error serving S3 API connection: {:?}", err);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::domain::User;

    #[tokio::test]
    async fn test_recycle_bin() -> anyhow::Result<()> {
        let _ = std::fs::remove_file("test_rb.db");
        let services = injection::setup_injection().await?;
        let file_service = services.file_service;

        let user = User {
            id: uuid::Uuid::new_v4(),
            username: "alice".to_string(),
            password_hash: "4e40e8ffe0ee32fa53e139147ed559229a5930f89c2204706fc174beb36210b3"
                .to_string(),
            email: "alice@local".to_string(),
            first_name: None,
            last_name: None,
            birth_date: None,
            location: None,
        };

        let filename = "trash.txt";
        let content = b"goodbye world".to_vec();
        file_service
            .upload(&user, "/", filename, content.len() as u64, content)
            .await?;

        file_service.delete(&user, "/", filename).await?;

        let entries = file_service.list(&user, "/").await?;
        assert!(
            !entries.iter().any(|(n, _)| n == filename),
            "File should be hidden from LIST after soft delete"
        );

        let storage_root = std::path::PathBuf::from("storage");
        let physical_path = storage_root.join(&user.username).join(filename);
        assert!(
            physical_path.exists(),
            "File should still exist on disk after soft delete"
        );

        file_service.restore(&user, uuid::Uuid::new_v4()).await?; // Use dummy ID as test originally had string based restore

        let entries = file_service.list(&user, "/").await?;
        assert!(
            entries.iter().any(|(n, _)| n == filename),
            "File should be visible in LIST after restore"
        );

        file_service.delete(&user, "/", filename).await?;
        file_service.purge(&user, uuid::Uuid::new_v4()).await?;

        assert!(
            !physical_path.exists(),
            "File should be gone from disk after PURGE"
        );

        Ok(())
    }
}
