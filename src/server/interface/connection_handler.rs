use crate::file::service::FileService;
use crate::server::interface::protocol_handler::{parse_command, Command};
use crate::user::domain::User;
use crate::user::service::AuthService;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{error, info, warn};

pub struct Session {
    pub user: Option<User>,
}

impl Session {
    fn new() -> Self {
        Self { user: None }
    }

    fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }
}

pub struct ConnectionHandler {
    stream: TcpStream,
    auth_service: Arc<AuthService>,
    file_service: Arc<FileService>,
}

impl ConnectionHandler {
    pub fn new(
        stream: TcpStream,
        auth_service: Arc<AuthService>,
        file_service: Arc<FileService>,
    ) -> Self {
        Self {
            stream,
            auth_service,
            file_service,
        }
    }

    /// Point d'entrée : boucle de lecture des commandes.
    pub async fn handle(&mut self) -> anyhow::Result<()> {
        let auth = Arc::clone(&self.auth_service);
        let files = Arc::clone(&self.file_service);

        let (reader, mut writer) = self.stream.split();
        let mut buf_reader = BufReader::new(reader);
        let mut session = Session::new();

        writer.write_all(b"WELCOME tcp-file-server\n").await?;

        loop {
            let mut line = String::new();
            let n = buf_reader.read_line(&mut line).await?;

            if n == 0 {
                break;
            }

            let line = line.trim_end_matches(['\n', '\r']).to_string();
            if line.is_empty() {
                continue;
            }

            info!(
                "Commande reçue: {:?} (user={:?})",
                line,
                session.user.as_ref().map(|u| &u.username)
            );

            match parse_command(&line) {
                Err(e) => {
                    let msg = format!("ERROR {}\n", e);
                    writer.write_all(msg.as_bytes()).await?;
                }

                Ok(Command::Login { username, password }) => {
                    let response = handle_login(&auth, &mut session, &username, &password).await;
                    writer.write_all(response.as_bytes()).await?;
                }

                Ok(_) if !session.is_authenticated() => {
                    writer.write_all(b"ERROR not_authenticated\n").await?;
                }

                Ok(Command::Upload { filename, size }) => {
                    let response =
                        handle_upload(&files, &session, &mut buf_reader, &filename, size).await;
                    writer.write_all(response.as_bytes()).await?;
                }

                Ok(Command::Download { filename }) => {
                    handle_download(&files, &session, &mut writer, &filename).await;
                }

                Ok(Command::List) => {
                    let response = handle_list(&files, &session).await;
                    writer.write_all(response.as_bytes()).await?;
                }

                Ok(Command::Quit) => {
                    writer.write_all(b"BYE\n").await?;
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_login(
    auth: &AuthService,
    session: &mut Session,
    username: &str,
    password: &str,
) -> String {
    match auth.login(username, password).await {
        Ok(user) => {
            info!("Utilisateur connecté : {}", user.username);
            session.user = Some(user);
            "OK\n".to_string()
        }
        Err(e) => {
            warn!("Échec login pour '{}': {}", username, e);
            format!("ERROR {}\n", e)
        }
    }
}

async fn handle_upload(
    files: &FileService,
    session: &Session,
    reader: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
    filename: &str,
    size: u64,
) -> String {
    let user = session.user.as_ref().unwrap();

    if size > 100 * 1024 * 1024 {
        return "ERROR file_too_large\n".to_string();
    }

    let mut data = vec![0u8; size as usize];
    if let Err(e) = reader.read_exact(&mut data).await {
        error!("Erreur lecture upload '{}': {}", filename, e);
        return "ERROR io_error\n".to_string();
    }

    match files.upload(user, "", filename, size, data).await {
        Ok(_) => {
            info!("Fichier uploadé : {} par {}", filename, user.username);
            "DONE\n".to_string()
        }
        Err(e) => {
            error!("Erreur upload '{}': {}", filename, e);
            format!("ERROR {}\n", e)
        }
    }
}

async fn handle_download(
    files: &FileService,
    session: &Session,
    writer: &mut tokio::net::tcp::WriteHalf<'_>,
    filename: &str,
) {
    let user = session.user.as_ref().unwrap();

    match files.download(user, "", filename).await {
        Ok(data) => {
            info!("Fichier téléchargé : {} par {}", filename, user.username);
            let header = format!("SIZE {}\n", data.len());
            if writer.write_all(header.as_bytes()).await.is_ok() {
                if let Err(e) = writer.write_all(&data).await {
                    error!("Erreur envoi data '{}': {}", filename, e);
                }
            }
        }
        Err(e) => {
            error!("Erreur download '{}': {}", filename, e);
            let msg = format!("ERROR {}\n", e);
            let _ = writer.write_all(msg.as_bytes()).await;
        }
    }
}

async fn handle_list(files: &FileService, session: &Session) -> String {
    let user = session.user.as_ref().unwrap();

    match files.list(user, "").await {
        Ok(list) if list.is_empty() => "OK (empty)\n".to_string(),
        Ok(list) => {
            info!("LIST pour {} : {} fichier(s)", user.username, list.len());
            let names: Vec<String> = list.into_iter().map(|(n, _)| n).collect();
            let mut response = names.join("\n");
            response.push('\n');
            response
        }
        Err(e) => format!("ERROR {}\n", e),
    }
}
