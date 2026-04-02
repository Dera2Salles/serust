use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use tokio::io::BufReader;

pub struct SystHandler;
#[async_trait]
impl Handler for SystHandler {
    fn command(&self) -> &'static str { "SYST" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self, 
        ctx: &mut Context, 
        _: &[&str], 
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>, 
        _: &mut tokio::net::tcp::WriteHalf<'_>
    ) -> HandlerResult {
        ctx.write_line("215 UNIX Type: L8");
        Ok(())
    }
}

pub struct FeatHandler;
#[async_trait]
impl Handler for FeatHandler {
    fn command(&self) -> &'static str { "FEAT" }
    fn requires_auth(&self) -> bool { false }
    async fn handle(
        &self, 
        ctx: &mut Context, 
        _: &[&str], 
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>, 
        _: &mut tokio::net::tcp::WriteHalf<'_>
    ) -> HandlerResult {
        ctx.write_line("211-Features:");
        ctx.write_line(" PASV");
        ctx.write_line(" UTF8");
        ctx.write_line("211 End");
        Ok(())
    }
}

pub struct TypeHandler;
#[async_trait]
impl Handler for TypeHandler {
    fn command(&self) -> &'static str { "TYPE" }
    async fn handle(
        &self, 
        ctx: &mut Context, 
        _: &[&str], 
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>, 
        _: &mut tokio::net::tcp::WriteHalf<'_>
    ) -> HandlerResult {
        ctx.write_line("200 Command okay.");
        Ok(())
    }
}
