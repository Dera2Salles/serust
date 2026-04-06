mod application;
mod domain;
mod framework;
mod handlers;
mod infrastructure;
mod interface;
mod mcp;
mod middlewares;
mod injection;

use framework::{app::App, metrics::Metrics};
use handlers::auth::{PassHandler, UserHandler};
use handlers::dir::{CdupHandler, CwdHandler, DirHandler};
use handlers::info::{
    FeatHandler, MlstHandler, NoopHandler, QuitHandler, SystHandler, TypeHandler,
};
use handlers::network::{PasvHandler, PortHandler};
use handlers::share::{ShareHandler, SharesHandler, UnshareHandler};
use handlers::transfer::{
    DeleHandler, ListDirHandler, MkdHandler, NlstHandler, RetrHandler, RmdHandler, RnfrHandler,
    RntoHandler, SizeHandler, StorHandler,
};
use mcp::{registry::McpRegistry, server::run_mcp_server};
use middlewares::{
    auth_middleware::AuthMiddleware, logging_middleware::LoggingMiddleware,
    rate_limit_middleware::RateLimitMiddleware,
};
use std::sync::Arc;
use std::time::Duration;
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

    App::new()
        .banner("220 tcp-framework FTP Server ready.")
        .max_connections(1024)
        .timeout(Duration::from_secs(300))
        .middleware(LoggingMiddleware)
        .middleware(RateLimitMiddleware::new(200, Duration::from_secs(60)))
        .middleware(AuthMiddleware)
        .route(UserHandler)
        .route(PassHandler::new(Arc::clone(&auth_service)))
        .route(SystHandler)
        .route(FeatHandler)
        .route(TypeHandler)
        .route(NoopHandler)
        .route(QuitHandler)
        .route(MlstHandler::new(Arc::clone(&file_service)))
        .route(PasvHandler)
        .route(PortHandler)
        .route(DirHandler)
        .route(CwdHandler::new(Arc::clone(&file_service)))
        .route(CdupHandler)
        .route(StorHandler::new(Arc::clone(&file_service)))
        .route(RetrHandler::new(Arc::clone(&file_service)))
        .route(ListDirHandler::new(Arc::clone(&file_service)))
        .route(NlstHandler::new(Arc::clone(&file_service)))
        .route(MkdHandler::new(Arc::clone(&file_service)))
        .route(RmdHandler::new(Arc::clone(&file_service)))
        .route(DeleHandler::new(Arc::clone(&file_service)))
        .route(RnfrHandler)
        .route(RntoHandler::new(Arc::clone(&file_service)))
        .route(SizeHandler::new(Arc::clone(&file_service)))
        .route(ShareHandler::new(Arc::clone(&share_service)))
        .route(UnshareHandler::new(Arc::clone(&share_service)))
        .route(SharesHandler::new(Arc::clone(&share_service)))
        .run("0.0.0.0:8080")
        .await
}
