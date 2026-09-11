use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AccountRow {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub refresh_token: String,
    pub access_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRow {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub resolution_w: Option<i64>,
    pub resolution_h: Option<i64>,
    pub fullscreen: bool,
    pub game_dir: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub last_played: Option<DateTime<Utc>>,
    #[serde(default)]
    pub launch_count: i64,
    #[serde(default)]
    pub mod_count: i64,
    #[serde(default)]
    pub disk_usage: i64,
    #[serde(default)]
    pub ram_mb: Option<i64>,
    #[serde(default)]
    pub instance_group: Option<String>,
    #[serde(default = "default_true")]
    pub auto_optimize: bool,
    #[serde(default)]
    pub use_vulkan: bool,
}

fn default_true() -> bool {
    true
}
