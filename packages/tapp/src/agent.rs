use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub type SessionId = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpawnOptions {
    pub provider: Option<String>,
    pub cwd: Option<String>,
    pub system_prompt: Option<String>,
    pub visible: bool,
}

impl SpawnOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn visible(mut self) -> Self {
        self.visible = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
}

pub struct AgentSession {
    _private: (),
}

impl AgentSession {
    pub fn spawn(_options: SpawnOptions) -> Result<Self> {
        Err(Error::Runtime("Agent spawning not yet implemented".to_string()))
    }

    pub fn send(&self, _text: &str) -> Result<()> {
        Err(Error::Runtime("Agent send not yet implemented".to_string()))
    }

    pub fn get_output(&self) -> Result<Option<String>> {
        Err(Error::Runtime("Agent get_output not yet implemented".to_string()))
    }

    pub fn kill(self) -> Result<()> {
        Err(Error::Runtime("Agent kill not yet implemented".to_string()))
    }

    pub fn inject_to_active(_text: &str) -> Result<()> {
        Err(Error::Runtime("Agent inject not yet implemented".to_string()))
    }
}
