mod commands;
pub mod network;
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

    for line in msg.lines() {
        if let Some((name, value)) = line.split_once(':') {
            let name_trim = name.trim().to_ascii_lowercase();
            let val_trim = value.trim();
            if name_trim == "origin" || name_trim == "referer" {
                let allowed = val_trim.starts_with("tauri://")
                    || val_trim.starts_with("http://tauri.localhost")
                    || val_trim.starts_with("https://tauri.localhost")
                    || val_trim.starts_with("http://localhost")
                    || val_trim.starts_with("http://127.0.0.1");
                if !allowed {
                    let header = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = socket.write_all(header.as_bytes()).await;
                    return;
                }
            }
        }
    }

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
        _ => {
            let _ = socket.write_all(b"HTTP/1.1 415 Unsupported Media Type\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
            return;
        },
    };

    let canonical = match path.canonicalize() {
        Ok(c) => c,
        Err(_) => {
            let header = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = socket.write_all(header.as_bytes()).await;
            return;
        }
    };

    if !canonical.is_file() {
        let header = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
        return;
    }

    let canon_lower = canonical.to_string_lossy().to_lowercase();
    let sensitive_markers = [
        "/.ssh", "\\.ssh",
        "/.gnupg", "\\.gnupg",
        "/.aws", "\\.aws",
        ".env", "id_rsa", "id_ed25519",
        "/etc/", "\\etc\\",
        "shadow", "passwd",
        "luxmc.db", "keyring", "token", "credentials"
    ];
    if sensitive_markers.iter().any(|m| canon_lower.contains(m)) {
        let header = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
        return;
    }

    let mut allowed = false;
    if let Some(proj) = directories::ProjectDirs::from("io", "github", "Luxmc") {
        if let Ok(can) = proj.data_dir().canonicalize() {
            if canonical.starts_with(&can) { allowed = true; }
        }
        if let Ok(can) = proj.config_dir().canonicalize() {
            if canonical.starts_with(&can) { allowed = true; }
        }
        if let Ok(can) = proj.cache_dir().canonicalize() {
            if canonical.starts_with(&can) { allowed = true; }
        }
    }
    if !allowed {
        if let Some(user) = directories::UserDirs::new() {
            if let Ok(can) = user.home_dir().canonicalize() {
                if canonical.starts_with(&can) { allowed = true; }
            }
        }
    }
    if !allowed {
        if let Ok(cwd) = std::env::current_dir().and_then(|c| c.canonicalize()) {
            if canonical.starts_with(&cwd) { allowed = true; }
        }
    }

    if !allowed {
        let header = "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(header.as_bytes()).await;
        return;
    }

    let total_len = match tokio::fs::metadata(&canonical).await {
        Ok(m) => m.len(),
        Err(_) => {
            let header = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = socket.write_all(header.as_bytes()).await;
            return;
        }
    };

    let mut range = None;
    for line in msg.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("range") {
                match crate::core::media_range::parse_range(value, total_len) {
                    Ok(parsed) => range = Some(parsed),
                    Err(()) => {
                        let header = format!("HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: bytes */{total_len}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                        let _ = socket.write_all(header.as_bytes()).await;
                        return;
                    }
                }
            }
        }
    }

    let mut file = match tokio::fs::File::open(&canonical).await {
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
            "HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
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
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
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

#[cfg(target_os = "linux")]
fn ensure_linux_desktop_integration() {
    let Some(base_dirs) = directories::BaseDirs::new() else { return; };
    let data_dir = base_dirs.data_dir();

    let icons_512_dir = data_dir.join("icons/hicolor/512x512/apps");
    let icons_128_dir = data_dir.join("icons/hicolor/128x128/apps");
    let icons_64_dir = data_dir.join("icons/hicolor/64x64/apps");
    let icons_32_dir = data_dir.join("icons/hicolor/32x32/apps");
    let pixmaps_dir = data_dir.join("pixmaps");
    let apps_dir = data_dir.join("applications");

    let _ = std::fs::create_dir_all(&icons_512_dir);
    let _ = std::fs::create_dir_all(&icons_128_dir);
    let _ = std::fs::create_dir_all(&icons_64_dir);
    let _ = std::fs::create_dir_all(&icons_32_dir);
    let _ = std::fs::create_dir_all(&pixmaps_dir);
    let _ = std::fs::create_dir_all(&apps_dir);

    let icon_512_bytes = include_bytes!("../icons/icon.png");
    let icon_128_bytes = include_bytes!("../icons/128x128.png");
    let icon_64_bytes = include_bytes!("../icons/64x64.png");
    let icon_32_bytes = include_bytes!("../icons/32x32.png");

    let _ = std::fs::write(icons_512_dir.join("luxmc.png"), icon_512_bytes);
    let _ = std::fs::write(icons_128_dir.join("luxmc.png"), icon_128_bytes);
    let _ = std::fs::write(icons_64_dir.join("luxmc.png"), icon_64_bytes);
    let _ = std::fs::write(icons_32_dir.join("luxmc.png"), icon_32_bytes);
    let _ = std::fs::write(pixmaps_dir.join("luxmc.png"), icon_512_bytes);

    let exe_path = if let Ok(appimage) = std::env::var("APPIMAGE") {
        appimage
    } else if let Ok(p) = std::env::current_exe() {
        p.to_string_lossy().to_string()
    } else {
        "luxmc".to_string()
    };

    let desktop_content = format!(
        "[Desktop Entry]\n\
        Name=Luxmc\n\
        GenericName=Minecraft Launcher\n\
        Comment=Linux-first Minecraft launcher\n\
        Exec=\"{}\" %u\n\
        Icon=luxmc\n\
        Terminal=false\n\
        Type=Application\n\
        Categories=Game;ActionGame;AdventureGame;\n\
        MimeType=x-scheme-handler/luxmc;\n\
        StartupWMClass=luxmc\n\
        StartupNotify=true\n\
        Keywords=minecraft;launcher;luxmc;modpack;optifine;fabric;forge;neoforge;\n",
        exe_path
    );

    let _ = std::fs::remove_file(apps_dir.join("io.github.luxmc.Luxmc.desktop"));
    let _ = std::fs::remove_file(apps_dir.join("luxmc-handler.desktop"));
    let _ = std::fs::remove_file(apps_dir.join("luxmc-debug-handler.desktop"));

    if std::path::Path::new("/usr/share/applications/luxmc.desktop").exists() {
        let _ = std::fs::remove_file(apps_dir.join("luxmc.desktop"));
        let _ = std::process::Command::new("update-desktop-database").arg(&apps_dir).output();
        let _ = std::process::Command::new("xdg-mime").args(["default", "luxmc.desktop", "x-scheme-handler/luxmc"]).output();
        return;
    }

    let _ = std::fs::write(apps_dir.join("luxmc.desktop"), &desktop_content);

    let _ = std::process::Command::new("update-desktop-database").arg(&apps_dir).output();
    let _ = std::process::Command::new("xdg-mime").args(["default", "luxmc.desktop", "x-scheme-handler/luxmc"]).output();
    let _ = std::process::Command::new("gtk-update-icon-cache").args(["-f", "-t", &data_dir.join("icons/hicolor").to_string_lossy()]).output();
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
            #[cfg(target_os = "linux")]
            {
                extern "C" {
                    fn g_set_prgname(prgname: *const std::ffi::c_char);
                    fn g_set_application_name(application_name: *const std::ffi::c_char);
                }
                unsafe {
                    g_set_prgname(b"luxmc\0".as_ptr() as *const _);
                    g_set_application_name(b"Luxmc\0".as_ptr() as *const _);
                }
                ensure_linux_desktop_integration();
            }

            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                commands::deep_links::enqueue(&handle, event.urls().into_iter().map(|url| url.to_string()).collect());
            });
            if let Some(urls) = app.deep_link().get_current()? {
                commands::deep_links::enqueue(app.handle(), urls.into_iter().map(|url| url.to_string()).collect());
            }
            #[cfg(target_os = "windows")]
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
                    let connections = std::sync::Arc::new(tokio::sync::Semaphore::new(8));
                    while let Ok((mut socket, _)) = listener.accept().await {
                        let Ok(permit) = connections.clone().try_acquire_owned() else { continue; };
                        let overlay_ref = handle_overlay.clone();
                        tokio::spawn(async move {
                            let _permit = permit;
                            use tokio::io::AsyncReadExt;
                            let mut buf = [0u8; 4096];
                            if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(5), socket.read(&mut buf)).await {
                                let msg = String::from_utf8_lossy(&buf[..n]);
                                if msg.starts_with("GET /media") {
                                    let _ = tokio::time::timeout(std::time::Duration::from_secs(120), handle_media_request(socket, &msg)).await;
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
            network::p2p_tunnel::host_world,
            network::p2p_tunnel::join_world,
            network::p2p_tunnel::stop_session,
            network::p2p_tunnel::tunnel_status,
            commands::mesh::mesh_status,
            commands::mesh::mesh_ping,
            commands::deep_links::deep_links_take,
            commands::deep_links::open_portal,
            commands::skins::minecraft_uuid,
            commands::social::social_request,
            commands::news::minecraft_news,
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
            commands::auth::auth_save_appearance,
            commands::auth::auth_resolve_texture,
            commands::auth::auth_read_local_texture,
            commands::wallpaper::wallpaper_prepare_video,
            commands::wallpaper::wallpaper_prepare_poster,
            commands::instance_lab::instance_capsule_create,
            commands::instance_lab::instance_capsules_list,
            commands::instance_lab::instance_capsule_restore,
            commands::instance_lab::instance_isolation_status,
            commands::instance_lab::instance_isolation_start,
            commands::instance_lab::instance_isolation_report,
            commands::instance_lab::instance_isolation_restore,
            commands::instance_lab::instance_benchmarks_list,
            commands::instance_lab::instance_benchmark_import,
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
            commands::storage::storage_full_report,
            commands::storage::storage_clear_logs,
            commands::storage::storage_clear_cache,
            commands::storage::storage_delete_instance,
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
