// src/main.rs
// Point d'entrée — monte le serveur via le builder du framework.
// Pour ajouter une commande : implémenter Handler, ajouter .route(MonHandler) ici.

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
use handlers::info::{SystHandler, FeatHandler, TypeHandler};
use handlers::network::{PasvHandler, PortHandler};
use handlers::dir::{DirHandler, CwdHandler};
use handlers::transfer::{StorHandler, RetrHandler, ListDirHandler, MkdHandler, RmdHandler, DeleHandler};
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
    // ── Logs ──────────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .init();

    info!("Démarrage du framework TCP...");

    // ── Repositories ──────────────────────────────────────────────────────────
    let user_repo = Arc::new(UserRepository::new("users.json").await);
    let file_repo = Arc::new(FileRepository::new("storage"));

    // ── Services ──────────────────────────────────────────────────────────────
    let auth_service = Arc::new(AuthService::new(Arc::clone(&user_repo)));
    let file_service = Arc::new(FileService::new(Arc::clone(&file_repo)));

    // ── Seed utilisateurs de démo ─────────────────────────────────────────────
    for (name, pass) in [("alice", "alice123"), ("bob", "bob456"), ("carol", "carol789")] {
        let _ = auth_service.register(name, pass).await;
        info!("Utilisateur prêt : {}", name);
    }

    // ── Métriques partagées entre handlers ────────────────────────────────────
    let _metrics = Metrics::new();

    // ── Construction du serveur via le framework ───────────────────────────────
    //
    // Ordre des middlewares = ordre d'exécution :
    //   1. LoggingMiddleware   → log chaque commande
    //   2. RateLimitMiddleware → bloque si trop de requêtes
    //   3. AuthMiddleware      → bloque si non authentifié
    //   4. Handler             → logique métier
    //
    // Pour ajouter une commande : .route(MonHandler::new(...))
    // Pour ajouter un middleware : .middleware(MonMiddleware)

    App::new()
        .banner("WELCOME tcp-framework/1.0")
        .max_connections(1024)
        .timeout(Duration::from_secs(300))
        // ── Middlewares ──
        .middleware(LoggingMiddleware)
        .middleware(RateLimitMiddleware::new(200, Duration::from_secs(60)))
        .middleware(AuthMiddleware)
        // ── Handlers (RFC 959) ──
        .route(UserHandler)
        .route(PassHandler::new(Arc::clone(&auth_service)))
        .route(SystHandler)
        .route(FeatHandler)
        .route(TypeHandler)
        .route(PasvHandler)
        .route(PortHandler)
        .route(DirHandler)
        .route(CwdHandler)
        .route(StorHandler::new(Arc::clone(&file_service)))
        .route(RetrHandler::new(Arc::clone(&file_service)))
        .route(ListDirHandler::new(Arc::clone(&file_service)))
        .route(MkdHandler::new(Arc::clone(&file_service)))
        .route(RmdHandler::new(Arc::clone(&file_service)))
        .route(DeleHandler::new(Arc::clone(&file_service)))
        .run("0.0.0.0:8080")
        .await
}
