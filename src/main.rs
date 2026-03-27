use crate::server::tcp_server::run_server;

mod server;
mod protocol;
mod service;
mod storage;
mod session;

#[tokio::main]
async fn main() {
    println!("🚀 File Server démarré (Tokio) sur 0.0.0.0:9000");
    run_server().await;
}
