use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::error::{TappError, TappResult};
use super::manifest::Manifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub registry_url: String,
    pub auto_update: bool,
    pub check_interval_hours: u32,
    pub verify_signatures: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.tyck.dev".to_string(),
            auto_update: true,
            check_interval_hours: 24,
            verify_signatures: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppListing {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub download_url: String,
    pub download_size: u64,
    pub sha256: String,
    pub signature: Option<String>,
    pub public_key: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub downloads: u64,
    pub rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPackage {
    pub manifest: Manifest,
    pub wasm_hash: [u8; 32],
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl SignedPackage {
    pub fn verify(&self, wasm_data: &[u8]) -> TappResult<bool> {
        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }
        // Verify the wasm_hash matches the actual WASM data
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(wasm_data);
        let computed_hash: [u8; 32] = hasher.finalize().into();
        Ok(computed_hash == self.wasm_hash)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub app_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub changelog: Option<String>,
    pub is_critical: bool,
}

pub struct AppStore {
    config: StoreConfig,
    cache_path: PathBuf,
    listings_cache: HashMap<String, AppListing>,
    last_check: Option<std::time::Instant>,
}

impl AppStore {
    pub fn new(cache_path: PathBuf) -> TappResult<Self> {
        std::fs::create_dir_all(&cache_path)?;
        
        Ok(Self {
            config: StoreConfig::default(),
            cache_path,
            listings_cache: HashMap::new(),
            last_check: None,
        })
    }

    pub fn with_config(mut self, config: StoreConfig) -> Self {
        self.config = config;
        self
    }

    pub async fn search(&self, query: &str) -> TappResult<Vec<AppListing>> {
        let results: Vec<AppListing> = self.listings_cache
            .values()
            .filter(|app| {
                app.name.to_lowercase().contains(&query.to_lowercase()) ||
                app.description.to_lowercase().contains(&query.to_lowercase()) ||
                app.id.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .collect();
        
        Ok(results)
    }

    pub async fn get_listing(&self, app_id: &str) -> TappResult<Option<AppListing>> {
        Ok(self.listings_cache.get(app_id).cloned())
    }

    pub async fn refresh_listings(&mut self) -> TappResult<()> {
        self.last_check = Some(std::time::Instant::now());
        Ok(())
    }

    pub async fn download_app(&self, app_id: &str) -> TappResult<PathBuf> {
        let listing = self.listings_cache.get(app_id)
            .ok_or_else(|| TappError::AppNotFound(app_id.to_string()))?;

        let download_path = self.cache_path.join(format!("{}-{}.tapp", app_id, listing.version));

        Ok(download_path)
    }

    pub async fn check_updates(&self, installed_apps: &[(String, String)]) -> TappResult<Vec<UpdateInfo>> {
        let mut updates = Vec::new();

        for (app_id, current_version) in installed_apps {
            if let Some(listing) = self.listings_cache.get(app_id) {
                if version_is_newer(&listing.version, current_version) {
                    updates.push(UpdateInfo {
                        app_id: app_id.clone(),
                        current_version: current_version.clone(),
                        latest_version: listing.version.clone(),
                        download_url: listing.download_url.clone(),
                        changelog: None,
                        is_critical: false,
                    });
                }
            }
        }

        Ok(updates)
    }

    pub async fn verify_package(&self, package_path: &Path) -> TappResult<bool> {
        if !self.config.verify_signatures {
            return Ok(true);
        }

        if !package_path.exists() {
            return Err(TappError::IoError(format!(
                "Package not found: {:?}",
                package_path
            )));
        }

        Ok(true)
    }

    pub fn get_cached_path(&self, app_id: &str, version: &str) -> PathBuf {
        self.cache_path.join(format!("{}-{}.tapp", app_id, version))
    }

    pub fn is_cached(&self, app_id: &str, version: &str) -> bool {
        self.get_cached_path(app_id, version).exists()
    }

    pub async fn clear_cache(&self) -> TappResult<()> {
        for entry in std::fs::read_dir(&self.cache_path)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "tapp") {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    pub fn config(&self) -> &StoreConfig {
        &self.config
    }

    pub fn set_registry_url(&mut self, url: String) {
        self.config.registry_url = url;
    }

    pub fn set_auto_update(&mut self, enabled: bool) {
        self.config.auto_update = enabled;
    }
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for i in 0..latest_parts.len().max(current_parts.len()) {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

pub struct PackageSigner {
    private_key: Option<Vec<u8>>,
}

impl PackageSigner {
    pub fn new() -> Self {
        Self { private_key: None }
    }

    pub fn with_key(mut self, key: Vec<u8>) -> Self {
        self.private_key = Some(key);
        self
    }

    pub fn sign_package(&self, manifest: &Manifest, wasm_data: &[u8]) -> TappResult<SignedPackage> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(wasm_data);
        let wasm_hash: [u8; 32] = hasher.finalize().into();

        let signature = if let Some(ref key) = self.private_key {
            // HMAC-like signature: SHA-256(key || hash)
            let mut sig_hasher = Sha256::new();
            sig_hasher.update(key);
            sig_hasher.update(wasm_hash);
            sig_hasher.finalize().to_vec()
        } else {
            Vec::new()
        };

        Ok(SignedPackage {
            manifest: manifest.clone(),
            wasm_hash,
            signature,
            public_key: Vec::new(),
        })
    }
}

impl Default for PackageSigner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryApi {
    base_url: String,
}

impl RegistryApi {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub fn list_apps_url(&self) -> String {
        format!("{}/apps", self.base_url)
    }

    pub fn get_app_url(&self, app_id: &str) -> String {
        format!("{}/apps/{}", self.base_url, app_id)
    }

    pub fn download_url(&self, app_id: &str) -> String {
        format!("{}/apps/{}/download", self.base_url, app_id)
    }

    pub fn publish_url(&self) -> String {
        format!("{}/apps", self.base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(version_is_newer("1.1.0", "1.0.9"));
        assert!(version_is_newer("2.0.0", "1.9.9"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.1"));
        assert!(!version_is_newer("0.9.0", "1.0.0"));
    }
}
