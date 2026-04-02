// src/middlewares/auth_middleware.rs
//
// Middleware d'authentification :
// - Laisse passer LOGIN sans vérification
// - Bloque toutes les autres commandes si non authentifié

use crate::framework::context::Context;
use crate::middlewares::middleware::{Middleware, MiddlewareResult};
use async_trait::async_trait;

pub struct AuthMiddleware;

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, ctx: &mut Context, command: &str) -> MiddlewareResult {
        if command == "USER" || command == "PASS" || command == "QUIT" || command == "FEAT" || command == "SYST" || command == "AUTH" {
            return MiddlewareResult::Continue;
        }
        // Toutes les autres commandes nécessitent d'être authentifié
        if !ctx.is_authenticated() {
            ctx.error(530, "Not logged in.");
            return MiddlewareResult::Stop;
        }
        MiddlewareResult::Continue
    }
}
