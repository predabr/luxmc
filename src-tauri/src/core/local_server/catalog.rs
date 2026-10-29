use super::*;
use crate::core::mods::{curseforge, pack_download, ModVersionDetail, ModrinthClient};
use std::collections::{BTreeMap, HashSet};

fn compatible(loaders: &[String], kind: &str, loader: &str) -> bool {
    let target = match kind {
        "datapack" => "datapack",
        "resourcepack" => "minecraft",
        _ => loader,
    };
    loaders
        .iter()
        .any(|value| value == target || (kind == "mod" && loader == "quilt" && value == "fabric"))
        || (kind == "resourcepack" && loaders.is_empty())
}

impl LocalServerManager {
    pub async fn download_content(
        &self,
        id: &str,
        project: &str,
        source: &str,
        kind: &str,
    ) -> AppResult<Vec<String>> {
        let _job = self.content_job()?;
        if !["mod", "datapack", "resourcepack"].contains(&kind)
            || !["modrinth", "curseforge"].contains(&source)
            || project.is_empty()
            || project.len() > 96
            || !project
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || b"_-".contains(&value))
        {
            return Err(invalid("Projeto ou provedor inválido."));
        }
        if source == "curseforge" && kind != "mod" {
            return Err(invalid(
                "Use Modrinth ou importe o ZIP local para este tipo de conteúdo de servidor.",
            ));
        }
        let config = self.config(id).await?;
        let loader = self.loader(id).await?;
        if kind == "mod" && loader.kind == "vanilla" {
            return Err(invalid(
                "Configure um loader de servidor antes de instalar mods.",
            ));
        }
        let client = ModrinthClient::new(self.http.clone());
        let mut queue = vec![(project.to_string(), None::<String>)];
        let mut planned = BTreeMap::<String, ModVersionDetail>::new();
        while let Some((project, pin)) = queue.pop() {
            pack_download::cancelled(Some(&self.shutting_down))?;
            if planned.len() >= 128 {
                return Err(invalid(
                    "Dependências demais; nenhuma alteração foi aplicada ao servidor.",
                ));
            }
            if source == "modrinth" {
                let mut response = self
                    .http
                    .get(format!("https://api.modrinth.com/v2/project/{project}"))
                    .timeout(Duration::from_secs(20))
                    .send()
                    .await?
                    .error_for_status()?;
                let mut bytes = Vec::new();
                while let Some(chunk) = response.chunk().await? {
                    if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                        return Err(invalid("Metadados do projeto muito grandes."));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                let metadata: serde_json::Value = serde_json::from_slice(&bytes)?;
                if metadata["project_type"].as_str()
                    != Some(if kind == "datapack" { "mod" } else { kind })
                {
                    return Err(invalid(
                        "O projeto não corresponde ao tipo de conteúdo selecionado.",
                    ));
                }
                if kind == "mod" && metadata["server_side"].as_str() == Some("unsupported") {
                    return Err(invalid(format!(
                        "O projeto {project} é exclusivo do cliente e foi excluído do servidor."
                    )));
                }
            }
            let candidates = if source == "curseforge" {
                curseforge::get_mod_versions(&self.http, &project, &config.mc_version).await?
            } else {
                client
                    .get_mod_versions(&project, &config.mc_version)
                    .await?
            };
            let selected = candidates.into_iter().find(|version| compatible(&version.loaders, kind, &loader.kind) && pin.as_ref().is_none_or(|pin| *pin == version.id)).ok_or_else(|| invalid(format!("{project} não oferece uma versão compatível com Minecraft {} / {} para este servidor.", config.mc_version, loader.kind)))?;
            let version = if source == "curseforge" {
                ModVersionDetail {
                    id: selected.id.clone(),
                    project_id: project.clone(),
                    game_versions: selected.game_versions,
                    loaders: selected.loaders,
                    name: selected.name,
                    version_number: selected.version_number,
                    files: vec![
                        curseforge::get_mod_file_details(&self.http, &project, &selected.id)
                            .await?,
                    ],
                    dependencies: curseforge::file_dependencies(&self.http, &project, &selected.id)
                        .await?,
                }
            } else {
                client
                    .get_version_detail(&project, &selected.id, &config.mc_version)
                    .await?
            };
            if let Some(previous) = planned.get(&version.project_id) {
                if previous.id != version.id {
                    return Err(invalid("Os mods exigem versões diferentes da mesma dependência. Nenhuma alteração foi aplicada."));
                }
                continue;
            }
            if kind == "mod" {
                for dependency in &version.dependencies {
                    if dependency.dependency_type == "required" {
                        if dependency.project_id.is_empty() {
                            return Err(invalid("Uma dependência externa precisa ser revisada manualmente. Nenhuma alteração foi aplicada."));
                        }
                        queue.push((dependency.project_id.clone(), dependency.version_id.clone()));
                    }
                }
            }
            planned.insert(version.project_id.clone(), version);
        }
        for version in planned.values() {
            for dependency in &version.dependencies {
                if dependency.dependency_type == "incompatible"
                    && planned.values().any(|candidate| {
                        candidate.project_id == dependency.project_id
                            && dependency
                                .version_id
                                .as_ref()
                                .is_none_or(|pin| *pin == candidate.id)
                    })
                {
                    return Err(invalid("Há mods incompatíveis na árvore de dependências."));
                }
            }
        }
        let directory = self.directory(id).await?;
        let download_root = directory.join(format!(".luxmc-download-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&download_root).await?;
        backups::normal_file(&download_root)?;
        let result = async {
        let download_client = pack_download::client()?;
        let mut downloaded = Vec::new(); let mut names = HashSet::new();
        for version in planned.values() {
            let file = version.files.first().ok_or_else(|| invalid("O provedor não retornou um arquivo para instalação."))?;
            pack_download::relative_path(&file.filename)?;
            if file.filename.contains(['/', '\\']) || !names.insert(file.filename.to_ascii_lowercase()) || file.size > 512 * 1024 * 1024 || file.sha1.len() != 40 || !file.sha1.bytes().all(|value| value.is_ascii_hexdigit()) { return Err(invalid("Arquivo sem hash válido, muito grande ou com nome duplicado.")); }
            let subdirectory = download_root.join(uuid::Uuid::new_v4().to_string()); tokio::fs::create_dir(&subdirectory).await?;
            let path = subdirectory.join(&file.filename);
            crate::core::mods::resumable::download(&download_client, &file.url, &path, Some(&self.shutting_down)).await?;
            if !pack_download::verify_existing(&path, Some(file.size), Some(&file.sha1), None, true).await { return Err(invalid("O conteúdo não passou na verificação de integridade. Nenhum arquivo do servidor foi substituído.")); }
            downloaded.push(path);
        }
        let mut installed = Vec::new();
        for path in downloaded {
            self.stage_content(id, kind, &path).await?;
            if kind == "mod" {
                let current = self.contents(id).await?;
                let incoming = path.clone(); let root = directory.clone();
                let old_files = current.iter().filter(|file| !file.staged && file.kind == "mod" && path.file_name().is_some_and(|name| name != file.name.as_str())).map(|file| file.name.clone()).collect::<Vec<_>>();
                let replacements = tokio::task::spawn_blocking(move || {
                    let Some(new_ids) = crate::core::mods::validation::primary_ids(&incoming) else { return Vec::new(); };
                    old_files.into_iter().filter(|name| crate::core::mods::validation::primary_ids(&root.join("mods").join(name)).is_some_and(|ids| !ids.is_disjoint(&new_ids))).collect::<Vec<_>>()
                }).await.map_err(|error| AppError::Internal(error.to_string()))?;
                for name in replacements { if !current.iter().any(|file| file.staged && file.kind == "mod" && file.name == name) { self.stage_remove(id, "mod", &name).await?; } }
            }
            installed.push(path.file_name().unwrap_or_default().to_string_lossy().into_owned());
            tokio::fs::remove_file(&path).await?; if let Some(parent) = path.parent() { tokio::fs::remove_dir(parent).await?; }
        }
        Ok(installed)
        }.await;
        let root = directory.clone();
        let temporary = download_root.clone();
        let _ = tokio::task::spawn_blocking(move || backups::remove_owned_tree(&root, &temporary))
            .await;
        result
    }
}
