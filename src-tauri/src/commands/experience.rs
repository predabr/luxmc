use crate::error::{AppError, AppResult};
use serde_json::{json, Value};
use sha2::Digest;
use base64::Engine;

pub async fn owner_tools_core(account_id: String, profile_id: String, operation: String) -> AppResult<Value> {
    let response = crate::commands::social::social_request_core(account_id, crate::commands::social::SocialRequest::ProfileGet { target_id: None }).await?;
    if response.pointer("/profile/role").and_then(Value::as_str) != Some("owner") {
        return Err(AppError::InvalidState("Ferramenta exclusiva da conta de dono".into()));
    }
    if operation == "diagnose" { return support_report(profile_id).await; }
    if operation != "repair" { return Err(AppError::InvalidInput("Operação desconhecida".into())); }
    if crate::core::launcher::get_active_game_pid() != 0 { return Err(AppError::InvalidState("Feche o Minecraft antes da manutenção".into())); }
    let db = crate::db::shared_db().await?;
    let profile = sqlx::query_as::<_, crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?").bind(&profile_id).fetch_optional(db.pool()).await?.ok_or_else(|| AppError::NotFound("Instância não encontrada".into()))?;
    let repaired = crate::commands::instances::repair_instance_duplicates(&profile).await?;
    sqlx::query("UPDATE profiles SET force_full_verification = 1 WHERE id = ?").bind(&profile_id).execute(db.pool()).await?;
    Ok(json!({"repaired":repaired,"verificationScheduled":true,"backup":".luxmc/removed-duplicates"}))
}

#[tauri::command]
pub async fn owner_tools(account_id: String, profile_id: String, operation: String) -> AppResult<Value> {
    owner_tools_core(account_id, profile_id, operation).await
}

pub async fn record_performance(profile: &str, fields: Value) -> AppResult<()> {
    let db = crate::db::shared_db().await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS performance_history (id INTEGER PRIMARY KEY AUTOINCREMENT, profile_id TEXT NOT NULL, recorded_at TEXT NOT NULL, metrics TEXT NOT NULL)").execute(db.pool()).await?;
    sqlx::query("INSERT INTO performance_history (profile_id,recorded_at,metrics) VALUES (?, ?, ?)").bind(profile).bind(chrono::Utc::now().to_rfc3339()).bind(fields.to_string()).execute(db.pool()).await?;
    sqlx::query("DELETE FROM performance_history WHERE id NOT IN (SELECT id FROM performance_history ORDER BY id DESC LIMIT 500)").execute(db.pool()).await?;
    Ok(())
}

#[tauri::command]
pub async fn performance_history() -> AppResult<Value> {
    let db = crate::db::shared_db().await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS performance_history (id INTEGER PRIMARY KEY AUTOINCREMENT, profile_id TEXT NOT NULL, recorded_at TEXT NOT NULL, metrics TEXT NOT NULL)").execute(db.pool()).await?;
    let rows: Vec<(String,String,String)> = sqlx::query_as("SELECT profile_id,recorded_at,metrics FROM performance_history ORDER BY id DESC LIMIT 500").fetch_all(db.pool()).await?;
    Ok(Value::Array(rows.into_iter().map(|(profile_id,timestamp,metrics)| json!({"profileId":profile_id,"timestamp":timestamp,"metrics":serde_json::from_str::<Value>(&metrics).unwrap_or(Value::Null)})).collect()))
}

#[tauri::command]
pub async fn verify_pack_download(path: String, size: u64, sha1: String) -> AppResult<()> {
    let base = directories::ProjectDirs::from("io","github","Luxmc").ok_or_else(|| AppError::InvalidState("Cache indisponível".into()))?;
    let root = tokio::fs::canonicalize(base.cache_dir().join("modpacks")).await?;
    let target = tokio::fs::canonicalize(path).await?;
    if !target.starts_with(&root) { return Err(AppError::InvalidInput("Arquivo fora do cache de downloads".into())); }
    if !sha1.is_empty() && (sha1.len() != 40 || !sha1.chars().all(|c| c.is_ascii_hexdigit())) { return Err(AppError::InvalidInput("Hash do provedor inválido".into())); }
    if !crate::core::mods::pack_download::verify_existing(&target, (size > 0).then_some(size), (!sha1.is_empty()).then_some(sha1.as_str()), None, true).await {
        tokio::fs::remove_file(&target).await?;
        return Err(AppError::InvalidInput("Falha na integridade do download: tamanho, hash ou estrutura divergente. O arquivo foi descartado; use Retomar para baixar uma cópia correta.".into()));
    }
    Ok(())
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Compatibility {
    pub mc_version: String,
    pub loader: String,
    pub mod_fingerprint: String,
    pub mod_count: usize,
}

#[tauri::command]
pub async fn instance_compatibility(profile_id: String) -> AppResult<Compatibility> {
    let db = crate::db::shared_db().await?;
    let row = sqlx::query_as::<_,crate::db::models::ProfileRow>("SELECT * FROM profiles WHERE id = ?").bind(profile_id).fetch_one(db.pool()).await?;
    tokio::task::spawn_blocking(move || -> AppResult<Compatibility> {
        use std::io::Read;
        let root = std::path::Path::new(&row.game_dir).join("mods");
        let mut hashes = Vec::new();
        if root.is_dir() {
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                if !entry.file_type()?.is_file() || entry.path().extension().is_none_or(|e| e != "jar") { continue; }
                let mut file = std::fs::File::open(entry.path())?;
                if file.metadata()?.len() > 1024 * 1024 * 1024 { return Err(AppError::InvalidInput("Mod acima de 1 GiB".into())); }
                let mut hash = sha2::Sha256::new();
                let mut buffer = [0u8;65536];
                loop { let count=file.read(&mut buffer)?; if count==0 {break;} hash.update(&buffer[..count]); }
                hashes.push(format!("{:x}",hash.finalize()));
            }
        }
        hashes.sort();
        Ok(Compatibility {mc_version:row.mc_version,loader:row.loader,mod_count:hashes.len(),mod_fingerprint:format!("{:x}",sha2::Sha256::digest(hashes.join("\n").as_bytes()))})
    }).await.map_err(|e| AppError::Internal(e.to_string()))?
}

pub async fn active_compatibility() -> Option<Compatibility> {
    let directory = crate::core::launcher::get_active_game_dir()?;
    let db = crate::db::shared_db().await.ok()?;
    let id: Option<String> = sqlx::query_scalar("SELECT id FROM profiles WHERE game_dir = ? LIMIT 1").bind(directory.to_string_lossy().as_ref()).fetch_optional(db.pool()).await.ok()?;
    instance_compatibility(id?).await.ok()
}

pub fn redact(value: &str) -> String {
    let mut result = value.to_owned();
    for (pattern,replacement) in [
        (r#"(?i)(bearer\s+)[^\s"']+"#, "$1[REMOVIDO]"),
        (r"(?i)((?:access[_-]?token|refresh[_-]?token|password|passwd|authorization|client_secret|api[_-]?key)\s*[=:]\s*)[^\s,;]+", "$1[REMOVIDO]"),
        (r"(?i)(--accessToken\s+)\S+", "$1[REMOVIDO]"),
        (r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}", "[EMAIL]"),
        (r"(?i)(?:[A-Z]:[\\/]Users[\\/]|/home/|/Users/)[^\\/\s]+", "[USUARIO]"),
        (r"\b(?:\d{1,3}\.){3}\d{1,3}\b", "[IP]"),
        (r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b", "[UUID]"),
    ] { if let Ok(regex) = regex::Regex::new(pattern) { result = regex.replace_all(&result,replacement).into_owned(); } }
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_|std::env::var("HOME")) { result=result.replace(&home,"[PASTA_PESSOAL]"); }
    if let Ok(regex)=regex::Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b") {result=regex.replace_all(&result,"[TOKEN]").into_owned();}
    result
}

#[tauri::command]
pub async fn support_report(profile_id: String) -> AppResult<Value> {
    let readiness = crate::commands::doctor::doctor_instance_readiness(profile_id.clone()).await?;
    let diagnosis = crate::commands::doctor::crash_doctor_diagnose_core(profile_id, None).await?;
    let report = json!({"launcherVersion":env!("CARGO_PKG_VERSION"),"platform":std::env::consts::OS,"readiness":readiness,"diagnosis":diagnosis});
    Ok(scrub_report(report))
}

#[tauri::command]
pub async fn support_report_export(path: String, report: Value) -> AppResult<()> {
    if !report.is_object() || report.get("readiness").is_none() || report.get("diagnosis").is_none() { return Err(AppError::InvalidInput("Diagnóstico inválido".into())); }
    let serialized=serde_json::to_vec_pretty(&scrub_report(report))?;
    if serialized.len()>1024*1024 { return Err(AppError::InvalidInput("Diagnóstico acima de 1 MB".into())); }
    tokio::fs::write(path,serialized).await?;
    Ok(())
}

fn appearance(value: &Value) -> AppResult<Value> {
    let object = value.as_object().ok_or_else(|| AppError::InvalidInput("Tema inválido".into()))?;
    let mut result = serde_json::Map::new();
    for key in ["theme","accentTheme","density","animations","blur","interfaceOpacity","wallpaperDim","customBackground","wallpaperFps","wallpaperWidth","pauseWallpaperOnBlur","animatedWallpaperBlur"] {
        if let Some(value) = object.get(key) { result.insert(key.into(), value.clone()); }
    }
    if result.get("theme").is_some_and(|v| !matches!(v.as_str(),Some("default-dark"|"default-light"))) || result.get("accentTheme").is_some_and(|v| !matches!(v.as_str(),Some("gold"|"cyan"|"emerald"|"rose"|"violet"|"orange"|"blue"))) || result.get("density").is_some_and(|v| !matches!(v.as_str(),Some("compact"|"comfortable"|"spacious"))) { return Err(AppError::InvalidInput("Preferências de tema não reconhecidas".into())); }
    for key in ["animations","blur","pauseWallpaperOnBlur","animatedWallpaperBlur"] { if result.get(key).is_some_and(|v| !v.is_boolean()) { return Err(AppError::InvalidInput("Preferência visual inválida".into())); } }
    for key in ["interfaceOpacity","wallpaperDim"] { if result.get(key).is_some_and(|v| v.as_f64().is_none_or(|v| !(0.0..=100.0).contains(&v))) { return Err(AppError::InvalidInput("Valor visual fora do intervalo".into())); } }
    if result.get("wallpaperFps").is_some_and(|v| ![15,24,30,60].contains(&v.as_u64().unwrap_or(0))) || result.get("wallpaperWidth").is_some_and(|v| ![960,1280,1920].contains(&v.as_u64().unwrap_or(0))) || result.get("customBackground").is_some_and(|v| v.as_str().is_none_or(|v| v.len()>64)) { return Err(AppError::InvalidInput("Preferência de wallpaper inválida".into())); }
    Ok(Value::Object(result))
}

#[tauri::command]
pub async fn theme_export(path: String) -> AppResult<()> {
    let settings = crate::commands::settings::settings_get().await?;
    let mut package = json!({"format":"luxmc-theme","schema":1,"appearance":appearance(&settings)?});
    if let Some(source) = settings.get("customWallpaperUrl").and_then(Value::as_str).filter(|value| !value.is_empty() && settings["customBackground"]=="custom") {
        let mut resolved = crate::core::launcher::resolve_local_or_asset_path(source).map(|path| path.to_string_lossy().into_owned());
        if resolved.is_none() {
            if let Ok(url) = url::Url::parse(source) {
                if url.host_str()==Some("asset.localhost") {resolved = urlencoding::decode(url.path().trim_start_matches('/')).ok().map(|s|s.into_owned());}
                if url.host_str()==Some("127.0.0.1") && url.path()=="/media" {resolved=url.query_pairs().find(|(key,_)| key=="path").map(|(_,value)|value.into_owned());}
            }
        }
        let source = resolved.as_deref().unwrap_or(source).strip_prefix("file://").unwrap_or(resolved.as_deref().unwrap_or(source));
        if !source.contains("://") {
            let meta = tokio::fs::metadata(source).await?;
            if meta.len() > 64 * 1024 * 1024 { return Err(AppError::InvalidInput("Para compartilhar, escolha um wallpaper de até 64 MB".into())); }
            let bytes = tokio::fs::read(source).await?;
            let extension = std::path::Path::new(source).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if !matches!(extension.as_str(), "mp4"|"webm"|"gif"|"png"|"jpg"|"jpeg"|"webp") { return Err(AppError::InvalidInput("Formato de wallpaper inválido".into())); }
            package["wallpaper"] = json!({"extension":extension,"sha256":format!("{:x}",sha2::Sha256::digest(&bytes)),"data":base64::engine::general_purpose::STANDARD.encode(bytes)});
        } else { let url=url::Url::parse(source).map_err(|e| AppError::InvalidInput(e.to_string()))?; if url.scheme()!="https" || !url.username().is_empty() || url.password().is_some() {return Err(AppError::InvalidInput("Escolha um wallpaper local ou HTTPS sem credenciais para compartilhar".into()));} package["wallpaperUrl"] = Value::String(source.into()); }
    }
    tokio::fs::write(path,serde_json::to_vec(&package)?).await?;
    Ok(())
}

#[tauri::command]
pub async fn theme_import(path: String) -> AppResult<Value> {
    if tokio::fs::metadata(&path).await?.len() > 90 * 1024 * 1024 { return Err(AppError::InvalidInput("Tema acima de 90 MB".into())); }
    let package: Value = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    if package["format"] != "luxmc-theme" || package["schema"] != 1 { return Err(AppError::InvalidInput("Arquivo não é um tema Luxmc compatível".into())); }
    let mut value = appearance(&package["appearance"])?;
    if let Some(wallpaper) = package.get("wallpaper") {
        let extension = wallpaper["extension"].as_str().unwrap_or("");
        if !matches!(extension,"mp4"|"webm"|"gif"|"png"|"jpg"|"jpeg"|"webp") { return Err(AppError::InvalidInput("Formato de wallpaper inválido".into())); }
        let bytes = base64::engine::general_purpose::STANDARD.decode(wallpaper["data"].as_str().unwrap_or("")).map_err(|e| AppError::InvalidInput(e.to_string()))?;
        let hash = format!("{:x}",sha2::Sha256::digest(&bytes));
        if bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 || wallpaper["sha256"] != hash { return Err(AppError::InvalidInput("Wallpaper corrompido: a conferência SHA-256 falhou".into())); }
        let base = directories::ProjectDirs::from("io","github","Luxmc").ok_or_else(|| AppError::InvalidState("Pasta de temas indisponível".into()))?;
        let directory = base.data_local_dir().join("wallpapers");
        tokio::fs::create_dir_all(&directory).await?;
        let target = directory.join(format!("{hash}.{extension}"));
        crate::core::mods::pack_download::atomic_write(&target,&bytes).await?;
        value["customWallpaperUrl"] = json!(target.to_string_lossy());
        value["customWallpaperType"] = json!(if matches!(extension,"mp4"|"webm") { "video" } else { "image" });
    } else if let Some(source) = package.get("wallpaperUrl").and_then(Value::as_str) {
        let url = url::Url::parse(source).map_err(|e| AppError::InvalidInput(e.to_string()))?;
        if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() { return Err(AppError::InvalidInput("Link de wallpaper inválido".into())); }
        value["customWallpaperUrl"] = json!(source);
        value["customWallpaperType"] = json!(if url.path().ends_with(".mp4") || url.path().ends_with(".webm") {"video"} else {"image"});
    } else { value["customWallpaperUrl"] = json!(""); }
    crate::commands::settings::settings_set(value.clone()).await?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    #[test]
    fn themes_do_not_import_account_or_execution_settings() {
        let result = super::appearance(&serde_json::json!({"theme":"default-dark","javaPath":"evil","activeAccountId":"secret","jvmArgs":"evil"})).unwrap();
        assert_eq!(result,serde_json::json!({"theme":"default-dark"}));
        assert!(super::appearance(&serde_json::json!({"theme":"evil"})).is_err());
    }
    #[test]
    fn support_hides_private_information() {
        let value = super::redact("--accessToken secret email@test.com C:\\Users\\preda\\test 192.168.1.2 password=secret");
        assert!(!value.contains("secret")); assert!(!value.contains("preda")); assert!(!value.contains("192.168")); assert!(!value.contains("email@test.com"));
    }
}

#[tauri::command]
pub async fn modpack_validate(profile_id: String) -> AppResult<crate::commands::doctor::InstanceReadiness> {
    let db=crate::db::shared_db().await?;
    let game_dir: String=sqlx::query_scalar("SELECT game_dir FROM profiles WHERE id = ?").bind(&profile_id).fetch_one(db.pool()).await?;
    let rows=crate::db::schema::mods::list_by_profile(&db,&profile_id).await?;
    let root=std::path::PathBuf::from(&game_dir);
    let audit_root=root.clone();
    let audit=tokio::task::spawn_blocking(move || crate::core::mods::validation::audit(&audit_root,true)).await.map_err(|e| AppError::Internal(e.to_string()))??;
    let mut readiness=crate::commands::doctor::doctor_instance_readiness(profile_id).await?;
    readiness.blockers.extend(audit.errors);
    readiness.warnings.extend(audit.warnings);
    for row in rows {
        if row.sha1.len()!=40 {continue;}
        let file=crate::core::mods::pack_download::destination(&root.join("mods"),&row.file_name)?;
        let disabled=file.with_file_name(format!("{}.disabled",row.file_name));
        let path=if file.is_file(){file}else{disabled};
        if !crate::core::mods::pack_download::verify_existing(&path,None,Some(&row.sha1),None,true).await { readiness.blockers.push(format!("Hash divergente ou mod ausente: {}. Reinstale o conteúdo pela fonte original.",row.file_name)); }
    }
    readiness.ready=readiness.blockers.is_empty();
    Ok(readiness)
}

fn scrub_report(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact(&text)),
        Value::Array(values) => Value::Array(values.into_iter().map(scrub_report).collect()),
        Value::Object(values) => Value::Object(values.into_iter().map(|(key,value)| (key,scrub_report(value))).collect()),
        other => other,
    }
}
