use crate::framework::handler::Handler;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Router {
    handlers: HashMap<String, Arc<dyn Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Enregistre un handler pour sa commande.
    /// Utilisation : router.register(UploadHandler);
    pub fn register<H: Handler>(&mut self, handler: H) -> &mut Self {
        self.handlers
            .insert(handler.command().to_uppercase(), Arc::new(handler));
        self
    }

    /// Retourne le handler correspondant à la commande, si existant.
    pub fn resolve(&self, command: &str) -> Option<Arc<dyn Handler>> {
        self.handlers.get(&command.to_uppercase()).cloned()
    }

    /// Retourne toutes les commandes enregistrées (pour HELP, debug…).
    pub fn commands(&self) -> Vec<&str> {
        let mut cmds: Vec<&str> = self.handlers.keys().map(|s| s.as_str()).collect();
        cmds.sort();
        cmds
    }
}
