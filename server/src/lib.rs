pub mod common;
pub mod database;
mod file;
mod injection;
mod log;
mod mcp;
mod server;
mod share;
mod user;
mod webdav;

use mcp::{
    registry::McpRegistry,
    server::{run_mcp_server, McpServerState},
};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};
use dotenvy;

pub async fn run_server() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let file_appender = tracing_appender::rolling::never(".", "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let _ = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false))
        .try_init();

    info!("Démarrage du framework TCP...");

    let services = injection::setup_injection().await?;
    let auth_service = services.auth_service;
    let share_service = services.share_service;
    let file_service = services.file_service;
    let log_access_usecase = services.log_access_usecase;
    let db = services.db;
    
    let settings = crate::common::config::load_config();
    let session_registry = Arc::new(crate::common::session::SessionRegistry::new());

    let mcp_registry = Arc::new(McpRegistry::new(
        Arc::clone(&file_service),
        Arc::new(crate::database::user_repository::UserRepository::new(
            db.clone(),
        )),
        Arc::clone(&share_service),
    ));
    let mcp_state = Arc::new(McpServerState {
        registry: Arc::clone(&mcp_registry),
        file_service: Arc::clone(&file_service),
        auth_service: Arc::clone(&auth_service),
        log_access_usecase: Arc::clone(&log_access_usecase),
        db: db.clone(),
        sessions: Arc::clone(&session_registry),
    });
    
    let mcp_addr = format!("0.0.0.0:{}", settings.mcp_port);
    tokio::spawn(async move {
        if let Err(e) = run_mcp_server(mcp_state, &mcp_addr).await {
            tracing::error!("MCP server error: {}", e);
        }
    });
    info!("MCP server spawned on 0.0.0.0:{}", settings.mcp_port);

    let webdav_auth = Arc::clone(&auth_service);
    let webdav_files = Arc::clone(&file_service);
    let webdav_port = settings.webdav_port;
    let webdav_sessions = Arc::clone(&session_registry);
    tokio::spawn(async move {
        let addr_str = format!("0.0.0.0:{}", webdav_port);
        let addr: std::net::SocketAddr = addr_str.parse().unwrap();
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            tracing::info!("WebDAV HTTP server listening on {}", addr);
            loop {
                if let Ok((stream, peer)) = listener.accept().await {
                    let io = hyper_util::rt::TokioIo::new(stream);
                    let auth = Arc::clone(&webdav_auth);
                    let files = Arc::clone(&webdav_files);
                    let sessions = Arc::clone(&webdav_sessions);
                    let session_id = uuid::Uuid::new_v4().to_string();
                    
                    sessions.add_session(crate::common::session::ActiveSession {
                        id: session_id.clone(),
                        peer_addr: peer.to_string(),
                        connected_at: chrono::Utc::now(),
                        last_command: Some("CONNECT".to_string()),
                        username: None,
                    });

                    let sessions_task = Arc::clone(&sessions);
                    let session_id_task = session_id.clone();

                    tokio::task::spawn(async move {
                        let sessions_svc = Arc::clone(&sessions_task);
                        let session_id_svc = session_id_task.clone();
                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(
                                io,
                                hyper::service::service_fn(move |req| {
                                    let sessions_inner = Arc::clone(&sessions_svc);
                                    let session_id_inner = session_id_svc.clone();
                                    let auth = Arc::clone(&auth);
                                    let files = Arc::clone(&files);
                                    async move {
                                        let method = req.method().to_string();
                                        let path = req.uri().path().to_string();
                                        sessions_inner.update_last_command(&session_id_inner, format!("{} {}", method, path), None);
                                        crate::webdav::handler::serve_webdav(
                                            req,
                                            auth,
                                            files,
                                            sessions_inner,
                                            session_id_inner,
                                        ).await
                                    }
                                }),
                            )
                            .await
                        {
                            tracing::error!("Error serving WebDAV connection: {:?}", err);
                        }
                        sessions_task.remove_session(&session_id_task);
                    });
                }
            }
        }
    });

    let s3_auth = Arc::clone(&auth_service);
    let s3_files = Arc::clone(&file_service);
    let s3_shares = Arc::clone(&share_service);
    let s3_logs = Arc::clone(&log_access_usecase);
    let s3_sessions = Arc::clone(&session_registry);
    let s3_addr_str = format!("0.0.0.0:{}", settings.s3_port);
    let addr: std::net::SocketAddr = s3_addr_str.parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("S3 API server listening on {}", addr);
    loop {
        if let Ok((stream, peer)) = listener.accept().await {
            let io = hyper_util::rt::TokioIo::new(stream);
            let auth = Arc::clone(&s3_auth);
            let files = Arc::clone(&s3_files);
            let shares = Arc::clone(&s3_shares);
            let logs = Arc::clone(&s3_logs);
            let sessions = Arc::clone(&s3_sessions);
            let session_id = uuid::Uuid::new_v4().to_string();

            sessions.add_session(crate::common::session::ActiveSession {
                id: session_id.clone(),
                peer_addr: peer.to_string(),
                connected_at: chrono::Utc::now(),
                last_command: Some("CONNECT".to_string()),
                username: None,
            });

            let sessions_task = Arc::clone(&sessions);
            let session_id_task = session_id.clone();

            tokio::task::spawn(async move {
                let sessions_svc = Arc::clone(&sessions_task);
                let session_id_svc = session_id_task.clone();
                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(
                        io,
                        hyper::service::service_fn(move |req| {
                            let sessions_inner = Arc::clone(&sessions_svc);
                            let session_id_inner = session_id_svc.clone();
                            let auth = Arc::clone(&auth);
                            let files = Arc::clone(&files);
                            let shares = Arc::clone(&shares);
                            let logs = Arc::clone(&logs);
                            async move {
                                let method = req.method().to_string();
                                let path = req.uri().path().to_string();
                                sessions_inner.update_last_command(&session_id_inner, format!("{} {}", method, path), None);
                                crate::server::handlers::s3_handler::serve_s3(
                                    req,
                                    auth,
                                    files,
                                    shares,
                                    logs,
                                    sessions_inner,
                                    session_id_inner,
                                ).await
                            }
                        }),
                    )
                    .await
                {
                    tracing::error!("Error serving S3 API connection: {:?}", err);
                }
                sessions_task.remove_session(&session_id_task);
            });
        }
    }
}
