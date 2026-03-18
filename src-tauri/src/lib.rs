mod commands;
mod mcp;
mod wasm;
mod apps;

use commands::{checkpoint, fs, git, settings, terminal, tyck, worktree};
use apps::commands as tapp_commands;
use apps::manager::create_shared_manager;
use apps::store::AppStore;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&open_folder_item)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::undo(app, None)?)
                .item(&PredefinedMenuItem::redo(app, None)?)
                .separator()
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
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().0.as_str();
            match id {
                "settings" => { let _ = app.emit("open-settings", ()); }
                "open-folder" => { let _ = app.emit("open-folder", ()); }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            fs::read_directory,
            fs::read_file,
            fs::write_file,
            fs::watch_directory,
            fs::stop_watching,
            git::git_status,
            git::git_is_repo,
            git::git_init_repo,
            git::git_revert_files,
            git::git_has_remote,
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
