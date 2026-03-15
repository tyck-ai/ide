use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;

pub async fn run(id: &str, json: bool) -> Result<()> {
    if id.is_empty() || id.len() > 64
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow!("Invalid app id '{}': must be 1-64 chars of lowercase letters, digits, and hyphens", id));
    }

    let apps_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?
        .join(".tyck")
        .join("apps");

    let app_dir = apps_dir.join(id);
    let registry_path = apps_dir.join("registry.json");

    if !app_dir.exists() {
        return Err(anyhow!("App '{}' is not installed", id));
    }

    let app_name = if let Ok(manifest_str) = fs::read_to_string(app_dir.join("manifest.json")) {
        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_str) {
            manifest["name"].as_str().unwrap_or(id).to_string()
        } else {
            id.to_string()
        }
    } else {
        id.to_string()
    };

    fs::remove_dir_all(&app_dir)?;

    if registry_path.exists() {
        let mut registry: serde_json::Value = serde_json::from_str(&fs::read_to_string(&registry_path)?)?;

        if let Some(apps) = registry["apps"].as_object_mut() {
            apps.remove(id);
        }

        let tmp = registry_path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&registry)?)?;
        fs::rename(&tmp, &registry_path)?;
    }

    if json {
        let output = serde_json::json!({
            "success": true,
            "app_id": id,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{} Uninstalled '{}' ({})", "✓".green().bold(), app_name.cyan(), id);
    }

    Ok(())
}
