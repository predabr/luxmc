use serde::{Deserialize, Serialize};

use crate::error::AppResult;

const VERSION_MANIFEST_URL: &str =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionSummary {
    pub id: String,
    #[serde(alias = "type")]
    pub version_type: String,
    pub url: String,
    pub release_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetail {
    pub id: String,
    #[serde(alias = "type")]
    pub version_type: String,
    pub arguments: Option<VersionArguments>,
    pub minecraft_arguments: Option<String>,
    pub main_class: Option<String>,
    pub downloads: Option<VersionDownloads>,
    pub assets: Option<String>,
    pub asset_index: Option<AssetIndex>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub java_version: Option<JavaVersion>,
}

impl VersionDetail {
    pub fn java_major_version(&self) -> u32 {
        self.java_version
            .as_ref()
            .map(|jv| jv.major_version)
            .unwrap_or(8)
    }

    pub fn effective_game_args(&self) -> Vec<String> {
        if let Some(ref args) = self.arguments {
            let mut result = Vec::new();
            for v in &args.game {
                if let Some(s) = v.as_str() {
                    if s != "--demo" {
                        result.push(s.to_string());
                    }
                } else if let Some(obj) = v.as_object() {
                    if let Some(rules_val) = obj.get("rules").and_then(|r| r.as_array()) {
                        if !evaluate_rules(rules_val) {
                            continue;
                        }
                    }
                    if let Some(val) = obj.get("value") {
                        if let Some(s) = val.as_str() {
                            if s != "--demo" {
                                result.push(s.to_string());
                            }
                        } else if let Some(arr) = val.as_array() {
                            result.extend(arr.iter().filter_map(|v| {
                                let s = v.as_str()?;
                                if s == "--demo" {
                                    None
                                } else {
                                    Some(s.to_string())
                                }
                            }));
                        }
                    }
                }
            }
            return result;
        }
        if let Some(ref mc_args) = self.minecraft_arguments {
            return mc_args
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        vec![
            "--username".into(),
            "${auth_player_name}".into(),
            "--gameDir".into(),
            "${game_directory}".into(),
            "--assetsDir".into(),
            "${assets_directory}".into(),
            "--version".into(),
            self.id.clone(),
        ]
    }

    pub fn effective_jvm_args(&self) -> Option<Vec<serde_json::Value>> {
        self.arguments.as_ref().and_then(|a| a.jvm.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionArguments {
    pub game: Vec<serde_json::Value>,
    pub jvm: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDownloads {
    pub client: DownloadEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadEntry {
    pub url: String,
    pub size: u64,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<LibraryRule>>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadEntry>,
    pub classifiers: Option<std::collections::HashMap<String, DownloadEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRule {
    pub action: String,
    pub os: Option<LibraryOs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryOs {
    pub name: String,
}

pub fn lib_url_from_name(name: &str, base_url: Option<&str>) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return String::new();
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let filename = if parts.len() > 3 {
        format!("{}-{}-{}", artifact, version, parts[3])
    } else {
        format!("{}-{}.jar", artifact, version)
    };
    let path = format!("{}/{}/{}/{}", group, artifact, version, filename);
    let base = base_url.unwrap_or("https://libraries.minecraft.net/");
    if base.ends_with('/') {
        format!("{}{}", base, path)
    } else {
        format!("{}/{}", base, path)
    }
}

pub async fn fetch_version_manifest(http: &reqwest::Client) -> AppResult<VersionManifest> {
    let manifest: VersionManifest = http
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(manifest)
}

pub async fn fetch_version_detail(http: &reqwest::Client, url: &str) -> AppResult<VersionDetail> {
    let detail: VersionDetail = http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(detail)
}

pub static GAME_FEATURES: std::sync::LazyLock<std::collections::HashMap<String, bool>> =
    std::sync::LazyLock::new(|| {
        let mut m = std::collections::HashMap::new();
        m.insert("is_demo_user".into(), false);
        m.insert("has_custom_resolution".into(), true);
        m.insert("has_quick_plays_support".into(), false);
        m.insert("is_quick_play_singleplayer".into(), false);
        m.insert("is_quick_play_multiplayer".into(), false);
        m.insert("is_quick_play_realms".into(), false);
        m
    });

fn evaluate_rules(rules: &[serde_json::Value]) -> bool {
    let mut allowed = false;
    let current_os = if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    for rule in rules {
        let action = rule
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("allow");
        let mut matches = true;

        // Check OS rule
        if let Some(os_obj) = rule.get("os").and_then(|os| os.as_object()) {
            if let Some(os_name) = os_obj.get("name").and_then(|n| n.as_str()) {
                if os_name != current_os {
                    matches = false;
                }
            }
        }

        // Check features rule (only if OS matched or no OS rule)
        if matches {
            if let Some(features) = rule.get("features").and_then(|f| f.as_object()) {
                for (key, required) in features {
                    let want = required.as_bool().unwrap_or(false);
                    let have = GAME_FEATURES.get(key.as_str()).copied().unwrap_or(false);
                    if want != have {
                        matches = false;
                        break;
                    }
                }
            }
        }

        allowed = match action {
            "allow" if matches => true,
            "disallow" if matches => false,
            _ => allowed,
        };
    }
    allowed
}

#[cfg(test)]
mod tests {
    use super::{evaluate_rules, JavaVersion, VersionSummary};

    #[test]
    fn demo_feature_rule_is_disabled_by_default() {
        let rules = vec![serde_json::json!({
            "action": "allow",
            "features": { "is_demo_user": true }
        })];

        assert!(!evaluate_rules(&rules));
    }

    #[test]
    fn demo_disallow_when_feature_matches_is_honoured() {
        let rules = vec![serde_json::json!({
            "action": "disallow",
            "features": { "is_demo_user": true }
        })];
        assert!(!evaluate_rules(&rules));
    }

    #[test]
    fn all_official_version_types_are_preserved() {
        let json = r#"[
			{ "id": "1.21.4", "type": "release", "url": "https://x/1.json", "releaseTime": "2024-12-03T10:00:00+00:00" },
			{ "id": "1.21.4-pre1", "type": "snapshot", "url": "https://x/2.json", "releaseTime": "2024-12-02T10:00:00+00:00" },
			{ "id": "26.2", "type": "release", "url": "https://x/3.json", "releaseTime": "2026-09-01T10:00:00+00:00" },
			{ "id": "b1.7.3", "type": "old_beta", "url": "https://x/4.json", "releaseTime": "2011-06-01T10:00:00+00:00" },
			{ "id": "a1.2.6", "type": "old_alpha", "url": "https://x/5.json", "releaseTime": "2010-12-01T10:00:00+00:00" }
		]"#;

        let versions: Vec<VersionSummary> = serde_json::from_str(json).unwrap();
        assert_eq!(versions.len(), 5);
        assert!(versions
            .iter()
            .any(|v| v.id == "26.2" && v.version_type == "release"));
        assert!(versions.iter().any(|v| v.version_type == "snapshot"));
        assert!(versions.iter().any(|v| v.version_type == "old_beta"));
        assert!(versions.iter().any(|v| v.version_type == "old_alpha"));
    }

    #[test]
    fn java_major_version_is_read_per_version() {
        let m_21 = serde_json::json!({
            "component": "java-runtime-delta",
            "majorVersion": 21
        });
        let m_25 = serde_json::json!({
            "component": "java-runtime-delta",
            "majorVersion": 25
        });

        let v21: JavaVersion = serde_json::from_value(m_21).unwrap();
        let v25: JavaVersion = serde_json::from_value(m_25).unwrap();
        assert_eq!(v21.major_version, 21);
        assert_eq!(v25.major_version, 25);
    }

    #[test]
    fn future_versions_are_not_filtered_by_id() {
        // The manifest keeps adding versions; the parser must accept any id.
        let summary: VersionSummary = serde_json::from_str(
            r#"{
				"id": "26.10.0-future",
				"type": "snapshot",
				"url": "https://example/26.10.0.json",
				"releaseTime": "2099-01-01T00:00:00+00:00"
			}"#,
        )
        .unwrap();
        assert_eq!(summary.id, "26.10.0-future");
        assert_eq!(summary.version_type, "snapshot");
    }
}
