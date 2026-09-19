use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::AppState;

#[derive(Deserialize)]
struct RpcRequest {
    id: u64,
    command: String,
    #[serde(default)]
    args: Value,
}

#[derive(Serialize)]
struct RpcResponse {
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn run_daemon() {
    if let Err(e) = crate::db::shared_db().await {
        eprintln!("[Daemon] Database init failed: {e}");
    }

    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(12));
        loop {
            interval.tick().await;
            crate::core::native_cpp::trim_memory_native();
        }
    });

    let state = Arc::new(AppState::default());
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let stdout = Arc::new(tokio::sync::Mutex::new(tokio::io::stdout()));

    while let Ok(Some(line)) = reader.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = serde_json::json!({
                    "id": 0,
                    "error": format!("Invalid JSON: {e}")
                });
                let mut out = stdout.lock().await;
                let _ = out.write_all(format!("{}\n", err_resp).as_bytes()).await;
                let _ = out.flush().await;
                continue;
            }
        };

        let state_clone = Arc::clone(&state);
        let stdout_clone = Arc::clone(&stdout);

        tokio::spawn(async move {
            let res = dispatch_command(&req.command, req.args, state_clone).await;
            let resp = match res {
                Ok(val) => RpcResponse {
                    id: req.id,
                    result: Some(val),
                    error: None,
                },
                Err(err) => RpcResponse {
                    id: req.id,
                    result: None,
                    error: Some(err),
                },
            };

            if let Ok(line) = serde_json::to_string(&resp) {
                let mut out = stdout_clone.lock().await;
                let _ = out.write_all(format!("{}\n", line).as_bytes()).await;
                let _ = out.flush().await;
            }
        });
    }
}

async fn dispatch_command(
    cmd: &str,
    args: Value,
    state: Arc<AppState>,
) -> Result<Value, String> {
    match cmd {
        "social_request" => {
            let account_id = args.get("accountId").and_then(|v| v.as_str()).ok_or("Missing account ID")?.to_owned();
            let request = serde_json::from_value(args.get("request").cloned().ok_or("Missing request")?).map_err(|e| e.to_string())?;
            crate::commands::social::social_request_core(account_id, request).await.map_err(|e| e.to_string())
        },
        "ping" => Ok(Value::String("pong".into())),
        "app_info" => Ok(serde_json::to_value(crate::commands::system::app_info()).map_err(|e| e.to_string())?),
        "optimizer_trim_memory" => {
            let res = crate::core::native_cpp::trim_memory_native();
            Ok(Value::Bool(res))
        },
        "optimizer_native_cpu_profile" => {
            let profile = crate::core::native_cpp::get_cpu_profile();
            Ok(serde_json::to_value(profile).map_err(|e| e.to_string())?)
        },
        "optimizer_detect_gpu" => {
            let gpu = crate::core::optimizer::detect_gpu();
            Ok(serde_json::to_value(gpu).map_err(|e| e.to_string())?)
        },
        "optimizer_get_flags" => {
            let ram_mb = args.get("ramMb").or_else(|| args.get("ram_mb")).and_then(|v| v.as_u64()).unwrap_or(4096);
            let auto = args.get("autoOptimize").or_else(|| args.get("auto_optimize")).and_then(|v| v.as_bool()).unwrap_or(true);
            let flags = crate::commands::optimizer::optimizer_get_flags(ram_mb, auto);
            Ok(serde_json::to_value(flags).map_err(|e| e.to_string())?)
        },
        "optimizer_get_perf_pack" => {
            let loader = args.get("loader").and_then(|v| v.as_str()).unwrap_or("fabric").to_string();
            let mc_version = args.get("mcVersion").or_else(|| args.get("mc_version")).and_then(|v| v.as_str()).unwrap_or("1.20.1").to_string();
            let pack = crate::commands::optimizer::optimizer_get_perf_pack(loader, mc_version);
            Ok(serde_json::to_value(pack).map_err(|e| e.to_string())?)
        },
        "optimizer_install_perf_pack" => {
            let instance_id = args.get("instanceId").or_else(|| args.get("instance_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::optimizer::optimizer_install_perf_pack_core(&state.http, instance_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "get_system_specs" => {
            let specs = crate::commands::system::get_system_specs();
            Ok(serde_json::to_value(specs).map_err(|e| e.to_string())?)
        },
        "env_check" => {
            let check = crate::commands::env::env_check().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(check).map_err(|e| e.to_string())?)
        },

        "profiles_list" => {
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let list = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles ORDER BY updated_at DESC")
                .fetch_all(db.pool())
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "profiles_get" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::profiles::profiles_get(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "profiles_create" => {
            let input_val = args.get("input").cloned().unwrap_or(args);
            let input: crate::commands::profiles::ProfileCreate = serde_json::from_value(input_val).map_err(|e| e.to_string())?;
            let res = crate::commands::profiles::profiles_create(input).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "profiles_update" => {
            let input_val = args.get("input").cloned().unwrap_or(args);
            let input: crate::commands::profiles::ProfileUpdate = serde_json::from_value(input_val).map_err(|e| e.to_string())?;
            let res = crate::commands::profiles::profiles_update(input).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "profiles_delete" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::profiles::profiles_delete(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::json!({ "success": true }))
        },

        "instances_list" => {
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let list = crate::db::schema::profiles::list(&db).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "instances_duplicate" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instances_duplicate(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instances_open_folder" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instances_open_folder(id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instances_screenshots" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instances_screenshots(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "screenshot_delete" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::screenshot_delete(path).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "screenshots_open_folder" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::screenshots_open_folder(profile_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_health_check" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_health_check(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_repair_modpack" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_repair_modpack_core(None, &state, profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_repair" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instance_tools::instance_repair_core(None, &state.http, profile_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_file_tree" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let sub_path = args.get("subPath").or_else(|| args.get("sub_path")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::instances::instance_file_tree(profile_id, sub_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_export" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let dest_path = args.get("destPath").or_else(|| args.get("dest_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_export(profile_id, dest_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_set_notes" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let notes = args.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
            crate::commands::instances::instance_set_notes(profile_id, notes).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_set_favorite" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let favorite = args.get("favorite").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::commands::instances::instance_set_favorite(profile_id, favorite).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_mod_toggle" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file_name = args.get("fileName").or_else(|| args.get("file_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let res = crate::commands::instances::instance_mod_toggle(profile_id, file_name, enabled).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_mod_delete" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file_name = args.get("fileName").or_else(|| args.get("file_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_mod_delete(profile_id, file_name).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_mod_add" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source_path = args.get("sourcePath").or_else(|| args.get("source_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_mod_add(profile_id, source_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_mods_open_folder" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_mods_open_folder(profile_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_pack_add" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let pack_type = args.get("packType").or_else(|| args.get("pack_type")).and_then(|v| v.as_str()).unwrap_or("resourcepacks").to_string();
            let source_path = args.get("sourcePath").or_else(|| args.get("source_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_pack_add(profile_id, pack_type, source_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_pack_delete" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let pack_type = args.get("packType").or_else(|| args.get("pack_type")).and_then(|v| v.as_str()).unwrap_or("resourcepacks").to_string();
            let file_name = args.get("fileName").or_else(|| args.get("file_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_pack_delete(profile_id, pack_type, file_name).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_pack_open_folder" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let pack_type = args.get("packType").or_else(|| args.get("pack_type")).and_then(|v| v.as_str()).unwrap_or("resourcepacks").to_string();
            crate::commands::instances::instance_pack_open_folder(profile_id, pack_type).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_import_modpack" => {
            let file_path = args.get("filePath").or_else(|| args.get("file_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let profile_name = args.get("profileName").or_else(|| args.get("profile_name")).and_then(|v| v.as_str()).unwrap_or("Modpack").to_string();
            let mc_version = args.get("mcVersion").or_else(|| args.get("mc_version")).and_then(|v| v.as_str()).unwrap_or("1.20.1").to_string();
            let loader = args.get("loader").and_then(|v| v.as_str()).unwrap_or("forge").to_string();
            let icon = args.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ram_mb = args.get("ramMb").or_else(|| args.get("ram_mb")).and_then(|v| v.as_i64());
            let res = crate::commands::instances::instance_import_modpack_core(None, &state, file_path, profile_name, mc_version, loader, icon, ram_mb).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_import_mrpack" => {
            let file_path = args.get("filePath").or_else(|| args.get("file_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let profile_name = args.get("profileName").or_else(|| args.get("profile_name")).and_then(|v| v.as_str()).unwrap_or("Modpack").to_string();
            let icon = args.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string());
            let ram_mb = args.get("ramMb").or_else(|| args.get("ram_mb")).and_then(|v| v.as_i64());
            let res = crate::commands::instances::instance_import_mrpack_core(None, &state, file_path, profile_name, icon, ram_mb).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_cancel_import" => {
            crate::commands::instances::instance_cancel_import_core(&state).map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_install_quick_pack" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let pack_type = args.get("packType").or_else(|| args.get("pack_type")).and_then(|v| v.as_str()).unwrap_or("resourcepacks").to_string();
            let project_id = args.get("projectId").or_else(|| args.get("project_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_install_quick_pack_core(&state.http, profile_id, pack_type, project_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_options_get" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::options_editor::instance_options_get(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_options_set" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let opt_val = args.get("options").cloned().unwrap_or(Value::Null);
            let options: crate::commands::options_editor::InstanceMinecraftOptions = serde_json::from_value(opt_val).map_err(|e| e.to_string())?;
            crate::commands::options_editor::instance_options_set(profile_id, options).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_config_read" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let rel_path = args.get("relativePath").or_else(|| args.get("relative_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::options_editor::instance_config_read(profile_id, rel_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_config_write" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let rel_path = args.get("relativePath").or_else(|| args.get("relative_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::options_editor::instance_config_write(profile_id, rel_path, content).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_worlds_list" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_worlds_list(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_world_delete" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_world_delete(profile_id, folder_name).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_world_snapshot_create" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let label = args.get("label").and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::instances::instance_world_snapshot_create(profile_id, folder_name, label).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_world_snapshots_list" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instances::instance_world_snapshots_list(profile_id, folder_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_world_snapshot_restore" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let filename = args.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_world_snapshot_restore(profile_id, folder_name, filename).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_world_snapshot_delete" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let filename = args.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instances::instance_world_snapshot_delete(profile_id, folder_name, filename).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "instance_world_inspect_region" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let folder_name = args.get("folderName").or_else(|| args.get("folder_name")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let region_file = args.get("regionFile").or_else(|| args.get("region_file")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::instances::instance_world_inspect_region(profile_id, folder_name, region_file).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_disk_usage" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_disk_usage(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_export_zip" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let output_path = args.get("outputPath").or_else(|| args.get("output_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_export_zip(profile_id, output_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_backup_saves" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let output_path = args.get("outputPath").or_else(|| args.get("output_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_backup_saves(profile_id, output_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_restore_saves" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let zip_path = args.get("zipPath").or_else(|| args.get("zip_path")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_restore_saves(profile_id, zip_path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_export_share_code" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_export_share_code(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_import_share_code" => {
            let share_code = args.get("shareCode").or_else(|| args.get("share_code")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::instance_import_share_code(share_code).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_export_modpack" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let export_format = args.get("exportFormat").or_else(|| args.get("export_format")).or_else(|| args.get("packFormat")).and_then(|v| v.as_str()).unwrap_or("mrpack").to_string();
            let custom_name = args.get("customName").or_else(|| args.get("custom_name")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::modpack_export::instance_export_modpack(profile_id, export_format, custom_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "instance_shield_scan" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::shield::instance_shield_scan(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "versions_list" => {
            let res = crate::commands::versions::versions_list_core(&state.http).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "versions_detail" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::versions::versions_detail_core(&state.http, &id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "versions_download" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::versions::versions_download_core(&state.http, &id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "versions_check_installed" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::versions::versions_check_installed_core(&id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "version_repair" => {
            let version_id = args.get("versionId").or_else(|| args.get("version_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instance_tools::version_repair_core(None, &state.http, version_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "loaders_versions" => {
            let loader = args.get("loader").and_then(|v| v.as_str()).unwrap_or("fabric");
            let game_version = args.get("gameVersion").or_else(|| args.get("mcVersion")).and_then(|v| v.as_str()).unwrap_or("1.20.1");
            let versions = crate::core::loaders::fetch_loader_versions(&state.http, loader, game_version).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(versions).map_err(|e| e.to_string())?)
        },

        "mods_search" | "mods_search_typed" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let game_version = args.get("mcVersion").or_else(|| args.get("gameVersion")).and_then(|v| v.as_str()).unwrap_or("1.20.1").to_string();
            let loader = args.get("loader").and_then(|v| v.as_str()).map(|s| s.to_string());
            let content_type = args.get("contentType").and_then(|v| v.as_str()).map(|s| s.to_string());
            let category = args.get("category").and_then(|v| v.as_str()).map(|s| s.to_string());
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as u32);
            let offset = args.get("offset").and_then(|v| v.as_u64()).map(|n| n as u32);
            let sort_by = args.get("sortBy").or_else(|| args.get("sort_by")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let source = args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::mods::mods_search_core(&state, query, game_version, limit, offset, content_type, sort_by, loader, category, source).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_versions" => {
            let project_id = args.get("projectId").or_else(|| args.get("project_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mc_version = args.get("mcVersion").or_else(|| args.get("mc_version")).and_then(|v| v.as_str()).unwrap_or("1.20.1").to_string();
            let source = args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::mods::mods_versions_core(&state, project_id, mc_version, source).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_project_details" => {
            let project_id = args.get("projectId").or_else(|| args.get("project_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let source = args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::mods::mods_project_details_core(&state, project_id, source).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_list" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let res = crate::db::schema::mods::list_by_profile(&db, &profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_install" => {
            let req_val = args.get("request").cloned().unwrap_or(args);
            let request: crate::commands::mods::ModInstallRequest = serde_json::from_value(req_val).map_err(|e| e.to_string())?;
            let res = crate::commands::mods::mods_install_core(&state, request).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_install_with_deps" => {
            let req_val = args.get("request").cloned().unwrap_or(args);
            let request: crate::commands::mods::ModInstallRequest = serde_json::from_value(req_val).map_err(|e| e.to_string())?;
            let res = crate::commands::mods::mods_install_with_deps_core(&state, request).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_remove" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let project_id = args.get("projectId").or_else(|| args.get("project_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::mods::mods_remove(profile_id, project_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "mods_check_updates" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::mods::mods_check_updates_core(&state, profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_update" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let project_id = args.get("projectId").or_else(|| args.get("project_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let version_id = args.get("versionId").or_else(|| args.get("version_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::mods::mods_update_core(&state, profile_id, project_id, version_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_download_to_temp" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file_name = args.get("fileName").or_else(|| args.get("file_name")).and_then(|v| v.as_str()).unwrap_or("temp.jar").to_string();
            let res = crate::commands::mods::mods_download_to_temp_core(&state, url, file_name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_resolve_names" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::mods::mods_resolve_names_core(&state, profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "mods_resolve_icons" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::mods::mods_resolve_icons_core(&state, profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "curseforge_status" => {
            let res = crate::commands::mods::curseforge_status();
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "curseforge_get_key" => {
            let res = crate::commands::mods::curseforge_get_key();
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "curseforge_set_key" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::mods::curseforge_set_key(key).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "curseforge_remove_key" => {
            crate::commands::mods::curseforge_remove_key().await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "curseforge_validate_key" => {
            let res = crate::commands::mods::curseforge_validate_key_core(&state.http).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "launch_logs_list" => {
            let limit = args.get("limit").and_then(|v| v.as_i64());
            let res = crate::commands::launch_logs::launch_logs_list(limit).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "launch_logs_get" => {
            let id = args.get("id").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
            let res = crate::commands::launch_logs::launch_logs_get(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "launch_logs_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let min_level = args.get("minLevel").or_else(|| args.get("min_level")).and_then(|v| v.as_str()).unwrap_or("INFO").to_string();
            let limit = args.get("limit").and_then(|v| v.as_i64());
            let res = crate::commands::launch_logs::launch_logs_search(query, min_level, limit).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "launch_logs_clear" => {
            crate::commands::launch_logs::launch_logs_clear().await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "launch_log_open" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let version_id = args.get("versionId").or_else(|| args.get("mcVersion")).or_else(|| args.get("version_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::launch_log_session::launch_log_open(profile_id, version_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "launch_log_append" => {
            let log_id = args.get("logId").or_else(|| args.get("log_id")).and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
            let stream = args.get("stream").and_then(|v| v.as_str()).unwrap_or("stdout").to_string();
            let level = args.get("level").and_then(|v| v.as_str()).unwrap_or("INFO").to_string();
            let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::launch_log_session::launch_log_append(log_id, stream, level, message).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "launch_log_close" => {
            let log_id = args.get("logId").or_else(|| args.get("log_id")).and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
            let exit_code = args.get("exitCode").or_else(|| args.get("exit_code")).and_then(|v| v.as_i64());
            let summary = args.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
            let error_classification = args.get("errorClassification").or_else(|| args.get("error_classification")).and_then(|v| v.as_str()).map(|s| s.to_string());
            crate::commands::launch_log_session::launch_log_close(log_id, exit_code, summary, error_classification).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "crash_summary" => {
            let log = args.get("logContent").or_else(|| args.get("log_content")).and_then(|v| v.as_str()).unwrap_or("");
            let lines: Vec<String> = log.lines().map(|s| s.to_string()).collect();
            let res = crate::commands::instance_tools::crash_summary(lines).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "share_log_mclogs" => {
            let log = args.get("logContent").or_else(|| args.get("log_content")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::share_log_mclogs_core(&state.http, log).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "storage_breakdown" => {
            let res = crate::commands::storage::storage_breakdown().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "storage_total" => {
            let paths: Vec<String> = args.get("paths").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
            let res = crate::commands::instance_icons::storage_total(paths).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "directory_exists" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_icons::directory_exists(path).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(res))
        },
        "ensure_directory" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instance_icons::ensure_directory(path).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "read_text_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_icons::read_text_file(path).await.map_err(|e| e.to_string())?;
            Ok(Value::String(res))
        },
        "write_text_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let contents = args.get("contents").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instance_icons::write_text_file(path, contents).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "delete_file_or_dir" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::instance_icons::delete_file_or_dir(path).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },

        "auth_accounts" => {
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let accounts = crate::db::schema::accounts::list(&db).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(accounts).map_err(|e| e.to_string())?)
        },
        "lux_account_login" => {
            let username = args.get("username").and_then(|v| v.as_str()).ok_or("Missing nickname")?.to_owned();
            let password = args.get("password").and_then(|v| v.as_str()).ok_or("Missing password")?.to_owned();
            let account = crate::commands::lux_account::lux_account_login(username, password).await.map_err(|e| e.to_string())?;
            serde_json::to_value(account).map_err(|e| e.to_string())
        },
        "lux_account_sync" => {
            let id = args.get("accountId").and_then(|v| v.as_str()).ok_or("Missing account")?.to_owned();
            let prefs = args.get("preferences").filter(|v| !v.is_null()).cloned().map(serde_json::from_value).transpose().map_err(|e| e.to_string())?;
            let revision = args.get("revision").and_then(|v| v.as_i64());
            let account = crate::commands::lux_account::lux_account_sync(id, prefs, revision).await.map_err(|e| e.to_string())?;
            serde_json::to_value(account).map_err(|e| e.to_string())
        },
        "lux_account_logout" => {
            let id = args.get("accountId").and_then(|v| v.as_str()).ok_or("Missing account")?.to_owned();
            crate::commands::lux_account::lux_account_logout(id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::json!(null))
        },
        "auth_offline_login" => {
            let username = args.get("username").and_then(|v| v.as_str()).unwrap_or("Player");
            let account = crate::commands::auth::auth_offline_login(username.to_string()).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
        },
        "auth_login" | "auth_begin" => {
            let res = crate::commands::auth::auth_begin_core(&state).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "auth_complete" => {
            let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let state_token = args.get("stateToken").or_else(|| args.get("state")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let verifier = args.get("verifier").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::auth::auth_complete_core(&state, code, state_token, verifier).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "auth_refresh" => {
            let refresh_token = args.get("refreshToken").or_else(|| args.get("refresh_token")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::auth::auth_refresh_core(&state, refresh_token).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "auth_dev_login" => {
            let res = crate::commands::auth::auth_dev_login_core(&state).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "auth_switch_account" => {
            let uuid = args.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::auth::auth_switch_account(uuid).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "auth_remove" => {
            let uuid = args.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::auth::auth_remove(uuid).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "auth_get_client_id" => {
            let res = crate::commands::auth::auth_get_client_id().await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "auth_set_client_id" => {
            let client_id = args.get("clientId").or_else(|| args.get("client_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::auth::auth_set_client_id(client_id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "auth_get_tenant_id" => {
            let res = crate::commands::auth::auth_get_tenant_id().await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "minecraft_uuid" => {
            let username = args.get("username").and_then(|value| value.as_str()).unwrap_or_default().to_owned();
            let result = crate::commands::skins::minecraft_uuid(username).await.map_err(|error| error.to_string())?;
            serde_json::to_value(result).map_err(|error| error.to_string())
        },
        "auth_change_skin" => {
            let uuid = args.get("uuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let variant = args.get("variant").and_then(|v| v.as_str()).unwrap_or("classic").to_string();
            let skin_url = args.get("skinUrl").or_else(|| args.get("skin_url")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::auth::auth_change_skin(uuid, variant, skin_url).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },

        "settings_get" => {
            let settings = crate::commands::settings::settings_get().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(settings).map_err(|e| e.to_string())?)
        },
        "settings_set" => {
            let val = args.get("value").cloned().unwrap_or(Value::Null);
            crate::commands::settings::settings_set(val).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },

        "launch_game" => {
            let req_val = args.get("request").cloned().unwrap_or(args);
            let request: crate::commands::launch::LaunchRequest = serde_json::from_value(req_val).map_err(|e| e.to_string())?;
            let resp = crate::commands::launch::launch_game_daemon(&state, request).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp).map_err(|e| e.to_string())?)
        },
        "app_init" => {
            let init_state = crate::commands::system::app_init_core(&state).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(init_state).map_err(|e| e.to_string())?)
        },
        "gaming_stats_get" => {
            let stats = crate::commands::skins::gaming_stats_get().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
        },
        "gaming_stats_save" => {
            let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::skins::gaming_stats_save(data).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "java_scan" => {
            let scan = crate::commands::java::java_scan_core(&state).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(scan).map_err(|e| e.to_string())?)
        },
        "discord_set_activity" => {
            let details = args.get("details").and_then(|v| v.as_str()).map(|s| s.to_string());
            let act_state = args.get("state").and_then(|v| v.as_str()).map(|s| s.to_string());
            let large_text = args.get("largeText").and_then(|v| v.as_str()).map(|s| s.to_string());
            let large_image = args.get("largeImage").and_then(|v| v.as_str()).map(|s| s.to_string());
            let small_text = args.get("smallText").and_then(|v| v.as_str()).map(|s| s.to_string());
            let small_image = args.get("smallImage").and_then(|v| v.as_str()).map(|s| s.to_string());
            let start_time = args.get("startTime").and_then(|v| v.as_i64());
            let in_game = args.get("inGame").and_then(|v| v.as_bool());
            let client_id = args.get("clientId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let _ = crate::commands::discord::discord_set_activity(
                details,
                act_state,
                large_text,
                large_image,
                small_text,
                small_image,
                start_time,
                in_game,
                client_id,
            ).await;
            Ok(Value::Bool(true))
        },
        "discord_clear_activity" => {
            let _ = crate::commands::discord::discord_clear_activity().await;
            Ok(Value::Bool(true))
        },

        "server_ping" => {
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1").to_string();
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(25565) as u16;
            let status = crate::core::server::ping_cached(&state.http, &host, port).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
        },
        "server_list" => {
            let list = crate::commands::servers::server_list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "server_favorites_list" => {
            let list = crate::commands::servers::server_favorites_list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "server_add" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("Server").to_string();
            let host = args.get("host").or_else(|| args.get("address")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(25565) as u16;
            let res = crate::commands::servers::server_add(host, port, name).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "server_remove" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::servers::server_remove(id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "server_favorite" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let favorite = args.get("favorite").and_then(|v| v.as_bool()).unwrap_or(false);
            crate::commands::servers::server_favorite(id, favorite).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },

        "skins_list" => {
            let list = crate::commands::skins::skins_list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "skins_save" => {
            let req_val = args.get("request").cloned().unwrap_or(args);
            let request: crate::commands::skins::SaveSkinRequest = serde_json::from_value(req_val).map_err(|e| e.to_string())?;
            let res = crate::commands::skins::skins_save_core(&state.http, request).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "skins_delete" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::skins::skins_delete(id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },
        "skins_import_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = args.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let model = args.get("modelType").or_else(|| args.get("model")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let res = crate::commands::skins::skins_import_file(path, name, model).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "capes_list" => {
            let list = crate::commands::skins::capes_list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "capes_save" => {
            let req_val = args.get("request").cloned().unwrap_or(args);
            let request: crate::commands::skins::SaveCapeRequest = serde_json::from_value(req_val).map_err(|e| e.to_string())?;
            let res = crate::commands::skins::capes_save_core(&state.http, request).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "capes_delete" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            crate::commands::skins::capes_delete(id).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
        },

        "keybinds_list" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::keybinds::keybinds_list(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "keybinds_update" => {
            let profile_id = args.get("profileId").or_else(|| args.get("profile_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let updates: std::collections::HashMap<String, String> = args.get("updates").or_else(|| args.get("keybinds")).and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
            let res = crate::commands::keybinds::keybinds_update(profile_id, updates).await.map_err(|e| e.to_string())?;
            Ok(Value::Bool(res))
        },
        "changelog_get" => {
            let res = crate::commands::changelog::changelog_get().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "p2p_get_local_info" => {
            let res = crate::commands::p2p::p2p_get_local_info().map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "p2p_get_host_link" => {
            let port = args.get("port").and_then(|v| v.as_u64()).map(|n| n as u16);
            let res = crate::commands::p2p::p2p_get_host_link(port).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "p2p_scan_lan_worlds" => {
            let res = crate::commands::p2p::p2p_scan_lan_worlds().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "p2p_start_listener" => {
            let res = crate::commands::p2p::p2p_start_listener_core(None).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "p2p_send_message" => {
            let target = args.get("targetAddress").or_else(|| args.get("target")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let sender = args.get("sender").and_then(|v| v.as_str()).unwrap_or("Player").to_string();
            let text = args.get("text").or_else(|| args.get("message")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::p2p::p2p_send_message(target, sender, text).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "upnp_open_port" => {
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(25565) as u16;
            let lease = args.get("leaseDurationSecs").or_else(|| args.get("lease_duration_secs")).and_then(|v| v.as_u64()).map(|n| n as u32);
            let res = crate::commands::instances::upnp_open_port(port, lease).await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "upnp_close_port" => {
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(25565) as u16;
            let res = crate::commands::instances::upnp_close_port(port).await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "jvm_args_validate" => {
            let args_str = args.get("jvmArgs").or_else(|| args.get("jvm_args")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let res = crate::commands::instance_tools::jvm_args_validate(args_str).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },
        "java_runtime_status" => {
            let major = args.get("major").and_then(|v| v.as_u64()).unwrap_or(21) as u32;
            let res = crate::commands::instance_tools::java_runtime_status_core(state.http.clone(), major).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        },

        "crash_doctor_diagnose" => {
            let profile_id = args.get("profileId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let crash_log = args.get("crashLog").or_else(|| args.get("logContent")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let diag = crate::commands::doctor::crash_doctor_diagnose_core(profile_id, crash_log).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(diag).map_err(|e| e.to_string())?)
        },
        "doctor_check_instance_conflicts" => {
            let profile_id = args.get("profileId").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let check = crate::commands::doctor::doctor_check_instance_conflicts(profile_id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(check).map_err(|e| e.to_string())?)
        },

        "plugin:store|load" => Ok(serde_json::json!(1)),
        "plugin:store|get" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("app");
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
                .bind(key)
                .fetch_optional(db.pool())
                .await
                .map_err(|e| e.to_string())?;
            match row {
                Some(r) => {
                    use sqlx::Row;
                    let raw: String = r.try_get("value").unwrap_or_default();
                    let val = serde_json::from_str(&raw).unwrap_or(Value::Null);
                    Ok(serde_json::json!([val, true]))
                },
                None => Ok(serde_json::json!([Value::Null, false])),
            }
        },
        "plugin:store|set" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("app");
            let val = args.get("value").cloned().unwrap_or(Value::Null);
            let raw = serde_json::to_string(&val).unwrap_or_default();
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let _ = sqlx::query("INSERT OR REPLACE INTO app_settings (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(raw)
                .execute(db.pool())
                .await;
            Ok(Value::Bool(true))
        },
        "plugin:store|save" => Ok(Value::Bool(true)),
        _ => Err(format!("Command '{cmd}' not yet registered in daemon dispatcher")),
    }
}
