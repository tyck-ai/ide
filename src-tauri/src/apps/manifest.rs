use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

use super::error::{TappError, TappResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    #[serde(rename = "fs:read")]
    FsRead,
    #[serde(rename = "fs:write")]
    FsWrite,
    #[serde(rename = "fs:system")]
    FsSystem,
    #[serde(rename = "network:fetch")]
    NetworkFetch,
    #[serde(rename = "network:unrestricted")]
    NetworkUnrestricted,
    #[serde(rename = "storage:session")]
    StorageSession,
    #[serde(rename = "storage:persistent")]
    StoragePersistent,
    #[serde(rename = "agent:inject")]
    AgentInject,
    #[serde(rename = "agent:tools")]
    AgentTools,
    #[serde(rename = "agent:hooks")]
    AgentHooks,
    #[serde(rename = "agent:spawn")]
    AgentSpawn,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::FsRead => "fs:read",
            Permission::FsWrite => "fs:write",
            Permission::FsSystem => "fs:system",
            Permission::NetworkFetch => "network:fetch",
            Permission::NetworkUnrestricted => "network:unrestricted",
            Permission::StorageSession => "storage:session",
            Permission::StoragePersistent => "storage:persistent",
            Permission::AgentInject => "agent:inject",
            Permission::AgentTools => "agent:tools",
            Permission::AgentHooks => "agent:hooks",
            Permission::AgentSpawn => "agent:spawn",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default = "default_layout")]
    pub layout: LayoutMode,
}

fn default_layout() -> LayoutMode {
    LayoutMode::Full
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    #[default]
    Full,
    Sidebar,
    Panel,
    Modal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

impl Manifest {
    pub fn load(path: &Path) -> TappResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest = serde_json::from_str(&content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> TappResult<()> {
        if self.id.is_empty() || self.id.len() > 64 {
            return Err(TappError::ManifestError("id must be 1-64 characters".to_string()));
        }
        if !self.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(TappError::ManifestError(
                "id must contain only lowercase letters, numbers, and hyphens".to_string(),
            ));
        }
        if self.name.is_empty() {
            return Err(TappError::ManifestError("name is required".to_string()));
        }
        if self.name.len() > 50 {
            return Err(TappError::ManifestError("name must be 50 characters or less".to_string()));
        }
        if self.version.is_empty() || self.version.len() > 20 {
            return Err(TappError::ManifestError("version must be 1-20 characters".to_string()));
        }
        if self.permissions.len() > 20 {
            return Err(TappError::ManifestError("too many permissions (max 20)".to_string()));
        }
        if self.description.as_ref().map(|d| d.len() > 500).unwrap_or(false) {
            return Err(TappError::ManifestError("description must be 500 characters or less".to_string()));
        }
        Ok(())
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn permissions_set(&self) -> HashSet<Permission> {
        self.permissions.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let valid = Manifest {
            id: "my-app".to_string(),
            name: "My App".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            permissions: vec![Permission::StorageSession],
            ui: UiConfig::default(),
            network: NetworkConfig::default(),
        };
        assert!(valid.validate().is_ok());

        let invalid_id = Manifest {
            id: "My_App".to_string(),
            ..valid.clone()
        };
        assert!(invalid_id.validate().is_err());
    }
}
