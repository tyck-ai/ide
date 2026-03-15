use anyhow::{anyhow, Result};
use colored::Colorize;

pub async fn run(id: &str, json: bool) -> Result<()> {
    let apps_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?
        .join(".tyck")
        .join("apps");

    let app_dir = apps_dir.join(id);

    if !app_dir.exists() {
        return Err(anyhow!("App '{}' is not installed", id));
    }

    if json {
        let output = serde_json::json!({
            "success": true,
            "message": "App launch requested. Open Tyck and press Cmd+Shift+A to view apps.",
            "app_id": id,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{} App '{}' is installed", "✓".green().bold(), id.cyan());
        println!();
        println!("  To run the app:");
        println!("    1. Open Tyck");
        println!("    2. Press {} to open the app launcher", "Cmd+Shift+A".cyan());
        println!("    3. Select '{}'", id);
        println!();
    }

    Ok(())
}
