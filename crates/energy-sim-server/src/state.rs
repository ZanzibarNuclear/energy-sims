//! In-memory session store for the remote API.

use std::collections::HashMap;
use std::sync::Arc;

use energy_sim_runtime::Session;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<Mutex<SessionStore>>,
}

#[derive(Default)]
pub struct SessionStore {
    inner: HashMap<String, Session>,
}

impl SessionStore {
    pub fn insert(&mut self, id: String, session: Session) {
        self.inner.insert(id, session);
    }

    pub fn get(&self, id: &str) -> Option<&Session> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.inner.get_mut(id)
    }

    pub fn contains_key(&self, id: &str) -> bool {
        self.inner.contains_key(id)
    }
}
