mod application;
mod domain;
mod infrastructure;
mod interface;

use application::{auth_service::AuthService, file_service::FileService};
use infrastructure::{
    file_repository::FileRepository, tcp_server::TcpServer, user_repository::UserRepository,
};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ─── Logs ─────────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    info!("Démarrage du serveur TCP de fichiers...");

    // ─── Repositories ─────────────────────────────────────────────────────────
    let user_repo = Arc::new(UserRepository::new("users.json").await);
    let file_repo = Arc::new(FileRepository::new("storage"));

    // ─── Services ──────────────────────────────────────────────────────────────
    let auth_service = Arc::new(AuthService::new(Arc::clone(&user_repo)));
    let file_service = Arc::new(FileService::new(Arc::clone(&file_repo)));

    // ─── Seed utilisateurs de démo ─────────────────────────────────────────────
    seed_users(&auth_service).await;

    // ─── Serveur TCP ───────────────────────────────────────────────────────────
    let server = TcpServer::new(
        "0.0.0.0:8080",
        Arc::clone(&auth_service),
        Arc::clone(&file_service),
    );
    server.run().await?;

    Ok(())
}

/// Crée des utilisateurs de test si absents.
async fn seed_users(auth: &AuthService) {
    let users = [
        ("alice", "alice123"),
        ("bob", "bob456"),
        ("carol", "carol789"),
    ];
    for (name, pass) in users {
        let _ = auth.register(name, pass).await;
        info!("Utilisateur enregistré (ou déjà existant) : {}", name);
    }
}
