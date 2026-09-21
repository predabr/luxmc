mod commands;
pub mod core;
pub mod daemon;
pub mod db;
pub mod error;
mod state;
use state::AppState;
use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;

async fn handle_media_request(mut socket: tokio::net::TcpStream, msg: &str) {
    use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

    let query_start = match msg.find("/media?") {
        Some(pos) => pos + 7,
        None => {
            let header = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = socket.write_all(header.as_bytes()).await;
            return;
        }
    };

    let query_line = msg[query_start..].split_whitespace().next().unwrap_or("");
    let mut file_path_str = String::new();
    for param in query_line.split('&') {
        if let Some(val) = param.strip_prefix("path=") {
            if let Ok(decoded) = urlencoding::decode(val) {
                file_path_str = decoded.into_owned();
            }
            break;
        }
    }

    if file_path_str.is_empty() {
        let header = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
        return;
    }

    let path = std::path::Path::new(&file_path_str);
    if !path.exists() || !path.is_file() {
        let header = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
        return;
    }

    let total_len = match tokio::fs::metadata(path).await {
        Ok(m) => m.len(),
        Err(_) => {
            let header = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = socket.write_all(header.as_bytes()).await;
            return;
        }
    };

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let content_type = match ext.as_str() {
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" | "ogg" => "video/ogg",
        "mkv" => "video/x-matroska",
        "gif" => "image/gif",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let mut range: Option<(u64, u64)> = None;
    for line in msg.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("range:") {
            if let Some(eq) = line_lower.find("bytes=") {
                let range_val = line[eq + 6..].trim();
                let parts: Vec<&str> = range_val.split('-').collect();
                if let Ok(start) = parts[0].parse::<u64>() {
                    let end = if parts.len() > 1 && !parts[1].is_empty() {
                        parts[1].parse::<u64>().unwrap_or(total_len.saturating_sub(1))
                    } else {
                        total_len.saturating_sub(1)
                    };
                    if start <= end && end < total_len {
                        range = Some((start, end));
                    }
                }
            }
        }
    }

    let mut file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(_) => {
            let header = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = socket.write_all(header.as_bytes()).await;
            return;
        }
    };

    if let Some((start, end)) = range {
        let chunk_len = end - start + 1;
        let header = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=3600\r\nConnection: close\r\n\r\n",
            content_type, start, end, total_len, chunk_len
        );
        let _ = socket.write_all(header.as_bytes()).await;
        let _ = file.seek(std::io::SeekFrom::Start(start)).await;
        let mut to_read = chunk_len;
        let mut buf = [0u8; 65536];
        while to_read > 0 {
            let limit = buf.len().min(to_read as usize);
            let n = match file.read(&mut buf[..limit]).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if socket.write_all(&buf[..n]).await.is_err() {
                break;
            }
            to_read -= n as u64;
        }
    } else {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: public, max-age=3600\r\nConnection: close\r\n\r\n",
            content_type, total_len
        );
        let _ = socket.write_all(header.as_bytes()).await;
        let mut buf = [0u8; 65536];
        loop {
            let n = match file.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            if socket.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
    }
    let _ = socket.flush().await;
}

async fn handle_cape_request(mut socket: tokio::net::TcpStream) {
    use tokio::io::AsyncWriteExt;
    let mut cape_bytes = crate::core::launcher::get_active_cape_bytes().await;
    if cape_bytes.is_empty() {
        if let Ok(db) = crate::db::shared_db().await {
            use sqlx::Row;
            if let Ok(Some(row)) = sqlx::query("SELECT cape_url FROM accounts WHERE cape_url IS NOT NULL AND cape_url != '' ORDER BY updated_at DESC LIMIT 1")
                .fetch_optional(db.pool())
                .await
            {
                if let Ok(Some(raw_cape)) = row.try_get::<Option<String>, _>("cape_url") {
                    if raw_cape.starts_with("data:image/") {
                        if let Some(pos) = raw_cape.find(',') {
                            use base64::Engine;
                            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&raw_cape[pos + 1..]) {
                                cape_bytes = decoded;
                            }
                        }
                    } else if let Some(local_path) = crate::core::launcher::resolve_local_or_asset_path(&raw_cape) {
                        if let Ok(bytes) = tokio::fs::read(local_path).await {
                            cape_bytes = bytes;
                        }
                    }
                }
            }
        }
    }

    if !cape_bytes.is_empty() {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            cape_bytes.len()
        );
        let _ = socket.write_all(header.as_bytes()).await;
        let _ = socket.write_all(&cape_bytes).await;
    } else {
        let header = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
    }
    let _ = socket.flush().await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .with_ansi(false)
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
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .manage(commands::deep_links::PendingLinks::default())
        .setup(|app| {
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                commands::deep_links::enqueue(&handle, event.urls().into_iter().map(|url| url.to_string()).collect());
            });
            if let Some(urls) = app.deep_link().get_current()? {
                commands::deep_links::enqueue(app.handle(), urls.into_iter().map(|url| url.to_string()).collect());
            }
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            if let Err(error) = app.deep_link().register_all() {
                tracing::warn!(%error, "Não foi possível registrar luxmc://");
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.maximize();
                if let Ok(dyn_img) = image::load_from_memory(include_bytes!("../icons/icon.png")) {
                    let rgba = dyn_img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
                    let _ = window.set_icon(icon);
                } else if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            let handle_overlay = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(listener) = tokio::net::TcpListener::bind("127.0.0.1:49152").await {
                    while let Ok((mut socket, _)) = listener.accept().await {
                        let overlay_ref = handle_overlay.clone();
                        tokio::spawn(async move {
                            use tokio::io::AsyncReadExt;
                            let mut buf = [0u8; 4096];
                            if let Ok(n) = socket.read(&mut buf).await {
                                let msg = String::from_utf8_lossy(&buf[..n]);
                                if msg.starts_with("GET /media") {
                                    handle_media_request(socket, &msg).await;
                                } else if msg.starts_with("GET ")
                                    && (msg.contains("/cape") || msg.contains("/optifine") || msg.contains("luxmc_cape") || msg.contains("/capes/"))
                                {
                                    handle_cape_request(socket).await;
                                } else if msg.contains("TOGGLE") {
                                    crate::commands::system::trigger_overlay_toggle(&overlay_ref);
                                }
                            }
                        });
                    }
                }
            });


            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::deep_links::deep_links_take,
            commands::deep_links::open_portal,
            commands::skins::minecraft_uuid,
            commands::social::social_request,
            commands::system::ping,
            commands::system::app_info,
            commands::system::app_init,
            commands::system::get_system_specs,
            commands::system::client_overlay_close,
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
            commands::lux_account::lux_account_login,
            commands::lux_account::lux_account_sync,
            commands::lux_account::lux_account_logout,
            commands::auth::auth_get_client_id,
            commands::auth::auth_get_tenant_id,
            commands::auth::auth_set_client_id,
            commands::auth::auth_change_skin,
            commands::auth::auth_set_account_cape,
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
            commands::launch::stop_game,
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
            commands::mods::mods_resolve_icons,
            commands::instances::instance_import_modpack,
            commands::instances::instance_repair_modpack,
            commands::instances::instance_cancel_import,
            commands::instances::instance_import_mrpack,
            commands::instances::instance_health_check,
            commands::instances::instance_file_tree,
            commands::instances::instance_export,
            commands::instances::instance_set_notes,
            commands::instances::instance_set_favorite,
            commands::instances::instance_worlds_list,
            commands::instances::instance_world_delete,
            commands::instances::instance_world_import,
            commands::instances::instance_mod_toggle,
            commands::instances::instance_mod_delete,
            commands::instances::instance_mod_add,
            commands::instances::instance_mods_open_folder,
            commands::instances::instance_pack_add,
            commands::instances::instance_pack_delete,
            commands::instances::instance_pack_open_folder,
            commands::instances::instance_install_quick_pack,
            commands::launch_logs::launch_logs_list,
            commands::launch_logs::launch_logs_get,
            commands::launch_logs::launch_logs_search,
            commands::launch_logs::launch_logs_clear,
            commands::launch_log_session::launch_log_open,
            commands::launch_log_session::launch_log_append,
            commands::launch_log_session::launch_log_close,
            commands::instance_tools::jvm_args_validate,
            commands::instance_tools::java_runtime_status,
            commands::doctor::crash_doctor_diagnose,
            commands::instance_tools::crash_summary,
            commands::instance_tools::share_log_mclogs,
            commands::instance_tools::instance_export_zip,
            commands::instance_tools::instance_export_share_code,
            commands::instance_tools::instance_import_share_code,
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
            commands::optimizer::optimizer_trim_memory,
            commands::optimizer::optimizer_native_cpu_profile,
            commands::instances::instance_world_snapshot_create,
            commands::instances::instance_world_snapshots_list,
            commands::instances::instance_world_snapshot_restore,
            commands::instances::instance_world_snapshot_delete,
            commands::instances::instance_world_inspect_region,
            commands::instances::upnp_open_port,
            commands::instances::upnp_close_port,
            commands::importer::importer_detect_launchers,
            commands::importer::importer_execute_import,
            commands::death_tracker::death_tracker_get_last_death,
            commands::shield::instance_shield_scan,
            commands::options_editor::instance_options_get,
            commands::options_editor::instance_options_set,
            commands::options_editor::instance_config_read,
            commands::options_editor::instance_config_write,
            commands::updater::app_perform_update,
            commands::skins::skins_list,
            commands::skins::skins_save,
            commands::skins::skins_delete,
            commands::skins::skins_import_file,
            commands::skins::capes_list,
            commands::skins::capes_save,
            commands::skins::capes_delete,
            commands::skins::gaming_stats_get,
            commands::skins::gaming_stats_save,
            commands::teamwork_preview::teamwork_preview,
            commands::java::java_scan,
            commands::java::java_install,
            commands::java::java_uninstall,
            commands::dep_checker::mods_check_missing_deps,
            commands::dep_checker::mods_install_missing_deps,
            commands::modpack_update::modpack_check_update,
            commands::modpack_update::modpack_update_atomic,
            commands::doctor::doctor_check_instance_conflicts,
            commands::modpack_export::instance_export_modpack,
            commands::keybinds::keybinds_list,
            commands::keybinds::keybinds_update,
            commands::world_backup::instance_backup_world,
            commands::world_backup::instance_list_world_backups,
            commands::p2p::p2p_scan_lan_worlds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
