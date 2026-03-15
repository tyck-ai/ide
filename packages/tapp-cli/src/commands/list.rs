use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;

pub async fn run(json: bool) -> Result<()> {
    let apps_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?
        .join(".tyck")
        .join("apps");

    let registry_path = apps_dir.join("registry.json");

    if !registry_path.exists() {
        if json {
            let output = serde_json::json!({
                "success": true,
                "apps": [],
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No apps installed.");
        }
        return Ok(());
    }

    let registry: serde_json::Value = serde_json::from_str(&fs::read_to_string(&registry_path)?)?;

    let apps = registry["apps"].as_object()
        .ok_or_else(|| anyhow!("Invalid registry format"))?;

    if json {
        let apps_list: Vec<_> = apps.values().collect();
        let output = serde_json::json!({
            "success": true,
            "apps": apps_list,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        if apps.is_empty() {
            println!("No apps installed.");
            return Ok(());
        }

        println!("{}", "Installed apps:".bold());
        println!();

        for (id, app) in apps {
            let name = app["manifest"]["name"].as_str().unwrap_or(id);
            let version = app["manifest"]["version"].as_str().unwrap_or("0.0.0");
            let enabled = app["enabled"].as_bool().unwrap_or(true);

            let status = if enabled {
                "enabled".green()
            } else {
                "disabled".yellow()
            };

            println!("  {} {} ({})", name.cyan(), format!("v{}", version).dimmed(), status);
            println!("    ID: {}", id.dimmed());

            if let Some(desc) = app["manifest"]["description"].as_str() {
                println!("    {}", desc.dimmed());
            }

            println!();
        }
    }

    Ok(())
}
