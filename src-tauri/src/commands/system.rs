use serde::Serialize;

use crate::db::models::{AccountRow, ProfileRow};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
	pub name: &'static str,
	pub version: &'static str,
	pub identifier: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInitState {
	pub dev_mode: bool,
	pub account: Option<AccountRow>,
	pub profiles: Vec<ProfileRow>,
	pub active_profile_id: Option<String>,
}

#[tauri::command]
pub async fn ping() -> Result<String, crate::error::AppError> {
	Ok("pong".into())
}

#[tauri::command]
pub fn app_info() -> AppInfo {
	AppInfo {
		name: env!("CARGO_PKG_NAME"),
		version: env!("CARGO_PKG_VERSION"),
		identifier: "io.github.luxmc.Luxmc",
	}
}

#[tauri::command]
pub async fn app_init() -> Result<AppInitState, crate::error::AppError> {
	let dev_mode = std::env::var("LUXMC_DEV_MODE").unwrap_or_default() == "1";
	let db = crate::db::shared_db().await?;

	let accounts = crate::db::schema::accounts::list(&db).await?;
	let account = accounts.into_iter().next();

	let profiles = crate::db::schema::profiles::list(&db).await?;

	let settings: serde_json::Value = {
		use sqlx::Row;
		let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'app'")
			.fetch_optional(db.pool())
			.await?;
		match row {
			Some(r) => {
				let raw: String = r.try_get("value")?;
				serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
			}
			None => serde_json::Value::Null,
		}
	};

	let active_profile_id = settings
		.get("activeProfileId")
		.and_then(|v| v.as_str())
		.map(|s| s.to_string());

	Ok(AppInitState {
		dev_mode,
		account,
		profiles,
		active_profile_id,
	})
}
