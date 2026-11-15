use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{ModAuthor, ModFile, ModGalleryImage, ModProjectDetails, ModSearchResult, ModVersion};
use crate::error::{AppError, AppResult};

const FTB_API: &str = "https://api.feed-the-beast.com/v1/modpacks";
const CATALOG_TTL: Duration = Duration::from_secs(1800);
const CATALOG_CONCURRENCY: usize = 8;

static CATALOG: tokio::sync::Mutex<Option<(Instant, Arc<Vec<serde_json::Value>>)>> = tokio::sync::Mutex::const_new(None);

#[derive(Debug, Clone)]
pub struct PackFile {
    pub path: String,
    pub name: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub client_only: bool,
    pub server_only: bool,
    pub kind: String,
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 20 && value.chars().all(|character| character.is_ascii_digit())
}

fn art<'a>(pack: &'a serde_json::Value, kind: &str) -> Option<&'a str> {
    pack.get("art")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.iter().find(|item| item["type"].as_str() == Some(kind)))
        .and_then(|item| item["url"].as_str())
}

fn icon(pack: &serde_json::Value) -> Option<String> {
    art(pack, "square").or_else(|| art(pack, "logo")).or_else(|| art(pack, "icon")).map(str::to_owned)
}

fn banner(pack: &serde_json::Value) -> Option<String> {
    art(pack, "splash").or_else(|| art(pack, "banner")).or_else(|| art(pack, "screenshot")).map(str::to_owned)
}

fn pack_versions(pack: &serde_json::Value) -> Vec<&serde_json::Value> {
    pack.get("versions").and_then(serde_json::Value::as_array).map(|items| items.iter().collect()).unwrap_or_default()
}

fn target<'a>(version: &'a serde_json::Value, kind: &str) -> Option<&'a serde_json::Value> {
    version.get("targets").and_then(serde_json::Value::as_array).and_then(|items| items.iter().find(|item| item["type"].as_str() == Some(kind)))
}

fn game_versions(pack: &serde_json::Value) -> Vec<String> {
    let mut values: Vec<String> = pack_versions(pack).into_iter().filter_map(|version| target(version, "game")).filter_map(|target| target["version"].as_str()).map(str::to_owned).collect();
    values.sort_by(|left, right| right.cmp(left));
    values.dedup();
    values
}

fn loaders(pack: &serde_json::Value) -> Vec<String> {
    let mut values: Vec<String> = pack_versions(pack).into_iter().filter_map(|version| target(version, "modloader")).filter_map(|target| target["name"].as_str()).map(str::to_lowercase).filter(|name| matches!(name.as_str(), "forge" | "neoforge" | "fabric" | "quilt")).collect();
    values.sort();
    values.dedup();
    values
}

fn categories(pack: &serde_json::Value) -> Vec<String> {
    pack.get("tags").and_then(serde_json::Value::as_array).map(|items| items.iter().filter_map(|tag| tag["name"].as_str()).map(|name| name.to_lowercase().replace(' ', "-")).collect()).unwrap_or_default()
}

fn author(pack: &serde_json::Value) -> Option<ModAuthor> {
    let first = pack.get("authors").and_then(serde_json::Value::as_array).and_then(|items| items.first())?;
    Some(ModAuthor { name: first["name"].as_str()?.to_owned(), avatar_url: None, role: Some("team".into()) })
}

fn synopsis(pack: &serde_json::Value) -> String {
    pack.get("synopsis").and_then(serde_json::Value::as_str).map(str::trim).filter(|value| !value.is_empty())
        .or_else(|| pack.get("description").and_then(serde_json::Value::as_str).map(str::trim))
        .unwrap_or_default()
        .chars().take(400).collect()
}

fn installed_count(pack: &serde_json::Value) -> u64 {
    pack["installs"].as_u64().or_else(|| pack["plays"].as_u64()).unwrap_or(0)
}

fn published(pack: &serde_json::Value) -> String {
    let seconds = pack_versions(pack).iter().filter_map(|version| version["released"].as_i64()).max().unwrap_or(0);
    match chrono::DateTime::from_timestamp(seconds, 0) { Some(date) => date.to_rfc3339(), None => String::new() }
}

fn to_search_result(pack: &serde_json::Value) -> ModSearchResult {
    ModSearchResult {
        slug: pack["slug"].as_str().unwrap_or_default().to_owned(),
        title: pack["name"].as_str().unwrap_or_default().to_owned(),
        description: synopsis(pack),
        downloads: installed_count(pack),
        icon_url: icon(pack),
        banner_url: banner(pack),
        author: author(pack).map(|author| author.name),
        categories: categories(pack),
        versions: game_versions(pack),
        source: "ftb".into(),
        source_id: pack["id"].as_u64().map(|id| id.to_string()).unwrap_or_default(),
    }
}

fn matches_pack(pack: &serde_json::Value, query: &str, mc_version: &str, loader: Option<&str>, category: Option<&str>) -> bool {
    if pack["private"] == true || pack["id"].as_u64().is_none() { return false; }
    if !query.trim().is_empty() {
        let needle = query.trim().to_lowercase();
        let haystack = format!(
            "{} {} {} {}",
            pack["name"].as_str().unwrap_or_default(),
            pack["slug"].as_str().unwrap_or_default(),
            pack["synopsis"].as_str().unwrap_or_default(),
            pack.get("tags").and_then(serde_json::Value::as_array).map(|items| items.iter().filter_map(|tag| tag["name"].as_str()).collect::<Vec<_>>().join(" ")).unwrap_or_default()
        ).to_lowercase();
        if !haystack.contains(&needle) { return false; }
    }
    if !mc_version.trim().is_empty() && !mc_version.eq_ignore_ascii_case("all") && mc_version != "Qualquer Versão" && !game_versions(pack).iter().any(|value| value == mc_version.trim()) { return false; }
    if let Some(loader) = loader.map(str::trim).filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("all") && *value != "Qualquer loader") {
        if !loaders(pack).iter().any(|value| value == &loader.to_lowercase()) { return false; }
    }
    if let Some(category) = category.map(str::trim).filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("all")) {
        let needle = category.to_lowercase().replace(' ', "-");
        if !categories(pack).iter().any(|value| value.contains(&needle) || needle.contains(value.as_str())) { return false; }
    }
    true
}

async fn fetch_pack(http: &reqwest::Client, id: u64) -> Option<serde_json::Value> {
    let value = http.get(format!("{FTB_API}/public/modpack/{id}")).timeout(Duration::from_secs(20)).send().await.ok()?.error_for_status().ok()?.json::<serde_json::Value>().await.ok()?;
    if value["id"].as_u64()? != id { return None; }
    Some(value)
}

pub async fn catalog(http: &reqwest::Client) -> AppResult<Arc<Vec<serde_json::Value>>> {
    let mut cache = CATALOG.lock().await;
    if let Some((when, packs)) = cache.as_ref() {
        if when.elapsed() < CATALOG_TTL { return Ok(packs.clone()); }
    }
    let ids: Vec<u64> = http.get(format!("{FTB_API}/public/modpack/all")).timeout(Duration::from_secs(20)).send().await?.error_for_status()?.json::<serde_json::Value>().await?["packs"]
        .as_array().map(|items| items.iter().filter_map(serde_json::Value::as_u64).collect()).unwrap_or_default();
    if ids.is_empty() { return Err(AppError::InvalidState("O catálogo do Feed The Beast respondeu sem pacotes.".into())); }
    let queue = Arc::new(tokio::sync::Mutex::new(ids));
    let packs = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(96)));
    let mut workers = Vec::new();
    for _ in 0..CATALOG_CONCURRENCY {
        let queue = queue.clone();
        let packs = packs.clone();
        let http = http.clone();
        workers.push(tokio::spawn(async move {
            loop {
                let Some(id) = queue.lock().await.pop() else { break; };
                if let Some(pack) = fetch_pack(&http, id).await { packs.lock().await.push(pack); }
            }
        }));
    }
    for worker in workers { let _ = worker.await; }
    let packs = Arc::try_unwrap(packs).map_err(|_| AppError::Internal("catálogo FTB em uso".into()))?.into_inner();
    if packs.is_empty() { return Err(AppError::InvalidState("Não foi possível carregar os pacotes do Feed The Beast.".into())); }
    let packs = Arc::new(packs);
    *cache = Some((Instant::now(), packs.clone()));
    Ok(packs)
}

pub async fn search(http: &reqwest::Client, query: &str, mc_version: &str, loader: Option<&str>, category: Option<&str>, limit: u32, offset: u32, sort_by: Option<&str>) -> AppResult<Vec<ModSearchResult>> {
    let packs = catalog(http).await?;
    let mut matched: Vec<&serde_json::Value> = packs.iter().filter(|pack| matches_pack(pack, query, mc_version, loader, category)).collect();
    match sort_by.unwrap_or("downloads") {
        "relevance" if !query.trim().is_empty() => {
            let needle = query.trim().to_lowercase();
            matched.sort_by_key(|pack| !pack["name"].as_str().unwrap_or_default().to_lowercase().contains(&needle));
        }
        "newest" | "updated" => matched.sort_by_key(|pack| std::cmp::Reverse(published(pack))),
        _ => matched.sort_by_key(|pack| std::cmp::Reverse(installed_count(pack))),
    }
    Ok(matched.into_iter().skip(offset as usize).take(limit as usize).map(to_search_result).collect())
}

fn version_entry(version: &serde_json::Value) -> Option<ModVersion> {
    let id = version["id"].as_u64()?.to_string();
    Some(ModVersion {
        id,
        name: version["name"].as_str().unwrap_or_default().to_owned(),
        version_number: version["name"].as_str().unwrap_or_default().to_owned(),
        files: Vec::new(),
        loaders: target(version, "modloader").and_then(|value| value["name"].as_str()).map(|name| vec![name.to_lowercase()]).unwrap_or_default(),
        game_versions: target(version, "game").and_then(|value| value["version"].as_str()).map(|value| vec![value.to_owned()]).unwrap_or_default(),
        release_type: Some(version["type"].as_str().unwrap_or("release").to_owned()),
        published_at: version["released"].as_i64().and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0)).map(|date| date.to_rfc3339()),
    })
}

pub async fn versions(http: &reqwest::Client, project_id: &str, mc_version: &str) -> AppResult<Vec<ModVersion>> {
    if !valid_id(project_id) { return Err(AppError::InvalidInput("Projeto FTB inválido".into())); }
    let pack = fetch_pack(http, project_id.parse::<u64>().map_err(|_| AppError::InvalidInput("Projeto FTB inválido".into()))?).await.ok_or_else(|| AppError::NotFound("Modpack FTB não encontrado".into()))?;
    let filter = mc_version.trim().to_owned();
    let filtered = !filter.is_empty() && !filter.eq_ignore_ascii_case("all") && filter != "Qualquer Versão";
    Ok(pack_versions(&pack).into_iter().filter(|version| !filtered || target(version, "game").and_then(|value| value["version"].as_str()) == Some(filter.as_str())).filter_map(version_entry).collect())
}

pub async fn version(http: &reqwest::Client, project_id: &str, version_id: &str) -> AppResult<serde_json::Value> {
    if !valid_id(project_id) || !valid_id(version_id) { return Err(AppError::InvalidInput("Versão FTB inválida".into())); }
    let value: serde_json::Value = http.get(format!("{FTB_API}/public/modpack/{project_id}/{version_id}")).timeout(Duration::from_secs(25)).send().await?.error_for_status()?.json().await?;
    if value["id"].as_u64().map(|id| id.to_string()).as_deref() != Some(version_id) || value["parent"].as_u64().map(|id| id.to_string()).as_deref() != Some(project_id) {
        return Err(AppError::InvalidInput("A versão do FTB não pertence a este modpack".into()));
    }
    Ok(value)
}

fn entry_path(path: &str, name: &str) -> Option<String> {
    let clean = path.trim_start_matches("./").replace('\\', "/");
    let directory = clean.trim_matches('/');
    if directory == "." || directory.contains("..") || name.is_empty() || name == ".gitkeep" || name.contains('/') || name.contains('\\') { return None; }
    if directory.is_empty() { return Some(name.to_owned()); }
    Some(format!("{directory}/{name}"))
}

pub async fn pack_files(http: &reqwest::Client, project_id: &str, version_id: &str) -> AppResult<Vec<PackFile>> {
    let value = version(http, project_id, version_id).await?;
    let files = value["files"].as_array().ok_or_else(|| AppError::NotFound("A versão do FTB não listou arquivos".into()))?;
    let mut parsed = Vec::with_capacity(files.len());
    for file in files {
        let Some(path) = file["path"].as_str() else { continue; };
        let name = file["name"].as_str().unwrap_or_default();
        if file["serveronly"] == true { continue; }
        let Some(full) = entry_path(path, name) else { continue; };
        let Some(url) = file["url"].as_str() else { continue; };
        if url.is_empty() { continue; }
        parsed.push(PackFile {
            path: full,
            name: name.to_owned(),
            url: url.to_owned(),
            sha1: file["hashes"]["sha1"].as_str().or_else(|| file["sha1"].as_str()).unwrap_or_default().to_owned(),
            size: file["size"].as_u64().unwrap_or(0),
            client_only: file["clientonly"] == true,
            server_only: false,
            kind: file["type"].as_str().unwrap_or_default().to_owned(),
        });
    }
    Ok(parsed)
}

pub async fn details(http: &reqwest::Client, project_id: &str) -> AppResult<ModProjectDetails> {
    if !valid_id(project_id) { return Err(AppError::InvalidInput("Projeto FTB inválido".into())); }
    let pack = fetch_pack(http, project_id.parse::<u64>().map_err(|_| AppError::InvalidInput("Projeto FTB inválido".into()))?).await.ok_or_else(|| AppError::NotFound("Modpack FTB não encontrado".into()))?;
    let link = |kind: &str| pack.get("links").and_then(serde_json::Value::as_array).and_then(|items| items.iter().find(|item| item["type"].as_str() == Some(kind) || item["name"].as_str().map(|name| name.eq_ignore_ascii_case(kind)).unwrap_or(false))).and_then(|item| item["link"].as_str()).map(str::to_owned);
    let latest = pack_versions(&pack).into_iter().filter(|version| version["type"].as_str() != Some("archived")).max_by_key(|version| version["released"].as_i64().unwrap_or(0));
    Ok(ModProjectDetails {
        id: project_id.to_owned(),
        slug: pack["slug"].as_str().unwrap_or_default().to_owned(),
        title: pack["name"].as_str().unwrap_or_default().to_owned(),
        description: synopsis(&pack),
        body: pack["description"].as_str().unwrap_or_default().to_owned(),
        body_type: "markdown".into(),
        icon_url: icon(&pack),
        downloads: installed_count(&pack),
        categories: categories(&pack),
        loaders: loaders(&pack),
        game_versions: game_versions(&pack),
        latest_version: latest.and_then(|version| version["name"].as_str()).map(str::to_owned),
        updated_at: pack["updated"].as_i64().and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0)).map(|date| date.to_rfc3339()),
        created_at: None,
        source: "ftb".into(),
        source_url: Some(format!("https://www.feed-the-beast.com/modpacks/{}", pack["id"].as_u64().unwrap_or_default())),
        issues_url: link("Issue Tracker"),
        wiki_url: link("Wiki"),
        discord_url: link("Discord"),
        donation_url: None,
        author: author(&pack),
        gallery: pack.get("art").and_then(serde_json::Value::as_array).map(|items| items.iter().filter(|item| matches!(item["type"].as_str(), Some("screenshot") | Some("splash"))).filter_map(|item| item["url"].as_str()).map(|url| ModGalleryImage { url: url.to_owned(), title: None, description: None }).collect()).unwrap_or_default(),
    })
}

pub async fn changelog(http: &reqwest::Client, project_id: &str, version_id: &str) -> AppResult<String> {
    let value = version(http, project_id, version_id).await?;
    let Some(url) = value["changelog"].as_str().filter(|value| !value.is_empty()) else { return Ok(String::new()); };
    if !url.starts_with("https://api.feed-the-beast.com/") { return Err(AppError::InvalidInput("Changelog FTB inválido".into())); }
    let response = http.get(url).timeout(Duration::from_secs(20)).send().await?;
    if !response.status().is_success() { return Ok(String::new()); }
    let text = response.text().await?;
    Ok(text.chars().take(200_000).collect())
}

pub fn latest_release<'a>(pack: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
    pack_versions(pack).into_iter().filter(|version| version["type"].as_str() != Some("archived")).max_by_key(|version| version["released"].as_i64().unwrap_or(0))
}

pub fn release_version_id(version: &serde_json::Value) -> Option<String> {
    version["id"].as_u64().map(|id| id.to_string())
}

pub fn release_version_name(version: &serde_json::Value) -> Option<String> {
    version["name"].as_str().map(str::to_owned)
}

pub fn target_minecraft(version: &serde_json::Value) -> Option<String> {
    target(version, "game").and_then(|value| value["version"].as_str()).map(str::to_owned)
}

pub fn target_loader(version: &serde_json::Value) -> Option<(String, String)> {
    let loader = target(version, "modloader")?;
    Some((loader["name"].as_str()?.to_lowercase(), loader["version"].as_str().unwrap_or_default().to_owned()))
}

pub fn target_java(version: &serde_json::Value) -> Option<String> {
    target(version, "runtime").and_then(|value| value["version"].as_str()).map(str::to_owned)
}

pub async fn pack_project(http: &reqwest::Client, project_id: &str) -> AppResult<serde_json::Value> {
    if !valid_id(project_id) { return Err(AppError::InvalidInput("Projeto FTB inválido".into())); }
    fetch_pack(http, project_id.parse::<u64>().map_err(|_| AppError::InvalidInput("Projeto FTB inválido".into()))?).await.ok_or_else(|| AppError::NotFound("Modpack FTB não encontrado".into()))
}

pub fn mod_file(path: &str, url: &str, sha1: &str, size: u64) -> ModFile {
    ModFile { url: url.to_owned(), filename: path.rsplit('/').next().unwrap_or(path).to_owned(), size, sha1: sha1.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> serde_json::Value {
        serde_json::json!({
            "id": 135, "name": "FTB Evolution 2", "slug": "ftb-evolution-2", "synopsis": "Evolve again", "private": false,
            "installs": 164907, "updated": 1791482945,
            "art": [{"type": "square", "url": "https://cdn.feed-the-beast.com/blob/a.webp"}, {"type": "splash", "url": "https://cdn.feed-the-beast.com/blob/b.webp"}],
            "tags": [{"name": "Tech"}, {"name": "Kitchen Sink"}],
            "authors": [{"name": "FTB Team"}],
            "versions": [
                {"id": 100553, "name": "1.1.0", "type": "release", "released": 1791648902, "targets": [{"type": "game", "version": "26.1.2"}, {"type": "modloader", "name": "neoforge", "version": "26.1.2.109"}]},
                {"id": 100548, "name": "1.0.0-rc.1", "type": "archived", "released": 1791404702, "targets": [{"type": "game", "version": "26.1.2"}, {"type": "modloader", "name": "neoforge", "version": "26.1.2.109"}]}
            ]
        })
    }

    #[test]
    fn maps_a_pack_into_a_catalog_entry() {
        let result = to_search_result(&pack());
        assert_eq!(result.source, "ftb");
        assert_eq!(result.source_id, "135");
        assert_eq!(result.title, "FTB Evolution 2");
        assert_eq!(result.downloads, 164907);
        assert_eq!(result.icon_url.as_deref(), Some("https://cdn.feed-the-beast.com/blob/a.webp"));
        assert_eq!(result.banner_url.as_deref(), Some("https://cdn.feed-the-beast.com/blob/b.webp"));
        assert_eq!(result.versions, vec!["26.1.2".to_string()]);
        assert_eq!(result.author.as_deref(), Some("FTB Team"));
        assert!(result.categories.contains(&"tech".to_string()));
        assert!(result.categories.contains(&"kitchen-sink".to_string()));
    }

    #[test]
    fn filters_by_name_version_loader_and_category() {
        let pack = pack();
        assert!(matches_pack(&pack, "evolution", "26.1.2", Some("neoforge"), Some("tech")));
        assert!(matches_pack(&pack, "", "", None, None));
        assert!(!matches_pack(&pack, "tongue tied", "", None, None));
        assert!(!matches_pack(&pack, "", "1.20.1", None, None));
        assert!(!matches_pack(&pack, "", "", Some("fabric"), None));
        assert!(!matches_pack(&pack, "", "", None, Some("adventure")));
    }

    #[test]
    fn excludes_private_and_archived_versions() {
        let mut private = pack();
        private["private"] = serde_json::json!(true);
        assert!(!matches_pack(&private, "", "", None, None));
        let catalogue = pack();
        let latest = latest_release(&catalogue).unwrap();
        assert_eq!(release_version_name(latest).as_deref(), Some("1.1.0"));
        assert_eq!(release_version_id(latest).as_deref(), Some("100553"));
    }

    #[test]
    fn version_entries_keep_targets() {
        let catalogue = pack();
        let versions = pack_versions(&catalogue);
        let mapped = version_entry(versions[0]).unwrap();
        assert_eq!(mapped.id, "100553");
        assert_eq!(mapped.version_number, "1.1.0");
        assert_eq!(mapped.loaders, vec!["neoforge".to_string()]);
        assert_eq!(mapped.game_versions, vec!["26.1.2".to_string()]);
        assert!(mapped.files.is_empty());
    }

    #[test]
    fn joins_pack_directory_and_file_names() {
        assert_eq!(entry_path("./config", "evilcraft-common.toml").as_deref(), Some("config/evilcraft-common.toml"));
        assert_eq!(entry_path("./config/", "evilcraft-common.toml").as_deref(), Some("config/evilcraft-common.toml"));
        assert_eq!(entry_path("./datapacks/datapack/data/a/b/", "c.json").as_deref(), Some("datapacks/datapack/data/a/b/c.json"));
        assert_eq!(entry_path("./mods", "mod.jar").as_deref(), Some("mods/mod.jar"));
        assert_eq!(entry_path("./", "mod.jar").as_deref(), Some("mod.jar"));
        assert_eq!(entry_path("./mods", "../escape.jar"), None);
        assert_eq!(entry_path("./mods/../mods", "mod.jar"), None);
        assert_eq!(entry_path("./mods", ".gitkeep"), None);
        assert_eq!(entry_path(".", "mod.jar"), None);
        assert_eq!(entry_path("./mods", ""), None);
    }

    #[test]
    fn rejects_non_numeric_identifiers() {
        assert!(valid_id("135"));
        assert!(!valid_id(""));
        assert!(!valid_id("../135"));
        assert!(!valid_id("135?x=1"));
    }
}
