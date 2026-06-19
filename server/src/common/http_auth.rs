use crate::common::error::DomainError;
use crate::user::domain::User;
use crate::user::service::AuthService;
use base64::{engine::general_purpose, Engine as _};
use hyper::body::Incoming;
use hyper::{header, Request};
use std::sync::Arc;

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

                        // Try as email first (existing behaviour)
                        if identifier.contains('@') {
                            return auth.login(identifier, password).await;
                        }

                        // Identifier looks like a username: look up the email then login
                        match auth.get_user_by_username(identifier).await {
                            Ok(Some(user)) => {
                                // Verify password using the stored hash
                                let hash = AuthService::hash_password(password);
                                if user.password_hash != hash {
                                    return Err(DomainError::InvalidCredentials);
                                }
                                // Re-use the email to go through the full login path
                                // (which checks is_active)
                                return auth.login(&user.email, password).await;
                            }
                            _ => return Err(DomainError::InvalidCredentials),
                        }
                    }
                }
            }
        }
    }
    tracing::warn!("Authentication failed: Missing or invalid Authorization header");
    Err(DomainError::InvalidCredentials)
}
