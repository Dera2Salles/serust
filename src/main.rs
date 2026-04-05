// src/main.rs
// Point d'entrée — monte le serveur via le builder du framework.

mod application;
mod domain;
mod framework;
mod handlers;
mod infrastructure;
mod interface;
mod mcp;
mod middlewares;

use application::{auth_service::AuthService, file_service::FileService, share_service::ShareService};
use framework::{app::App, metrics::Metrics};
use handlers::auth::{UserHandler, PassHandler};
use handlers::info::{SystHandler, FeatHandler, TypeHandler, NoopHandler, QuitHandler, MlstHandler};
use handlers::network::{PasvHandler, PortHandler};
use handlers::dir::{DirHandler, CwdHandler, CdupHandler};
use handlers::share::{ShareHandler, UnshareHandler, SharesHandler};
use handlers::transfer::{
    StorHandler, RetrHandler, ListDirHandler, NlstHandler,
    MkdHandler, RmdHandler, DeleHandler,
    RnfrHandler, RntoHandler, SizeHandler,
};
use infrastructure::{
    file_repository::FileRepository,
    share_repository::ShareRepository,
    user_repository::UserRepository,
};
use mcp::{server::run_mcp_server, registry::McpRegistry};
use middlewares::{
    auth_middleware::AuthMiddleware,
    logging_middleware::LoggingMiddleware,
    rate_limit_middleware::RateLimitMiddleware,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    info!("Démarrage du framework TCP...");

    let user_repo = Arc::new(UserRepository::new("users.json").await);
    let file_repo = Arc::new(FileRepository::new("storage"));
    let share_repo = Arc::new(ShareRepository::new("shares.json").await);

    let auth_service = Arc::new(AuthService::new(Arc::clone(&user_repo)));
    let share_service = Arc::new(ShareService::new(Arc::clone(&share_repo)));
    let file_service = Arc::new(FileService::new(Arc::clone(&file_repo), Arc::clone(&share_service)));

    for (name, pass) in [("alice", "alice123"), ("bob", "bob456"), ("carol", "carol789")] {
        let _ = auth_service.register(name, pass).await;
        info!("Utilisateur prêt : {}", name);
    }

    let _metrics = Metrics::new();

    // ── MCP Server (port 8081) ────────────────────────────────────────────────
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
        // ── Middlewares ──
        .middleware(LoggingMiddleware)
        .middleware(RateLimitMiddleware::new(200, Duration::from_secs(60)))
        .middleware(AuthMiddleware)
        // ── Auth ──
        .route(UserHandler)
        .route(PassHandler::new(Arc::clone(&auth_service)))
        // ── Info / utils ──
        .route(SystHandler)
        .route(FeatHandler)
        .route(TypeHandler)
        .route(NoopHandler)
        .route(QuitHandler)
        .route(MlstHandler::new(Arc::clone(&file_service)))
        // ── Data connection ──
        .route(PasvHandler)
        .route(PortHandler)
        // ── Navigation ──
        .route(DirHandler)
        .route(CwdHandler::new(Arc::clone(&file_service)))
        .route(CdupHandler)
        // ── Transfers ──
        .route(StorHandler::new(Arc::clone(&file_service)))
        .route(RetrHandler::new(Arc::clone(&file_service)))
        .route(ListDirHandler::new(Arc::clone(&file_service)))
        .route(NlstHandler::new(Arc::clone(&file_service)))
        // ── File management ──
        .route(MkdHandler::new(Arc::clone(&file_service)))
        .route(RmdHandler::new(Arc::clone(&file_service)))
        .route(DeleHandler::new(Arc::clone(&file_service)))
        .route(RnfrHandler)
        .route(RntoHandler::new(Arc::clone(&file_service)))
        .route(SizeHandler::new(Arc::clone(&file_service)))
        // ── Sharing ──
        .route(ShareHandler::new(Arc::clone(&share_service)))
        .route(UnshareHandler::new(Arc::clone(&share_service)))
        .route(SharesHandler::new(Arc::clone(&share_service)))
        .run("0.0.0.0:8080")
        .await
}
