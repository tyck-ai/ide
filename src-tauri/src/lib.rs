mod commands;
mod mcp;
mod wasm;
mod apps;

use commands::{checkpoint, fs, git, logs, lsp, search, settings, terminal, tyck, worktree};
use lsp::LspManager;
use apps::commands as tapp_commands;
use apps::manager::create_shared_manager;
use apps::store::AppStore;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Emitter, Manager};

static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(1);

// Maps window label → workspace path so we can remove entries when a window closes.
static WINDOW_PATHS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

// Last window to receive focus — set on Focused(true), never cleared on blur,
// so it survives the brief focus loss that happens when the menu bar is clicked.
static LAST_FOCUSED: Mutex<Option<String>> = Mutex::new(None);

fn window_paths() -> std::sync::MutexGuard<'static, HashMap<String, String>> {
    WINDOW_PATHS.lock().unwrap()
}

fn open_workspace_window(app: &tauri::AppHandle, path: Option<String>) {
    let id = WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    let label = format!("workspace-{}", id);

    // Track label → path so on_window_event can find it on close.
    if let Some(ref p) = path {
        window_paths().insert(label.clone(), p.clone());
        settings::add_open_window(p);
    }

    let url = match &path {
        Some(p) => {
            // Percent-encode the path for use as a query param value.
            // Encode UTF-8 bytes (not code points) so multi-byte chars are correct.
            let encoded: String = p.bytes().flat_map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                    vec![b as char]
                }
                b => format!("%{:02X}", b).chars().collect(),
            }).collect();
            format!("/?workspace={}", encoded)
        }
        None => "/".to_string(),
    };

    let title = path.as_deref()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("tyck");

    let _ = tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(1400.0, 900.0)
        .resizable(true)
        .decorations(true)
        .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logs::install_panic_hook();
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let app_manager = rt.block_on(async {
        create_shared_manager().await.expect("Failed to create app manager")
    });

    let app_store = rt.block_on(async {
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tyck")
            .join("store");
        Arc::new(RwLock::new(
            AppStore::new(cache_path).expect("Failed to create app store")
        ))
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_manager)
        .manage(app_store)
        .manage(Arc::new(LspManager::new()))
        .setup(|app| {
            // Start the MCP server so agents can call push_and_create_pr.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match mcp::start_mcp_server(handle).await {
                    Ok(port) => log::info!("[mcp] Ready on port {}", port),
                    Err(e) => log::warn!("[mcp] Failed to start: {}", e),
                }
            });

            let open_folder_item = MenuItemBuilder::new("Open Folder...")
                .accelerator("CmdOrCtrl+O")
                .id("open-folder")
                .build(app)?;

            let settings_item = MenuItemBuilder::new("Settings...")
                .accelerator("CmdOrCtrl+,")
                .id("settings")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "tyck")
                .item(&PredefinedMenuItem::about(app, Some("About tyck"), None)?)
                .separator()
                .item(&settings_item)
                .separator()
                .item(&PredefinedMenuItem::quit(app, Some("Quit tyck"))?)
                .build()?;

            let new_window_item = MenuItemBuilder::new("New Window")
                .accelerator("CmdOrCtrl+Shift+N")
                .id("new-window")
                .build(app)?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&new_window_item)
                .item(&open_folder_item)
                .build()?;

            // Note: undo/redo are intentionally omitted — the native macOS Undo accelerator
            // intercepts Cmd+Z before Monaco's JS keydown handler can receive it, breaking
            // Monaco's own undo stack. Monaco handles Cmd+Z/Y internally via addCommand.
            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&file_menu)
                .item(&edit_menu)
                .build()?;

            app.set_menu(menu)?;

            // Restore previously open workspaces, or open a blank window on first launch.
            // Filter out stale paths (e.g. unmounted drives) to avoid blank broken windows.
            let saved = settings::load_settings_inner();
            let valid: Vec<String> = saved.open_windows.into_iter()
                .filter(|p| std::path::Path::new(p).exists())
                .collect();
            if valid.is_empty() {
                open_workspace_window(app.handle(), None);
            } else {
                for workspace in valid {
                    open_workspace_window(app.handle(), Some(workspace));
                }
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().0.as_str();
            match id {
                "settings" => {
                    // Include the target label as payload so each window can
                    // filter and only open settings if it's the intended target.
                    // Fall back to the first workspace window if no focus event has fired yet.
                    let label = LAST_FOCUSED.lock().unwrap().clone().or_else(|| {
                        app.webview_windows().into_keys()
                            .find(|k| k.starts_with("workspace-"))
                    });
                    if let Some(label) = label {
                        let _ = app.emit("open-settings", &label);
                    }
                }
                "open-folder" => {
                    // Pick a folder from Rust and open it in a new window.
                    // Use a regular thread — blocking_pick_folder is a blocking call
                    // and must not run on the async executor.
                    let app = app.clone();
                    std::thread::spawn(move || {
                        use tauri_plugin_dialog::DialogExt;
                        let path = app.dialog().file().blocking_pick_folder();
                        if let Some(p) = path {
                            open_workspace_window(&app, Some(p.to_string()));
                        }
                    });
                }
                "new-window" => {
                    open_workspace_window(app, None);
                }
                _ => {}
            }
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(true) => {
                    *LAST_FOCUSED.lock().unwrap() = Some(window.label().to_string());
                }
                tauri::WindowEvent::Destroyed => {
                    let label = window.label().to_string();
                    if let Some(path) = window_paths().remove(&label) {
                        settings::remove_open_window(&path);
                    }
                    // Stop any file/git watchers owned by this window.
                    fs::stop_watching_for_window(&label);
                    git::stop_git_watching_for_window(&label);
                    let mut last = LAST_FOCUSED.lock().unwrap();
                    if last.as_deref() == Some(&label) {
                        *last = None;
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            fs::read_directory,
            fs::read_file,
            fs::write_file,
            fs::watch_directory,
            fs::stop_watching,
            fs::create_file,
            fs::create_directory,
            fs::rename_path,
            fs::delete_path,
            fs::reveal_in_file_manager,
            search::search_in_project,
            search::replace_in_project,
            git::git_status,
            git::git_is_repo,
            git::git_init_repo,
            git::git_revert_files,
            git::git_has_remote,
            git::git_add_remote,
            git::git_push_branch,
            git::git_merge_branch,
            git::gh_create_pr,
            git::git_full_status,
            git::git_branches,
            git::git_checkout,
            git::git_create_branch,
            git::git_delete_branch,
            git::git_stage,
            git::git_unstage,
            git::git_stage_all,
            git::git_discard_file,
            git::git_commit,
            git::git_push,
            git::git_pull,
            git::git_fetch,
            git::git_log,
            git::git_show_commit,
            git::git_diff_file,
            git::git_file_content_at_head,
            git::git_stash_list,
            git::git_stash_create,
            git::git_stash_apply,
            git::git_stash_pop,
            git::git_stash_drop,
            git::git_blame_file,
            git::watch_git_directory,
            git::stop_git_watching,
            terminal::spawn_terminal,
            terminal::spawn_agent_terminal,
            terminal::write_terminal,
            terminal::resize_terminal,
            terminal::get_terminal_backlog,
            terminal::kill_terminal,
            tyck::init_tyck,
            tyck::get_session_status_path,
            tyck::list_sessions,
            tyck::reconcile_sessions,
            tyck::get_resume_args,
            tyck::prepare_resume_session,
            tyck::watch_agent_status,
            tyck::stop_agent_status_watch,
            settings::detect_providers,
            settings::load_settings,
            settings::save_settings,
            settings::list_custom_themes,
            settings::save_custom_theme,
            settings::delete_custom_theme,
            settings::export_theme,
            logs::append_log,
            checkpoint::create_checkpoint,
            checkpoint::scan_changes,
            checkpoint::get_checkpoint_summary,
            checkpoint::get_file_diff,
            checkpoint::get_file_at_checkpoint,
            checkpoint::revert_file,
            checkpoint::revert_all,
            checkpoint::finalize_checkpoint,
            checkpoint::stash_agent_changes,
            checkpoint::scan_stashed_changes,
            checkpoint::get_file_from_agent_stash,
            checkpoint::apply_agent_file,
            checkpoint::finalize_review,
            checkpoint::list_reviews,
            worktree::create_worktree,
            worktree::scan_worktree_changes,
            worktree::get_file_from_worktree,
            worktree::get_file_at_base,
            worktree::accept_worktree_file,
            worktree::finalize_session_review,
            worktree::cleanup_worktree,
            worktree::get_worktree_path,
            worktree::worktree_has_pending_changes,
            worktree::debug_file_sync_status,
            worktree::set_worktree_provider_session,
            worktree::find_worktree_by_provider_session,
            worktree::find_worktree_for_resume,
            worktree::discover_provider_session_for_worktree,
            worktree::start_provider_session_discovery,
            worktree::stop_provider_session_discovery,
            worktree::three_way_merge,
            worktree::resolve_conflict,
            worktree::cleanup_stale_worktrees,
            worktree::check_git_version,
            // Language server commands
            lsp::lsp_start,
            lsp::lsp_send,
            lsp::lsp_stop,
            lsp::lsp_stop_all,
            lsp::lsp_list,
            lsp::lsp_check_binary,
            // Tapp extension system commands
            tapp_commands::tapp_list_apps,
            tapp_commands::tapp_install_app,
            tapp_commands::tapp_uninstall_app,
            tapp_commands::tapp_start_app,
            tapp_commands::tapp_stop_app,
            tapp_commands::tapp_enable_app,
            tapp_commands::tapp_disable_app,
            tapp_commands::tapp_get_tool_definitions,
            tapp_commands::tapp_execute_tool,
            tapp_commands::tapp_dispatch_hook,
            tapp_commands::tapp_hot_reload,
            tapp_commands::tapp_enable_watch,
            tapp_commands::tapp_disable_watch,
            tapp_commands::tapp_store_search,
            tapp_commands::tapp_store_get_listing,
            tapp_commands::tapp_store_refresh,
            tapp_commands::tapp_store_check_updates,
            tapp_commands::tapp_store_download,
            tapp_commands::tapp_get_ui,
            tapp_commands::tapp_dispatch_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
