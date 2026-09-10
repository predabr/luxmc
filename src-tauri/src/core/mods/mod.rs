use serde::{Deserialize, Serialize};

use crate::error::AppResult;

pub mod curseforge;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModSearchResult {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub source: String,
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModVersion {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub files: Vec<ModFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModDependency {
    pub project_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdate {
    pub project_id: String,
    pub project_name: String,
    pub current_version_id: String,
    pub latest_version_id: String,
    pub latest_version_number: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModVersionDetail {
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub files: Vec<ModFile>,
    pub dependencies: Vec<ModDependency>,
}

pub struct ModrinthClient {
    http: reqwest::Client,
}

impl ModrinthClient {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    pub async fn search_mods(
        &self,
        query: &str,
        mc_version: &str,
        limit: u32,
    ) -> AppResult<Vec<ModSearchResult>> {
        let facets = format!(
            r#"[["categories:fabric","categories:forge","categories:neoforge","categories:quilt"],["versions:{}"]]"#,
            mc_version
        );
        let url = format!(
            "{}/search?query={}&limit={}&facets={}",
            MODRINTH_API,
            urlencoding::encode(query),
            limit,
            urlencoding::encode(&facets)
        );
        let resp: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let hits = resp
            .get("hits")
            .and_then(|h| h.as_array())
            .cloned()
            .unwrap_or_default();
        let mut results = Vec::new();
        for hit in hits {
            results.push(ModSearchResult {
                slug: hit
                    .get("slug")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: hit
                    .get("title")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: hit
                    .get("description")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                downloads: hit.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                icon_url: hit
                    .get("icon_url")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
                categories: hit
                    .get("categories")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                versions: hit
                    .get("versions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                source: "modrinth".into(),
                source_id: hit
                    .get("project_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        Ok(results)
    }

    pub async fn search_typed(
        &self,
        query: &str,
        mc_version: &str,
        content_type: &str,
        limit: u32,
    ) -> AppResult<Vec<ModSearchResult>> {
        let category = match content_type {
            "resourcepack" => "resourcepack",
            "shader" => "shader",
            _ => "mod",
        };
        let facets = format!(r#"[["categories:{}"],["versions:{}"]]"#, category, mc_version);
        let url = format!(
            "{}/search?query={}&limit={}&facets={}",
            MODRINTH_API,
            urlencoding::encode(query),
            limit,
            urlencoding::encode(&facets)
        );
        let resp: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let hits = resp
            .get("hits")
            .and_then(|h| h.as_array())
            .cloned()
            .unwrap_or_default();
        let mut results = Vec::new();
        for hit in hits {
            results.push(ModSearchResult {
                slug: hit
                    .get("slug")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: hit
                    .get("title")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: hit
                    .get("description")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
                downloads: hit.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                icon_url: hit
                    .get("icon_url")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string()),
                categories: hit
                    .get("categories")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                versions: hit
                    .get("versions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                source: "modrinth".into(),
                source_id: hit
                    .get("project_id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        Ok(results)
    }

    pub async fn get_mod_versions(
        &self,
        project_id: &str,
        mc_version: &str,
    ) -> AppResult<Vec<ModVersion>> {
        let url = format!(
			"{}/project/{}/version?game_versions=[\"{}\"]&loaders=[\"fabric\",\"forge\",\"neoforge\",\"quilt\"]",
			MODRINTH_API, project_id, mc_version
		);
        let resp: Vec<serde_json::Value> = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut versions = Vec::new();
        for v in resp {
            let files: Vec<ModFile> = v
                .get("files")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| {
                            Some(ModFile {
                                url: f.get("url").and_then(|u| u.as_str())?.to_string(),
                                filename: f.get("filename").and_then(|u| u.as_str())?.to_string(),
                                size: f.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                                sha1: f
                                    .get("hashes")
                                    .and_then(|h| h.get("sha1"))
                                    .and_then(|h| h.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            versions.push(ModVersion {
                id: v
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string(),
                name: v
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                version_number: v
                    .get("version_number")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                files,
            });
        }
        Ok(versions)
    }

    pub async fn get_latest_version(
        &self,
        project_id: &str,
        mc_version: &str,
        loaders: &[&str],
    ) -> AppResult<Option<(String, String, String)>> {
        let loader_list: String = loaders
            .iter()
            .map(|l| format!("\"{}\"", l))
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "{}/project/{}/version?game_versions=[\"{}\"]&loaders=[{}]",
            MODRINTH_API, project_id, mc_version, loader_list
        );
        let resp: Vec<serde_json::Value> = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let v = resp
            .first()
            .ok_or_else(|| crate::error::AppError::NotFound("no versions found".into()))?;

        let version_id = v
            .get("id")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        let version_number = v
            .get("version_number")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let download_url = v
            .get("files")
            .and_then(|f| f.as_array())
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Some((version_id, version_number, download_url)))
    }

    pub async fn get_version_detail(
        &self,
        project_id: &str,
        version_id: &str,
        mc_version: &str,
    ) -> AppResult<ModVersionDetail> {
        let url = format!(
			"{}/project/{}/version?game_versions=[\"{}\"]&loaders=[\"fabric\",\"forge\",\"neoforge\",\"quilt\"]",
			MODRINTH_API, project_id, mc_version
		);
        let resp: Vec<serde_json::Value> = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        for v in resp {
            let id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
            if id != version_id {
                continue;
            }
            let files: Vec<ModFile> = v
                .get("files")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| {
                            Some(ModFile {
                                url: f.get("url").and_then(|u| u.as_str())?.to_string(),
                                filename: f.get("filename").and_then(|u| u.as_str())?.to_string(),
                                size: f.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                                sha1: f
                                    .get("hashes")
                                    .and_then(|h| h.get("sha1"))
                                    .and_then(|h| h.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let dependencies: Vec<ModDependency> = v
                .get("dependencies")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|d| {
                            Some(ModDependency {
                                project_id: d
                                    .get("project_id")
                                    .and_then(|p| p.as_str())?
                                    .to_string(),
                                dependency_type: d
                                    .get("dependency_type")
                                    .and_then(|t| t.as_str())?
                                    .to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            return Ok(ModVersionDetail {
                id: id.to_string(),
                name: v
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                version_number: v
                    .get("version_number")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                files,
                dependencies,
            });
        }
        Err(crate::error::AppError::NotFound(
            "mod version not found".into(),
        ))
    }
}
