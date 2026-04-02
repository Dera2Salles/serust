// src/infrastructure/tcp_server.rs

use crate::application::{auth_service::AuthService, file_service::FileService};
use crate::interface::connection_handler::ConnectionHandler;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Nombre maximum de connexions simultanées.
const MAX_CONNECTIONS: usize = 1024;

/// Durée maximale d'inactivité avant fermeture forcée.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

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

    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = self.build_listener().await?;
        info!("Serveur TCP démarré sur {}", self.addr);

        // Semaphore : limite MAX_CONNECTIONS connexions simultanées.
        let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));

        // Graceful shutdown : écoute CTRL+C ou SIGTERM.
        let shutdown = Self::shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                // Priorité au signal d'arrêt.
                biased;

                _ = &mut shutdown => {
                    info!("Signal d'arrêt reçu — fermeture du serveur.");
                    break;
                }

                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            // Tente d'acquérir un slot de connexion sans bloquer.
                            let permit = match semaphore.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_) => {
                                    warn!(
                                        "Limite de connexions atteinte ({}) — refus de {}",
                                        MAX_CONNECTIONS, peer_addr
                                    );
                                    // Ferme proprement sans crasher le serveur.
                                    drop(stream);
                                    continue;
                                }
                            };

                            info!("Client connecté : {} ({} slots restants)",
                                peer_addr,
                                semaphore.available_permits()
                            );

                            // Active TCP keepalive pour détecter les clients fantômes.
                            if let Err(e) = Self::configure_socket(&stream) {
                                warn!("Impossible de configurer socket {}: {}", peer_addr, e);
                            }

                            let auth  = Arc::clone(&self.auth_service);
                            let files = Arc::clone(&self.file_service);

                            // 1 connexion = 1 tâche Tokio.
                            // Le permit est déplacé dans la tâche : libéré quand la tâche se termine.
                            tokio::spawn(Self::handle_connection(
                                stream, auth, files, permit, peer_addr.to_string(),
                            ));
                        }

                        // Erreurs accept() non-fatales (ex: EMFILE, ENFILE).
                        Err(e) => {
                            error!("Erreur accept() : {} — pause 100ms", e);
                            // Pause courte pour éviter une boucle infinie qui brûle le CPU.
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        info!("Serveur arrêté proprement.");
        Ok(())
    }

    // ─── Helpers privés ──────────────────────────────────────────────────────

    /// Crée le TcpListener avec SO_REUSEADDR pour redémarrage rapide.
    async fn build_listener(&self) -> anyhow::Result<TcpListener> {
        use socket2::{Domain, Protocol, Socket, Type};
        use std::net::SocketAddr;

        let addr: SocketAddr = self.addr.parse()?;
        let socket = Socket::new(
            if addr.is_ipv6() {
                Domain::IPV6
            } else {
                Domain::IPV4
            },
            Type::STREAM,
            Some(Protocol::TCP),
        )?;

        // Permet de redémarrer le serveur sans attendre TIME_WAIT.
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(1024)?; // backlog = 1024

        let listener = TcpListener::from_std(socket.into())?;
        Ok(listener)
    }

    /// Configure TCP keepalive et Nagle sur la socket cliente.
    fn configure_socket(stream: &TcpStream) -> anyhow::Result<()> {
        use socket2::{SockRef, TcpKeepalive};

        let sock_ref = SockRef::from(stream);
        let keepalive = TcpKeepalive::new()
            .with_time(Duration::from_secs(60)) // idle avant premier probe
            .with_interval(Duration::from_secs(10)) // intervalle entre probes
            .with_retries(3); // probes avant abandon

        sock_ref.set_tcp_keepalive(&keepalive)?;
        // Désactive Nagle : envoie immédiatement les petites réponses texte.
        stream.set_nodelay(true)?;
        Ok(())
    }

    /// Gère une connexion avec un timeout global.
    /// Le permit est libéré automatiquement à la fin (drop).
    async fn handle_connection(
        stream: TcpStream,
        auth: Arc<AuthService>,
        files: Arc<FileService>,
        _permit: OwnedSemaphorePermit,
        peer_addr: String,
    ) {
        let task = async {
            let mut handler = ConnectionHandler::new(stream, auth, files);
            handler.handle().await
        };

        match timeout(CONNECTION_TIMEOUT, task).await {
            Ok(Ok(())) => {
                info!("Client déconnecté proprement : {}", peer_addr);
            }
            Ok(Err(e)) => {
                error!("Erreur connexion {} : {}", peer_addr, e);
            }
            Err(_) => {
                warn!(
                    "Timeout {}s dépassé — fermeture forcée : {}",
                    CONNECTION_TIMEOUT.as_secs(),
                    peer_addr
                );
            }
        }
        // _permit droppé ici → slot libéré dans le semaphore.
    }

    /// Attend CTRL+C (tous OS) ou SIGTERM (Unix seulement).
    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("Impossible d'écouter CTRL+C");
        };

        #[cfg(unix)]
        let sigterm = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Impossible d'écouter SIGTERM")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let sigterm = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c  => { info!("CTRL+C reçu"); }
            _ = sigterm => { info!("SIGTERM reçu"); }
        }
    }
}
