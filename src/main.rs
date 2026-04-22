mod common;
mod database;
mod file;
mod framework;
mod injection;
mod log;
mod mcp;
mod server;
mod share;
mod user;
mod webdav;

use framework::metrics::Metrics;
use mcp::{registry::McpRegistry, server::{run_mcp_server, McpServerState}};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("Démarrage du framework TCP...");

    let services = injection::setup_injection().await?;
    let auth_service = services.auth_service;
    let share_service = services.share_service;
    let file_service = services.file_service;
    let db = services.db;

    let _metrics = Metrics::new();

    let mcp_registry = Arc::new(McpRegistry::new(Arc::clone(&file_service)));
    let mcp_state = Arc::new(McpServerState {
        registry: Arc::clone(&mcp_registry),
        file_service: Arc::clone(&file_service),
        db: db.clone(),
    });
    tokio::spawn(async move {
        if let Err(e) = run_mcp_server(mcp_state, "0.0.0.0:8081").await {
            tracing::error!("MCP server error: {}", e);
        }
    });
    info!("MCP server spawned on 0.0.0.0:8081");

    let fallback_auth = Arc::clone(&auth_service);
    let fallback_files = Arc::clone(&file_service);
    tokio::spawn(async move {
        if let Ok(listener) = tokio::net::TcpListener::bind("0.0.0.0:8082").await {
            info!("Legacy Raw Protocol fallback listening on 0.0.0.0:8082");
            while let Ok((stream, _)) = listener.accept().await {
                let mut handler =
                    crate::server::interface::connection_handler::ConnectionHandler::new(
                        stream,
                        Arc::clone(&fallback_auth),
                        Arc::clone(&fallback_files),
                    );
                tokio::spawn(async move {
                    let _ = handler.handle().await;
                });
            }
        }
    });

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
                            .serve_connection(io, hyper::service::service_fn(move |req| {
                                crate::webdav::handler::serve_webdav(req, Arc::clone(&auth), Arc::clone(&files))
                            }))
                            .await
                        {
                            tracing::error!("Error serving WebDAV connection: {:?}", err);
                        }
                    });
                }
            }
        }
    });

    let server = server::tcp_server::TcpServer::new(
        Arc::clone(&auth_service),
        Arc::clone(&file_service),
        Arc::clone(&share_service),
    );

    server.run("0.0.0.0:8080").await?;

    Ok(())
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
            username: "alice".to_string(),
            password_hash: "4e40e8ffe0ee32fa53e139147ed559229a5930f89c2204706fc174beb36210b3".to_string(),
        };

        // 1. Upload
        let filename = "trash.txt";
        let content = b"goodbye world".to_vec();
        file_service.upload(&user, "/", filename, content.len() as u64, content).await?;

        // 2. Soft Delete
        file_service.delete(&user, "/", filename).await?;

        // 3. Verify hidden from LIST
        let entries = file_service.list(&user, "/").await?;
        assert!(!entries.iter().any(|(n, _)| n == filename), "File should be hidden from LIST after soft delete");

        // 4. Verify still on disk
        let storage_root = std::path::PathBuf::from("storage");
        let physical_path = storage_root.join(&user.username).join(filename);
        assert!(physical_path.exists(), "File should still exist on disk after soft delete");

        // 5. Undelete
        file_service.restore(&user, "/", filename).await?;

        // 6. Verify visible in LIST
        let entries = file_service.list(&user, "/").await?;
        assert!(entries.iter().any(|(n, _)| n == filename), "File should be visible in LIST after restore");

        // 7. Purge
        file_service.delete(&user, "/", filename).await?;
        file_service.purge(&user).await?;

        // 8. Verify gone from disk
        assert!(!physical_path.exists(), "File should be gone from disk after PURGE");

        Ok(())
    }
}

