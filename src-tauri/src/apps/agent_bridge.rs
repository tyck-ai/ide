use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use super::error::{TappError, TappResult};

pub type SessionId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnOptions {
    pub provider: Option<String>,
    pub cwd: Option<String>,
    pub system_prompt: Option<String>,
    pub visible: bool,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            provider: None,
            cwd: None,
            system_prompt: None,
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppAgentSession {
    pub app_id: String,
    pub session_id: SessionId,
    pub visible: bool,
    pub active: bool,
    pub created_at: Instant,
}

pub struct AgentBridge {
    sessions: HashMap<SessionId, AppAgentSession>,
    app_sessions: HashMap<String, Vec<SessionId>>,
}

impl AgentBridge {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            app_sessions: HashMap::new(),
        }
    }

    pub fn register_session(&mut self, app_id: &str, session_id: SessionId, visible: bool) {
        let session = AppAgentSession {
            app_id: app_id.to_string(),
            session_id: session_id.clone(),
            visible,
            active: true,
            created_at: Instant::now(),
        };

        self.sessions.insert(session_id.clone(), session);
        self.app_sessions
            .entry(app_id.to_string())
            .or_insert_with(Vec::new)
            .push(session_id);
    }

    pub fn unregister_session(&mut self, session_id: &str) -> Option<AppAgentSession> {
        if let Some(session) = self.sessions.remove(session_id) {
            if let Some(sessions) = self.app_sessions.get_mut(&session.app_id) {
                sessions.retain(|s| s != session_id);
            }
            Some(session)
        } else {
            None
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<&AppAgentSession> {
        self.sessions.get(session_id)
    }

    pub fn get_app_sessions(&self, app_id: &str) -> Vec<&AppAgentSession> {
        self.app_sessions
            .get(app_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.sessions.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn mark_inactive(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.active = false;
        }
    }

    pub fn cleanup_app(&mut self, app_id: &str) {
        if let Some(session_ids) = self.app_sessions.remove(app_id) {
            for id in session_ids {
                self.sessions.remove(&id);
            }
        }
    }

    /// Remove sessions older than the given duration.
    pub fn cleanup_expired(&mut self, max_age: std::time::Duration) {
        let expired: Vec<SessionId> = self.sessions.iter()
            .filter(|(_, s)| s.created_at.elapsed() > max_age)
            .map(|(id, _)| id.clone())
            .collect();
        for id in expired {
            self.unregister_session(&id);
        }
    }
}

impl Default for AgentBridge {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedAgentBridge = Arc<RwLock<AgentBridge>>;

pub fn create_shared_bridge() -> SharedAgentBridge {
    Arc::new(RwLock::new(AgentBridge::new()))
}
