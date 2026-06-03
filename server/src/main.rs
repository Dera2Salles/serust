#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tcp_file_server::run_server().await
}
