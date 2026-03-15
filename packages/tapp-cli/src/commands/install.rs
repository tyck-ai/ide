use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub async fn run(path: &str, json: bool) -> Result<()> {
    let path = Path::new(path);

    let manifest_path = if path.is_file() && path.extension().map(|e| e == "tapp").unwrap_or(false) {
        return Err(anyhow!(".tapp package installation not yet implemented"));
    } else {
        path.join("manifest.json")
    };

    if !manifest_path.exists() {
        return Err(anyhow!("manifest.json not found in {}", path.display()));
    }

    let manifest: serde_json::Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    let app_id = manifest["id"].as_str().ok_or_else(|| anyhow!("manifest.json missing 'id' field"))?;
    // Validate app_id to prevent path traversal
    if app_id.is_empty() || app_id.len() > 64
        || !app_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow!("Invalid app id '{}': must be 1-64 chars of lowercase letters, digits, and hyphens", app_id));
    }
    let app_name = manifest["name"].as_str().unwrap_or(app_id);

    let wasm_name = app_id.replace('-', "_");
    let wasm_path = path
        .join("target")
        .join("wasm32-wasip2")
        .join("release")
        .join(format!("{}.wasm", wasm_name));

    if !wasm_path.exists() {
        return Err(anyhow!(
            "WASM file not found at {}. Run 'tapp build' first.",
            wasm_path.display()
        ));
    }

    let apps_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?
        .join(".tyck")
        .join("apps");

    let install_dir = apps_dir.join(app_id);
    fs::create_dir_all(&install_dir)?;

    // Reject symlinks to prevent following links to sensitive files
    if fs::symlink_metadata(&manifest_path)?.file_type().is_symlink() {
        return Err(anyhow!("manifest.json is a symlink, refusing to install"));
    }
    if fs::symlink_metadata(&wasm_path)?.file_type().is_symlink() {
        return Err(anyhow!("WASM file is a symlink, refusing to install"));
    }
    fs::copy(&manifest_path, install_dir.join("manifest.json"))?;
    fs::copy(&wasm_path, install_dir.join("app.wasm"))?;

    let assets_src = path.join("assets");
    if assets_src.exists() {
        let assets_dest = install_dir.join("assets");
        fs::create_dir_all(&assets_dest)?;
        copy_dir_recursive(&assets_src, &assets_dest)?;
    }

    let data_dir = install_dir.join("data");
    fs::create_dir_all(&data_dir)?;

    update_registry(&apps_dir, app_id, &manifest)?;

    if json {
        let output = serde_json::json!({
            "success": true,
            "app_id": app_id,
            "install_path": install_dir.to_string_lossy(),
            "permissions": manifest["permissions"],
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{} Installed '{}' ({})", "✓".green().bold(), app_name.cyan(), app_id);
        println!("  Location: {}", install_dir.display());

        if let Some(permissions) = manifest["permissions"].as_array() {
            if !permissions.is_empty() {
                println!("  Permissions:");
                for perm in permissions {
                    println!("    - {}", perm.as_str().unwrap_or("unknown"));
                }
            }
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = fs::symlink_metadata(&src_path)?;

        // Skip symlinks to prevent path traversal attacks
        if meta.file_type().is_symlink() {
            continue;
        }

        if meta.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn update_registry(apps_dir: &Path, app_id: &str, manifest: &serde_json::Value) -> Result<()> {
    let registry_path = apps_dir.join("registry.json");
    let install_path = apps_dir.join(app_id);
    let wasm_path = install_path.join("app.wasm");

    let mut registry: serde_json::Value = if registry_path.exists() {
        serde_json::from_str(&fs::read_to_string(&registry_path)?)?
    } else {
        serde_json::json!({ "apps": {} })
    };

    let apps = registry["apps"].as_object_mut()
        .ok_or_else(|| anyhow!("Invalid registry format"))?;

    apps.insert(app_id.to_string(), serde_json::json!({
        "id": app_id,
        "manifest": manifest,
        "install_path": install_path.to_string_lossy(),
        "wasm_path": wasm_path.to_string_lossy(),
        "installed_at": get_iso8601_timestamp(),
        "enabled": true,
    }));

    // Atomic write: temp file + rename to prevent corruption on crash
    let tmp = registry_path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(&registry)?)?;
    fs::rename(&tmp, &registry_path)?;

    Ok(())
}

fn get_iso8601_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
