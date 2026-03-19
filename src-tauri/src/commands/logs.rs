use chrono::{Duration, Local};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn logs_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".tyck").join("logs")
}

fn today_log_path() -> PathBuf {
    let date = Local::now().format("%Y-%m-%d").to_string();
    logs_dir().join(format!("tyck-{}.log", date))
}

fn prune_old_logs() {
    let dir = logs_dir();
    let cutoff = (Local::now() - Duration::days(7)).date_naive();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "log").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(date_str) = stem.strip_prefix("tyck-") {
                        if let Ok(date) =
                            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        {
                            if date < cutoff {
                                let _ = fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Internal write used by the panic hook (no Result, no Tauri plumbing).
pub fn append_log_raw(level: &str, message: &str) {
    let dir = logs_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = today_log_path();
    if !path.exists() {
        prune_old_logs();
    }
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
    let line = format!("[{}] [{}] {}\n", timestamp, level.to_uppercase(), message);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[tauri::command]
pub fn append_log(level: String, message: String) -> Result<(), String> {
    let dir = logs_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = today_log_path();
    if !path.exists() {
        prune_old_logs();
    }
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
    let line = format!("[{}] [{}] {}\n", timestamp, level.to_uppercase(), message);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())
}

// ── log crate backend ────────────────────────────────────────────────────────

struct TyckLogger;

impl log::Log for TyckLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        match metadata.level() {
            // error and warn from any crate
            log::Level::Error | log::Level::Warn => true,
            // info only from our own code
            log::Level::Info => {
                let t = metadata.target();
                t.starts_with("tyck") || t.starts_with("mcp")
            }
            // debug / trace — ignore
            _ => false,
        }
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                log::Level::Error => "error",
                log::Level::Warn  => "warn",
                log::Level::Info  => "info",
                log::Level::Debug => "debug",
                log::Level::Trace => "trace",
            };
            let msg = format!("[rust:{}] {}", record.target(), record.args());
            append_log_raw(level, &msg);
        }
    }

    fn flush(&self) {}
}

/// Register our log backend and install the panic hook.
/// Must be called once at startup before any logging occurs.
pub fn install_panic_hook() {
    // Register TyckLogger as the global log backend. Ignore the error if
    // a logger was already set (e.g. in tests or hot-reload scenarios).
    static LOGGER: TyckLogger = TyckLogger;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);

    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_string()
        };
        let location = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        append_log_raw("error", &format!("PANIC: {}{}", msg, location));
    }));
}
