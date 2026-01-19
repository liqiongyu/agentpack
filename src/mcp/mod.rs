mod confirm;
mod server;
mod tools;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AgentpackMcp {
    confirm_tokens: Arc<Mutex<confirm::ConfirmTokenStore>>,
}

impl AgentpackMcp {
    pub fn new() -> Self {
        Self {
            confirm_tokens: Arc::new(Mutex::new(confirm::ConfirmTokenStore::default())),
        }
    }
}

impl Default for AgentpackMcp {
    fn default() -> Self {
        Self::new()
    }
}
