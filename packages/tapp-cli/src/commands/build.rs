use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub async fn run(path: &str, release: bool, json: bool) -> Result<()> {
    let path = Path::new(path);
    let manifest_path = path.join("manifest.json");

    if !manifest_path.exists() {
        return Err(anyhow!("manifest.json not found in {}", path.display()));
    }

    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
    let app_id = manifest["id"].as_str()
        .ok_or_else(|| anyhow!("manifest.json missing 'id' field"))?;
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(anyhow!("Invalid app id '{}': must be lowercase letters, digits, and hyphens", app_id));
    }

    if !json {
        println!("{} Building {}...", "→".blue().bold(), app_id.cyan());
    }

    let start = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .current_dir(path);

    if release {
        cmd.arg("--release");
    }

    let output = cmd.output()?;

    let elapsed = start.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if json {
            let errors = parse_cargo_errors(&stderr);
            let output = serde_json::json!({
                "success": false,
                "errors": errors,
                "build_time_ms": elapsed.as_millis(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            eprintln!("{} Build failed", "✗".red().bold());
            eprintln!("{}", stderr);
        }

        return Err(anyhow!("Build failed"));
    }

    let profile = if release { "release" } else { "debug" };
    let wasm_name = app_id.replace('-', "_");
    let wasm_path = path
        .join("target")
        .join("wasm32-wasip2")
        .join(profile)
        .join(format!("{}.wasm", wasm_name));

    let wasm_size = if wasm_path.exists() {
        std::fs::metadata(&wasm_path)?.len()
    } else {
        0
    };

    if json {
        let output = serde_json::json!({
            "success": true,
            "app_id": app_id,
            "wasm_path": wasm_path.to_string_lossy(),
            "wasm_size_bytes": wasm_size,
            "build_time_ms": elapsed.as_millis(),
            "release": release,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{} Built {} in {:.2}s", "✓".green().bold(), app_id.cyan(), elapsed.as_secs_f64());
        println!("  WASM: {} ({} bytes)", wasm_path.display(), wasm_size);
    }

    Ok(())
}

fn parse_cargo_errors(stderr: &str) -> Vec<serde_json::Value> {
    let mut errors = Vec::new();

    for line in stderr.lines() {
        if line.starts_with("error[E") {
            let code_end = line.find(']').unwrap_or(0);
            let code = if code_end > 6 {
                &line[6..code_end]
            } else {
                ""
            };

            let message = if code_end > 0 && line.len() > code_end + 2 {
                line[code_end + 2..].trim()
            } else {
                line
            };

            errors.push(serde_json::json!({
                "code": code,
                "message": message,
            }));
        }
    }

    errors
}
