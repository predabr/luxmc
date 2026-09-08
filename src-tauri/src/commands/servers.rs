use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::server::{self, ServerStatus};
use crate::error::AppResult;
use crate::state::AppState;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerRow {
	pub id: String,
	pub name: String,
	pub host: String,
	pub port: i64,
	pub favorite: bool,
	pub created_at: String,
}

#[tauri::command]
pub async fn server_ping(
	_state: State<'_, AppState>,
	host: String,
	port: u16,
) -> AppResult<ServerStatus> {
	tokio::task::spawn_blocking(move || server::ping(&host, port))
		.await
		.map_err(|e| crate::error::AppError::Internal(e.to_string()))?
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn server_add(
	_state: State<'_, AppState>,
	host: String,
	port: u16,
	name: String,
) -> AppResult<ServerRow> {
	let db = crate::db::shared_db().await?;
	let id = Uuid::new_v4().to_string();
	let row = ServerRow {
		id: id.clone(),
		name,
		host,
		port: port as i64,
		favorite: false,
		created_at: String::new(),
	};
	let db_row = crate::db::schema::servers::ServerRow {
		id: row.id.clone(),
		name: row.name.clone(),
		host: row.host.clone(),
		port: row.port,
		favorite: row.favorite,
		created_at: row.created_at.clone(),
	};
	crate::db::schema::servers::insert(&db, &db_row).await?;
	Ok(row)
}

#[tauri::command]
pub async fn server_list() -> AppResult<Vec<ServerRow>> {
	let db = crate::db::shared_db().await?;
	let rows = crate::db::schema::servers::list(&db).await?;
	Ok(rows
		.into_iter()
		.map(|r| ServerRow {
			id: r.id,
			name: r.name,
			host: r.host,
			port: r.port,
			favorite: r.favorite,
			created_at: r.created_at,
		})
		.collect())
}

#[tauri::command]
pub async fn server_remove(id: String) -> AppResult<()> {
	let db = crate::db::shared_db().await?;
	crate::db::schema::servers::delete(&db, &id).await
}

#[tauri::command]
pub async fn server_favorites_list() -> AppResult<Vec<ServerRow>> {
	let db = crate::db::shared_db().await?;
	let rows = crate::db::schema::servers::list_favorites(&db).await?;
	Ok(rows
		.into_iter()
		.map(|r| ServerRow {
			id: r.id,
			name: r.name,
			host: r.host,
			port: r.port,
			favorite: r.favorite,
			created_at: r.created_at,
		})
		.collect())
}

#[tauri::command]
pub async fn server_favorite(id: String, favorite: bool) -> AppResult<()> {
	let db = crate::db::shared_db().await?;
	crate::db::schema::servers::set_favorite(&db, &id, favorite).await
}
