use crate::user::service::AuthService;
use crate::user::domain::User;
use crate::common::error::DomainError;
use hyper::{Request, header};
use hyper::body::Incoming;
use std::sync::Arc;
use base64::{engine::general_purpose, Engine as _};

pub async fn authenticate_http(
    req: &Request<Incoming>,
    auth: Arc<AuthService>,
) -> Result<User, DomainError> {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    if let Some(auth_val) = auth_header {
        let auth_str = auth_val.to_str().unwrap_or("");
        if auth_str.starts_with("Basic ") {
            let encoded = &auth_str[6..];
            if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let identifier = parts[0];
                        let password = parts[1];
                        tracing::debug!("Authenticating user: {}", identifier);
                        return auth.login(identifier, password).await;
                    }
                }
            }
        }
    }
    tracing::warn!("Authentication failed: Missing or invalid Authorization header");
    Err(DomainError::InvalidCredentials)
}
