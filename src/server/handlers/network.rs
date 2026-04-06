use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tracing::error;

pub struct PasvHandler;
#[async_trait]
impl Handler for PasvHandler {
    fn command(&self) -> &'static str {
        "PASV"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        _: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        match TcpListener::bind("0.0.0.0:0").await {
            Ok(listener) => {
                let addr = listener.local_addr().unwrap();
                let port = addr.port();
                let p1 = port / 256;
                let p2 = port % 256;

                let local_ip = ctx.local_addr.ip().to_string();
                let ip_str = local_ip.replace(".", ",");

                ctx.data_listener = Some(Arc::new(listener));
                ctx.write_line(&format!(
                    "227 Entering Passive Mode ({},{},{}).",
                    ip_str, p1, p2
                ));
            }
            Err(e) => {
                error!("Failed to bind PASV listener: {}", e);
                ctx.error(425, "Can't open data connection.");
            }
        }
        Ok(())
    }
}

pub struct PortHandler;
#[async_trait]
impl Handler for PortHandler {
    fn command(&self) -> &'static str {
        "PORT"
    }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.is_empty() {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }
        let parts: Vec<&str> = args[0].split(',').collect();
        if parts.len() != 6 {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let p1: u16 = parts[4].parse().unwrap_or(0);
        let p2: u16 = parts[5].parse().unwrap_or(0);
        let port = (p1 * 256) + p2;
        let ip_str = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);

        if let Ok(ip) = Ipv4Addr::from_str(&ip_str) {
            ctx.data_address = Some(SocketAddr::new(ip.into(), port));
            ctx.write_line("200 Command okay.");
        } else {
            ctx.error(501, "Syntax error in parameters or arguments.");
        }
        Ok(())
    }
}
