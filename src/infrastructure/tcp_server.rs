
use crate::application::{auth_service::AuthService, file_service::FileService};
use crate::interface::connection_handler::ConnectionHandler;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

pub struct TcpServer {
    addr: String,
    auth_service: Arc<AuthService>,
    file_service: Arc<FileService>,
}

impl TcpServer {
    pub fn new(
        addr: impl Into<String>,
        auth_service: Arc<AuthService>,
        file_service: Arc<FileService>,
    ) -> Self {
        Self {
            addr: addr.into(),
            auth_service,
            file_service,
        }
    }

    /// Démarre la boucle d'acceptation des connexions TCP.
    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!("Serveur TCP démarré sur {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    info!("Client connecté : {}", peer_addr);

                    let auth = Arc::clone(&self.auth_service);
                    let files = Arc::clone(&self.file_service);

                    tokio::spawn(async move {
                        let mut handler = ConnectionHandler::new(stream, auth, files);
                        if let Err(e) = handler.handle().await {
                            error!("Erreur connexion {}: {}", peer_addr, e);
                        }
                        info!("Client déconnecté : {}", peer_addr);
                    });
                }
                Err(e) => {
                    error!("Erreur accept: {}", e);
                }
            }
        }
    }
}