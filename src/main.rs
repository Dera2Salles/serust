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

use framework::metrics::Metrics;
use mcp::{registry::McpRegistry, server::run_mcp_server};
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

    let _metrics = Metrics::new();

    let mcp_registry = Arc::new(McpRegistry::new(Arc::clone(&file_service)));
    tokio::spawn(async move {
        if let Err(e) = run_mcp_server(mcp_registry, "0.0.0.0:8081").await {
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

    let server = server::tcp_server::TcpServer::new(
        Arc::clone(&auth_service),
        Arc::clone(&file_service),
        Arc::clone(&share_service),
    );

    server.run("0.0.0.0:8080").await?;

    Ok(())
}
