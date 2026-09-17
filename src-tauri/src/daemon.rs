use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::mods::ModrinthClient;
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
            let ram_mb = args.get("ramMb").and_then(|v| v.as_u64()).unwrap_or(4096);
            let auto = args.get("autoOptimize").and_then(|v| v.as_bool()).unwrap_or(true);
            let flags = crate::commands::optimizer::optimizer_get_flags(ram_mb, auto);
            Ok(serde_json::to_value(flags).map_err(|e| e.to_string())?)
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
        "instances_list" => {
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let list = crate::db::schema::profiles::list(&db).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(list).map_err(|e| e.to_string())?)
        },
        "versions_list" => {
            let versions = crate::core::minecraft::fetch_version_manifest(&state.http).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(versions).map_err(|e| e.to_string())?)
        },
        "loaders_versions" => {
            let loader = args.get("loader").and_then(|v| v.as_str()).unwrap_or("fabric");
            let game_version = args.get("gameVersion").and_then(|v| v.as_str()).unwrap_or("1.20.1");
            let versions = crate::core::loaders::fetch_loader_versions(&state.http, loader, game_version).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(versions).map_err(|e| e.to_string())?)
        },
        "mods_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let game_version = args.get("gameVersion").and_then(|v| v.as_str()).unwrap_or("1.20.1");
            let loader = args.get("loader").and_then(|v| v.as_str());
            let content_type = args.get("contentType").and_then(|v| v.as_str()).unwrap_or("mod");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
            let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let client = ModrinthClient::new(state.http.clone());
            let results = client.search_mods(query, game_version, content_type, loader, None, limit, offset, Some("downloads")).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
        },
        "auth_accounts" => {
            let db = crate::db::shared_db().await.map_err(|e| e.to_string())?;
            let accounts = crate::db::schema::accounts::list(&db).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(accounts).map_err(|e| e.to_string())?)
        },
        "auth_offline_login" => {
            let username = args.get("username").and_then(|v| v.as_str()).unwrap_or("Player");
            let account = crate::commands::auth::auth_offline_login(username.to_string()).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(account).map_err(|e| e.to_string())?)
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
        _ => Err(format!("Command '{cmd}' not yet registered in daemon dispatcher")),
    }
}
