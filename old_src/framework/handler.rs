// src/framework/handler.rs
//
// Trait Handler : contrat que tout handler de commande doit implémenter.
// Pour ajouter une commande au serveur : implémenter Handler, enregistrer dans le Router.

use crate::framework::context::Context;
use async_trait::async_trait;

/// Résultat d'un handler.
pub type HandlerResult = anyhow::Result<()>;

/// Trait principal du framework.
/// Chaque commande TCP (UPLOAD, DOWNLOAD, LIST, custom…) est un Handler.
///
/// # Exemple — Ajouter une commande PING :
/// ```rust
/// pub struct PingHandler;
///
/// #[async_trait]
/// impl Handler for PingHandler {
///     fn command(&self) -> &'static str { "PING" }
///     fn requires_auth(&self) -> bool { false }
///
///     async fn handle(&self, ctx: &mut Context, _args: &[&str]) -> HandlerResult {
///         ctx.write_line("PONG");
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Handler: Send + Sync + 'static {
    /// Nom de la commande en MAJUSCULES (ex: "UPLOAD", "PING", "STATUS").
    fn command(&self) -> &'static str;

    /// Si true, le framework vérifie l'authentification avant d'appeler handle().
    fn requires_auth(&self) -> bool {
        true
    }

    /// Logique du handler.
    /// `args` = les tokens après le nom de la commande.
    /// Ex : "UPLOAD photo.jpg 1024" → args = ["photo.jpg", "1024"]
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        raw_reader: &mut tokio::io::BufReader<tokio::net::tcp::ReadHalf<'_>>,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult;
}
