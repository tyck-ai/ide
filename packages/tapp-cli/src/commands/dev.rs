use anyhow::{anyhow, Result};
use colored::Colorize;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn run(path: &str, json: bool) -> Result<()> {
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
        println!("{} Starting dev mode for {}...", "→".blue().bold(), app_id.cyan());
    }

    super::build::run(&path.to_string_lossy(), false, json).await?;

    if !json {
        println!();
        println!("{} Watching for changes...", "→".blue().bold());
    }

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_secs(1)),
    )?;

    watcher.watch(path.join("src").as_path(), RecursiveMode::Recursive)?;

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(_event) => {
                while rx.try_recv().is_ok() {}

                if !json {
                    println!();
                    println!("{} Change detected, rebuilding...", "→".blue().bold());
                }

                let _ = super::build::run(&path.to_string_lossy(), false, json).await;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}
