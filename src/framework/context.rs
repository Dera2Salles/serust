use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Données utilisateur authentifié (stockées en extension).
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
}

/// Conteneur d'extensions typées : n'importe qui peut y stocker/lire une valeur.
/// Ex : Context::set::<AuthenticatedUser>(...) / Context::get::<AuthenticatedUser>()
pub struct Extensions {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set<T: Any + Send + Sync>(&mut self, val: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(val));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }
}

/// Le contexte complet d'une connexion.
pub struct Context {
    /// Adresse du client distant.
    pub peer_addr: SocketAddr,
    /// Adresse locale du serveur (côté socket de contrôle).
    pub local_addr: SocketAddr,
    /// Extensions typées (auth, rate-limit, custom state…).
    pub extensions: Extensions,
    /// Buffer de réponse : les handlers écrivent ici, le framework flush ensuite.
    pub response: Vec<u8>,

    /// FTP State
    pub cwd: String,
    pub data_listener: Option<Arc<tokio::net::TcpListener>>,
    pub data_address: Option<SocketAddr>,
}

impl Context {
    pub fn new(peer_addr: SocketAddr, local_addr: SocketAddr) -> Self {
        Self {
            peer_addr,
            local_addr,
            extensions: Extensions::new(),
            response: Vec::with_capacity(256),
            cwd: "/".to_string(),
            data_listener: None,
            data_address: None,
        }
    }

    /// Écrit une ligne texte dans le buffer de réponse FTP (\r\n).
    pub fn write_line(&mut self, line: &str) {
        self.response.extend_from_slice(line.as_bytes());
        self.response.extend_from_slice(b"\r\n");
    }

    /// Raccourci : répond erreur FTP
    pub fn error(&mut self, code: u16, msg: &str) {
        self.write_line(&format!("{} {}", code, msg));
    }

    /// Vérifie si l'utilisateur est authentifié.
    pub fn is_authenticated(&self) -> bool {
        self.extensions.has::<AuthenticatedUser>()
    }

    /// Retourne l'utilisateur authentifié (panic si absent — utilise is_authenticated avant).
    pub fn user(&self) -> &AuthenticatedUser {
        self.extensions
            .get::<AuthenticatedUser>()
            .expect("user() appelé sans authentification")
    }
}
