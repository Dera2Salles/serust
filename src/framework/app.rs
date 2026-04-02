// src/framework/app.rs
//
// AppBuilder : l'API publique du framework.
// Permet de composer un serveur en chaînant .middleware() et .route().
//
// Usage :
//   App::new()
//       .max_connections(512)
//       .timeout(Duration::from_secs(120))
//       .middleware(AuthMiddleware::new(auth_service))
//       .middleware(LoggingMiddleware)
//       .middleware(RateLimitMiddleware::new(60))
//       .route(LoginHandler::new(auth_service))
//       .route(UploadHandler::new(file_service))
//       .route(DownloadHandler::new(file_service))
//       .route(ListHandler::new(file_service))
//       .route(PingHandler)          // commande custom
//       .run("0.0.0.0:8080")
//       .await?;

use crate::framework::{
    context::Context,
    handler::Handler,
    metrics::Metrics,
    router::Router,
};
use crate::middlewares::middleware::{Middleware, MiddlewareResult};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn};

const DEFAULT_MAX_CONNECTIONS: usize = 1024;
const DEFAULT_TIMEOUT_SECS: u64 = 300;
const METRICS_LOG_INTERVAL_SECS: u64 = 60;

pub struct App {
    max_connections: usize,
    conn_timeout: Duration,
    router: Router,
    middlewares: Vec<Arc<dyn Middleware>>,
    metrics: Arc<Metrics>,
    banner: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            conn_timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            router: Router::new(),
            middlewares: Vec::new(),
            metrics: Metrics::new(),
            banner: "WELCOME tcp-framework/1.0".to_string(),
        }
    }

    // ─── Builder methods ─────────────────────────────────────────────────────

    pub fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.conn_timeout = d;
        self
    }

    pub fn banner(mut self, b: impl Into<String>) -> Self {
        self.banner = b.into();
        self
    }

    /// Enregistre un middleware (ordre = ordre d'appel).
    pub fn middleware<M: Middleware>(mut self, m: M) -> Self {
        self.middlewares.push(Arc::new(m));
        self
    }

    /// Enregistre un handler de commande.
    pub fn route<H: Handler>(mut self, h: H) -> Self {
        self.router.register(h);
        self
    }



    // ─── Run ─────────────────────────────────────────────────────────────────

    /// Démarre le serveur TCP. Bloque jusqu'au signal d'arrêt.
    pub async fn run(self, addr: &str) -> anyhow::Result<()> {
        let listener = build_listener(addr)?;
        info!("Framework TCP démarré sur {} | max={} conn | timeout={}s",
            addr, self.max_connections, self.conn_timeout.as_secs());
        info!("Commandes enregistrées : {:?}", self.router.commands());

        let semaphore   = Arc::new(Semaphore::new(self.max_connections));
        let router      = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        let metrics     = Arc::clone(&self.metrics);
        let conn_timeout = self.conn_timeout;
        let banner      = Arc::new(self.banner);

        // Tâche périodique de log des métriques
        {
            let m = Arc::clone(&metrics);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(
                    Duration::from_secs(METRICS_LOG_INTERVAL_SECS)
                );
                interval.tick().await; // saute la première tick immédiate
                loop {
                    interval.tick().await;
                    m.log_snapshot();
                }
            });
        }

        let shutdown = shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                biased;
                _ = &mut shutdown => {
                    info!("Arrêt du framework — signal reçu.");
                    metrics.log_snapshot();
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            let permit = match semaphore.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_) => {
                                    warn!("Limite atteinte — refus {}", peer_addr);
                                    drop(stream);
                                    continue;
                                }
                            };

                            configure_socket(&stream);
                            metrics.connection_opened();

                            info!("+ {} ({} actives)", peer_addr,
                                metrics.snapshot().active_connections);

                            let router      = Arc::clone(&router);
                            let middlewares = Arc::clone(&middlewares);
                            let metrics     = Arc::clone(&metrics);
                            let banner      = Arc::clone(&banner);

                            tokio::spawn(run_connection(
                                stream, peer_addr, router, middlewares,
                                metrics, banner, permit, conn_timeout,
                            ));
                        }
                        Err(e) => {
                            error!("accept() error: {} — backoff 100ms", e);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ─── Boucle de connexion ─────────────────────────────────────────────────────

async fn run_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    router: Arc<Router>,
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    metrics: Arc<Metrics>,
    banner: Arc<String>,
    _permit: OwnedSemaphorePermit,
    conn_timeout: Duration,
) {
    let result = timeout(
        conn_timeout,
        handle_connection(stream, peer_addr, router, middlewares, metrics.clone(), banner),
    ).await;

    match result {
        Ok(Ok(())) => info!("- {} (propre)", peer_addr),
        Ok(Err(e)) => error!("- {} erreur: {}", peer_addr, e),
        Err(_)     => warn!("- {} timeout", peer_addr),
    }

    metrics.connection_closed();
    // _permit droppé → slot libéré dans le semaphore
}

async fn handle_connection(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    router: Arc<Router>,
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    metrics: Arc<Metrics>,
    banner: Arc<String>,
) -> anyhow::Result<()> {
    let local_addr = stream.local_addr()?;
    // Envoie le banner de bienvenue
    stream.write_all(format!("220 {}\r\n", banner).as_bytes()).await?;

    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::with_capacity(8 * 1024, reader);
    let mut ctx = Context::new(peer_addr, local_addr);

    loop {
        let mut line = String::new();
        let n = buf_reader.read_line(&mut line).await?;

        // EOF — client déconnecté
        if n == 0 { break; }

        let line = line.trim_end_matches(['\n', '\r']).to_string();
        if line.is_empty() { continue; }

        // Parse : "COMMAND arg1 arg2 ..."
        let mut parts = line.splitn(32, ' ');
        let command = match parts.next() {
            Some(c) => c.to_uppercase(),
            None => continue,
        };
        let args: Vec<&str> = parts.collect();

        // Commande QUIT gérée par le framework lui-même
        if command == "QUIT" {
            writer.write_all(b"221 Goodbye.\r\n").await?;
            break;
        }

        metrics.command_received();

        // ── Pipeline middlewares ──
        let mut stopped = false;
        for mw in middlewares.iter() {
            match mw.before(&mut ctx, &command).await {
                MiddlewareResult::Continue => {}
                MiddlewareResult::Stop => {
                    stopped = true;
                    break;
                }
            }
        }

        // ── Dispatch vers le handler ──
        if !stopped {
            match router.resolve(&command) {
                Some(handler) => {
                    // Vérification auth (délégué au handler via requires_auth)
                    if handler.requires_auth() && !ctx.is_authenticated() {
                        ctx.error(530, "Not logged in.");
                        metrics.error_occurred();
                    } else {
                        let arg_refs: Vec<&str> = args.iter().map(|s| *s).collect();
                        if let Err(e) = handler.handle(&mut ctx, &arg_refs, &mut buf_reader, &mut writer).await {
                            error!("Handler {} error: {}", command, e);
                            ctx.error(500, &e.to_string());
                            metrics.error_occurred();
                        }
                    }
                }
                None => {
                    ctx.error(500, "Unknown command.");
                    metrics.error_occurred();
                }
            }
        }

        // ── Flush le buffer de réponse du Context ──
        if !ctx.response.is_empty() {
            writer.write_all(&ctx.response).await?;
            ctx.response.clear();
        }
    }

    Ok(())
}

// ─── Helpers réseau ──────────────────────────────────────────────────────────

fn build_listener(addr: &str) -> anyhow::Result<TcpListener> {
    let addr: SocketAddr = addr.parse()?;
    let socket = Socket::new(
        if addr.is_ipv6() { Domain::IPV6 } else { Domain::IPV4 },
        Type::STREAM,
        Some(Protocol::TCP),
    )?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(4096)?;
    Ok(TcpListener::from_std(socket.into())?)
}

fn configure_socket(stream: &TcpStream) {
    use socket2::{SockRef, TcpKeepalive};
    let _ = stream.set_nodelay(true);
    let sock = SockRef::from(stream);
    let ka = TcpKeepalive::new()
        .with_time(Duration::from_secs(60))
        .with_interval(Duration::from_secs(10))
        .with_retries(3);
    let _ = sock.set_tcp_keepalive(&ka);
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM listener failed")
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
