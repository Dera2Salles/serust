// src/main.rs
// Point d'entrée — monte le serveur via le builder du framework.

mod application;
mod domain;
mod framework;
mod handlers;
mod infrastructure;
mod interface;
mod middlewares;

use application::{auth_service::AuthService, file_service::FileService};
use framework::{app::App, metrics::Metrics};
use handlers::auth::{UserHandler, PassHandler};
use handlers::info::{SystHandler, FeatHandler, TypeHandler, NoopHandler, QuitHandler};
use handlers::network::{PasvHandler, PortHandler};
use handlers::dir::{DirHandler, CwdHandler, CdupHandler};
use handlers::transfer::{
    StorHandler, RetrHandler, ListDirHandler, NlstHandler,
    MkdHandler, RmdHandler, DeleHandler,
    RnfrHandler, RntoHandler, SizeHandler,
};
use infrastructure::{
    file_repository::FileRepository,
    user_repository::UserRepository,
};
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

    let auth_service = Arc::new(AuthService::new(Arc::clone(&user_repo)));
    let file_service = Arc::new(FileService::new(Arc::clone(&file_repo)));

    for (name, pass) in [("alice", "alice123"), ("bob", "bob456"), ("carol", "carol789")] {
        let _ = auth_service.register(name, pass).await;
        info!("Utilisateur prêt : {}", name);
    }

    let _metrics = Metrics::new();

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
        .run("0.0.0.0:8080")
        .await
}
