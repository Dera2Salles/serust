use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::protocol::parser::{Command, parse_command};
use crate::session::session::Session;

pub async fn run_server() {
    let listener = TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("Impossible de binder le port 9000");

    let sessions = Arc::new(Mutex::new(Vec::<Session>::new()));

    println!("File Server (Tokio) en écoute sur 0.0.0.0:9000");

    loop {
        let (stream, addr) = listener.accept().await.expect("Accept failed");

        println!("Client connecté : {}", addr);

        let sessions_clone = Arc::clone(&sessions);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, sessions_clone).await {
                eprintln!("Erreur avec le client {} : {}", addr, e);
            }
            println!("Client {} déconnecté", addr);
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    _sessions: Arc<Mutex<Vec<Session>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::new();

    loop {
        buffer.clear();

        let bytes_read = reader.read_until(b'\n', &mut buffer).await?;

        if bytes_read == 0 {
            break;
        }

        let command_line = String::from_utf8_lossy(&buffer).trim().to_string();

        if command_line.is_empty() {
            continue;
        }

        println!("Commande reçue : {}", command_line);

        match parse_command(&command_line) {
            Ok(cmd) => {
                match cmd {
                    Command::Login { username, password } => {
                        // TODO: implémenter l'authentification réelle
                        println!("LOGIN → username: {}, password: {}", username, password);
                        let response = "LOGIN_OK\n";
                        reader.get_mut().write_all(response.as_bytes()).await?;
                    }
                    Command::List => {
                        println!("LIST demandé");
                        // TODO: implémenter LIST
                        let response = "LIST_OK\nfichier1.txt\nfichier2.pdf\n";
                        reader.get_mut().write_all(response.as_bytes()).await?;
                    }
                    Command::Upload { filename, size } => {
                        println!("UPLOAD → {} ({} bytes)", filename, size);
                        // TODO: recevoir les données binaires
                        let response = "UPLOAD_READY\n";
                        reader.get_mut().write_all(response.as_bytes()).await?;
                    }
                    Command::Download { filename } => {
                        println!("DOWNLOAD → {}", filename);
                        // TODO: implémenter DOWNLOAD
                        let response = "DOWNLOAD_START\nSIZE 1024\n";
                        reader.get_mut().write_all(response.as_bytes()).await?;
                    }
                    Command::Quit => {
                        println!("Client a demandé QUIT");
                        let response = "BYE\n";
                        reader.get_mut().write_all(response.as_bytes()).await?;
                        break;
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("ERROR {}\n", e);
                reader.get_mut().write_all(error_msg.as_bytes()).await?;
            }
        }
    }

    Ok(())
}
