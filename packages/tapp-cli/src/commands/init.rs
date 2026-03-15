use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::templates;

pub async fn run(name: &str, template: &str, json: bool) -> Result<()> {
    // Reject path traversal in name
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(anyhow!("Invalid app name '{}': must not contain path separators or '..'", name));
    }

    let path = Path::new(name);

    if path.exists() {
        return Err(anyhow!("Directory '{}' already exists", name));
    }

    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("assets"))?;

    let id = name.to_lowercase().replace(' ', "-");
    let id = id.chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect::<String>();

    let manifest = templates::manifest(&id, name);
    fs::write(path.join("manifest.json"), manifest)?;

    let cargo_toml = templates::cargo_toml(&id);
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    let lib_rs = match template {
        "minimal" => templates::lib_rs_minimal(),
        "tool" => templates::lib_rs_tool(),
        "full" => templates::lib_rs_full(),
        "sidebar" => templates::lib_rs_sidebar(),
        _ => templates::lib_rs_minimal(),
    };
    fs::write(path.join("src").join("lib.rs"), lib_rs)?;

    let gitignore = templates::gitignore();
    fs::write(path.join(".gitignore"), gitignore)?;

    let readme = templates::readme(name);
    fs::write(path.join("README.md"), readme)?;

    if json {
        let output = serde_json::json!({
            "success": true,
            "app_id": id,
            "path": path.to_string_lossy(),
            "template": template,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{} Created Tapp app '{}' using '{}' template", "✓".green().bold(), name.cyan(), template);
        println!();
        println!("  Next steps:");
        println!("    cd {}", name);
        println!("    tapp dev");
        println!();
    }

    Ok(())
}
