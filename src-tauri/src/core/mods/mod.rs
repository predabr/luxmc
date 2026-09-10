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
    pub banner_url: Option<String>,
    pub author: Option<String>,
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
pub struct ModGalleryImage {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModAuthor {
    pub name: String,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModProjectDetails {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_type: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub latest_version: Option<String>,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_url: Option<String>,
    pub author: Option<ModAuthor>,
    pub gallery: Vec<ModGalleryImage>,
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
        content_type: &str,
        limit: u32,
        offset: u32,
        sort_by: Option<&str>,
    ) -> AppResult<Vec<ModSearchResult>> {
        let project_type = match content_type.to_lowercase().as_str() {
            "modpack" => "modpack",
            "resource pack" | "resourcepack" => "resourcepack",
            "shader" => "shader",
            "data pack" | "datapack" => "datapack",
            _ => "mod",
        };

        let mut facet_groups: Vec<String> = Vec::new();
        facet_groups.push(format!(r#"["project_type:{}"]"#, project_type));

        if project_type == "mod" {
            facet_groups.push(r#"["categories:fabric","categories:forge","categories:neoforge","categories:quilt"]"#.to_string());
        }

        if !mc_version.is_empty() && mc_version != "Qualquer Versão" {
            facet_groups.push(format!(r#"["versions:{}"]"#, mc_version));
        }

        let facets = format!("[{}]", facet_groups.join(","));
        let sort_index = match sort_by.unwrap_or("downloads") {
            "relevance" => "relevance",
            "updated" => "updated",
            "newest" => "newest",
            _ => "downloads",
        };

        let url = format!(
            "{}/search?query={}&limit={}&offset={}&index={}&facets={}",
            MODRINTH_API,
            urlencoding::encode(query.trim()),
            limit,
            offset,
            sort_index,
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
            let banner_url = hit
                .get("featured_gallery")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    hit.get("gallery")
                        .and_then(|g| g.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });

            let author = hit
                .get("author")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());

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
                banner_url,
                author,
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
        self.search_mods(query, mc_version, content_type, limit, 0, None).await
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

    pub async fn get_project_details(&self, id_or_slug: &str) -> AppResult<ModProjectDetails> {
        let url = format!("{}/project/{}", MODRINTH_API, urlencoding::encode(id_or_slug));
        let resp: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let id = resp.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let slug = resp.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let title = resp.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = resp.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let body = resp.get("body").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let icon_url = resp.get("icon_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let downloads = resp.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);
        let updated_at = resp.get("updated").and_then(|v| v.as_str()).map(|s| s.to_string());
        let created_at = resp.get("published").and_then(|v| v.as_str()).map(|s| s.to_string());
        let source_url = resp.get("source_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let issues_url = resp.get("issues_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let wiki_url = resp.get("wiki_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let discord_url = resp.get("discord_url").and_then(|v| v.as_str()).map(|s| s.to_string());
        let donation_url = resp
            .get("donation_urls")
            .and_then(|arr| arr.as_array())
            .and_then(|a| a.first())
            .and_then(|d| d.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let categories: Vec<String> = resp
            .get("categories")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let loaders: Vec<String> = resp
            .get("loaders")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let game_versions: Vec<String> = resp
            .get("game_versions")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let gallery: Vec<ModGalleryImage> = resp
            .get("gallery")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|img| {
                        let url = img.get("url")?.as_str()?.to_string();
                        let title = img.get("title").and_then(|t| t.as_str()).map(|s| s.to_string());
                        let description = img
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string());
                        Some(ModGalleryImage {
                            url,
                            title,
                            description,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut author = None;
        let members_url = format!("{}/project/{}/members", MODRINTH_API, urlencoding::encode(&id));
        if let Ok(m_resp) = self.http.get(&members_url).send().await {
            if let Ok(m_arr) = m_resp.json::<Vec<serde_json::Value>>().await {
                if let Some(first_member) = m_arr.first() {
                    let role = first_member
                        .get("role")
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string());
                    if let Some(user_obj) = first_member.get("user") {
                        let name = user_obj
                            .get("username")
                            .and_then(|u| u.as_str())
                            .unwrap_or("Modder")
                            .to_string();
                        let avatar_url = user_obj
                            .get("avatar_url")
                            .and_then(|a| a.as_str())
                            .map(|s| s.to_string());
                        author = Some(ModAuthor {
                            name,
                            avatar_url,
                            role,
                        });
                    }
                }
            }
        }

        let latest_version = game_versions.last().cloned();

        Ok(ModProjectDetails {
            id,
            slug,
            title,
            description,
            body,
            body_type: "markdown".to_string(),
            icon_url,
            downloads,
            categories,
            loaders,
            game_versions,
            latest_version,
            updated_at,
            created_at,
            source: "modrinth".to_string(),
            source_url,
            issues_url,
            discord_url,
            wiki_url,
            donation_url,
            author,
            gallery,
        })
    }
}
