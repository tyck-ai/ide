use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::error::{TappError, TappResult};
use super::manifest::Manifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub manifest: Manifest,
    pub install_path: PathBuf,
    pub wasm_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegistryData {
    pub apps: HashMap<String, InstalledApp>,
}

pub struct AppRegistry {
    data: RegistryData,
    registry_path: PathBuf,
}

impl AppRegistry {
    pub fn new(base_path: &Path) -> TappResult<Self> {
        let registry_path = base_path.join("registry.json");
        let data = if registry_path.exists() {
            let content = std::fs::read_to_string(&registry_path)?;
            serde_json::from_str(&content)?
        } else {
            RegistryData::default()
        };

        Ok(Self { data, registry_path })
    }

    pub fn save(&self) -> TappResult<()> {
        if let Some(parent) = self.registry_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.data)?;
        // Atomic write: write to a temp file then rename, so a power loss
        // mid-write can't corrupt the registry.
        let tmp = self.registry_path.with_extension("json.tmp");
        std::fs::write(&tmp, &content)?;
        if let Err(e) = std::fs::rename(&tmp, &self.registry_path) {
            let _ = std::fs::remove_file(&tmp); // Best-effort cleanup
            return Err(e.into());
        }
        Ok(())
    }

    pub fn get(&self, app_id: &str) -> Option<&InstalledApp> {
        self.data.apps.get(app_id)
    }

    pub fn list(&self) -> Vec<&InstalledApp> {
        self.data.apps.values().collect()
    }

    pub fn list_enabled(&self) -> Vec<&InstalledApp> {
        self.data.apps.values().filter(|a| a.enabled).collect()
    }

    pub fn install(&mut self, manifest: Manifest, install_path: PathBuf, wasm_path: PathBuf) -> TappResult<()> {
        let app = InstalledApp {
            id: manifest.id.clone(),
            manifest,
            install_path,
            wasm_path,
            installed_at: chrono::Utc::now(),
            enabled: true,
        };
        self.data.apps.insert(app.id.clone(), app);
        self.save()
    }

    pub fn uninstall(&mut self, app_id: &str) -> TappResult<()> {
        if self.data.apps.remove(app_id).is_none() {
            return Err(TappError::AppNotFound(app_id.to_string()));
        }
        self.save()
    }

    pub fn enable(&mut self, app_id: &str) -> TappResult<()> {
        let app = self.data.apps.get_mut(app_id)
            .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;
        app.enabled = true;
        self.save()
    }

    pub fn disable(&mut self, app_id: &str) -> TappResult<()> {
        let app = self.data.apps.get_mut(app_id)
            .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;
        app.enabled = false;
        self.save()
    }

    pub fn is_installed(&self, app_id: &str) -> bool {
        self.data.apps.contains_key(app_id)
    }

    pub fn is_enabled(&self, app_id: &str) -> bool {
        self.data.apps.get(app_id).map(|a| a.enabled).unwrap_or(false)
    }
}

pub fn get_apps_base_path() -> TappResult<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| TappError::IoError("Could not find home directory".to_string()))?;
    Ok(home.join(".tyck").join("apps"))
}

pub fn get_app_data_path(app_id: &str) -> TappResult<PathBuf> {
    let base = get_apps_base_path()?;
    let path = base.join(app_id).join("data");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
