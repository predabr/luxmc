mod commands;
pub mod core;
pub mod db;
pub mod error;
mod state;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    if crate::core::mods::curseforge::api_key().is_some() {
        tracing::info!("CurseForge API: enabled");
    } else {
        tracing::warn!("CurseForge API: disabled (no API key found)");
    }

    if let Err(e) = db::shared_db().await {
        tracing::error!(error = %e, "failed to initialise database");
    }

    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(dyn_img) = image::load_from_memory(include_bytes!("../icons/icon.png")) {
                    let rgba = dyn_img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
                    let _ = window.set_icon(icon);
                } else if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::system::ping,
            commands::system::app_info,
            commands::system::app_init,
            commands::system::get_system_specs,
            commands::env::env_check,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::auth::auth_begin,
            commands::auth::auth_complete,
            commands::auth::auth_login,
            commands::auth::auth_refresh,
            commands::auth::auth_accounts,
            commands::auth::auth_switch_account,
            commands::auth::auth_remove,
            commands::auth::auth_dev_login,
            commands::auth::auth_offline_login,
            commands::auth::auth_get_client_id,
            commands::auth::auth_get_tenant_id,
            commands::auth::auth_set_client_id,
            commands::auth::auth_change_skin,
            commands::profiles::profiles_list,
            commands::profiles::profiles_get,
            commands::profiles::profiles_create,
            commands::profiles::profiles_update,
            commands::profiles::profiles_delete,
            commands::instances::instances_list,
            commands::instances::instances_duplicate,
            commands::instances::instances_open_folder,
            commands::instances::instances_screenshots,
            commands::instances::screenshot_delete,
            commands::instances::screenshots_open_folder,
            commands::versions::versions_list,
            commands::versions::versions_detail,
            commands::versions::versions_download,
            commands::versions::versions_check_installed,
            commands::launch::launch_game,
            commands::loaders::loaders_versions,
            commands::mods::mods_search,
            commands::mods::mods_search_typed,
            commands::mods::mods_versions,
            commands::mods::mods_project_details,
            commands::mods::mods_list,
            commands::mods::mods_install,
            commands::mods::mods_install_with_deps,
            commands::mods::mods_remove,
            commands::mods::mods_check_updates,
            commands::mods::mods_update,
            commands::mods::mods_download_to_temp,
            commands::mods::curseforge_status,
            commands::mods::curseforge_get_key,
            commands::mods::curseforge_set_key,
            commands::mods::curseforge_remove_key,
            commands::mods::curseforge_validate_key,
            commands::mods::mods_resolve_names,
            commands::instances::instance_import_modpack,
            commands::instances::instance_import_mrpack,
            commands::instances::instance_health_check,
            commands::instances::instance_file_tree,
            commands::instances::instance_export,
            commands::instances::instance_set_notes,
            commands::instances::instance_set_favorite,
            commands::instances::instance_worlds_list,
            commands::instances::instance_world_delete,
            commands::instances::instance_mod_toggle,
            commands::instances::instance_mod_delete,
            commands::instances::instance_mod_add,
            commands::instances::instance_mods_open_folder,
            commands::instances::instance_pack_add,
            commands::instances::instance_pack_delete,
            commands::instances::instance_pack_open_folder,
            commands::launch_logs::launch_logs_list,
            commands::launch_logs::launch_logs_get,
            commands::launch_logs::launch_logs_search,
            commands::launch_logs::launch_logs_clear,
            commands::launch_log_session::launch_log_open,
            commands::launch_log_session::launch_log_append,
            commands::launch_log_session::launch_log_close,
            commands::instance_tools::jvm_args_validate,
            commands::instance_tools::java_runtime_status,
            commands::instance_tools::crash_summary,
            commands::instance_tools::share_log_mclogs,
            commands::instance_tools::instance_export_zip,
            commands::instance_tools::instance_backup_saves,
            commands::instance_tools::instance_restore_saves,
            commands::instance_tools::instance_repair,
            commands::instance_tools::instance_disk_usage,
            commands::instance_tools::version_repair,
            commands::storage::storage_breakdown,
            commands::changelog::changelog_get,
            commands::instance_icons::storage_total,
            commands::instance_icons::directory_exists,
            commands::instance_icons::ensure_directory,
            commands::instance_icons::read_text_file,
            commands::instance_icons::write_text_file,
            commands::instance_icons::delete_file_or_dir,
            commands::servers::server_ping,
            commands::servers::server_add,
            commands::servers::server_list,
            commands::servers::server_remove,
            commands::servers::server_favorites_list,
            commands::servers::server_favorite,
            commands::p2p::p2p_get_local_info,
            commands::p2p::p2p_get_host_link,
            commands::p2p::p2p_start_listener,
            commands::p2p::p2p_send_message,
            commands::discord::discord_set_activity,
            commands::discord::discord_clear_activity,
            commands::optimizer::optimizer_get_flags,
            commands::optimizer::optimizer_get_perf_pack,
            commands::optimizer::optimizer_install_perf_pack,
            commands::optimizer::optimizer_detect_gpu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
