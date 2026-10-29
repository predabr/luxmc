use super::*;

impl LocalServerManager {
    pub async fn use_resource_pack(&self,id:&str,file:String,required:bool)->AppResult<()> {
        let _guard=self.change()?;
        if self.current(id).is_ok() {return Err(unavailable("Pare e salve o servidor antes de trocar o pacote de texturas."));}
        let mut settings=self.settings(id).await?;
        if !file.is_empty() {
            if !self.contents(id).await?.iter().any(|item|item.kind=="resourcepack" && !item.staged && item.name==file) {return Err(invalid("Importe e aplique o ZIP antes de selecioná-lo."));}
            let path=self.directory(id).await?.join("resourcepacks").join(&file);
            if backups::normal_file(&path)?.len()>128*1024*1024 {return Err(invalid("O pacote automático deve ter até 128 MiB."));}
            tokio::task::spawn_blocking(move || content::validate_archive(&path,"resourcepack")).await.map_err(|error|AppError::Internal(error.to_string()))??;
        }
        settings.resource_pack_file=file;settings.resource_pack_url.clear();settings.resource_pack_sha1.clear();settings.require_resource_pack=required && !settings.resource_pack_file.is_empty();settings.validate()?;
        let directory=self.directory(id).await?;
        crate::core::mods::pack_download::atomic_write(&directory.join("luxmc-settings.json"),&serde_json::to_vec_pretty(&settings)?).await?;
        crate::network::resource_packs::clear_host(id).await;Ok(())
    }
    pub async fn delete_server(&self, id: &str, name: &str, delete_backups: bool) -> AppResult<()> {
        let _guard = self.change()?;
        if self.content_jobs.load(Ordering::Acquire) > 0 { return Err(unavailable("Aguarde os downloads e importações antes de excluir.")); }
        if self.current(id).is_ok() { return Err(unavailable("Pare e salve o servidor antes de excluí-lo.")); }
        if self.config(id).await?.name != name { return Err(invalid("Digite o nome exato do servidor para confirmar.")); }
        let root = self.root().await?;
        let directory = self.directory(id).await?;
        let backup_root = root.join("backups");
        let backup = backup_root.join(id);
        let target = directory.clone();
        let verified_root = root.clone();
        tokio::task::spawn_blocking(move || {
            if target.parent() != Some(verified_root.as_path()) { return Err(invalid("Pasta de servidor inválida.")); }
            backups::tree_files(&target, u64::MAX)?;
            if delete_backups && backup.exists() {
                backups::normal_file(&backup_root)?;
                backups::normal_file(&backup)?;
                if std::fs::canonicalize(&backup_root)?.parent() != Some(verified_root.as_path()) || std::fs::canonicalize(&backup)?.parent() != Some(std::fs::canonicalize(&backup_root)?.as_path()) { return Err(invalid("Pasta de backups inválida.")); }
                backups::tree_files(&backup, u64::MAX)?;
                std::fs::remove_dir_all(&backup)?;
            }
            std::fs::remove_dir_all(&target)?;
            Ok(())
        }).await.map_err(|error| AppError::Internal(error.to_string()))?
    }

    pub async fn archive_server(&self, id: &str, name: &str) -> AppResult<()> {
        let _guard = self.change()?;
        if self.current(id).is_ok() { return Err(unavailable("Pare e salve o servidor antes de excluí-lo.")); }
        if self.config(id).await?.name != name { return Err(invalid("Digite o nome exato do servidor para confirmar.")); }
        let root = self.root().await?;
        let source = self.directory(id).await?;
        if source.parent() != Some(root.as_path()) { return Err(invalid("Pasta de servidor inválida.")); }
        backups::normal_file(&source)?;
        let deleted = root.join(".luxmc-deleted");
        tokio::fs::create_dir_all(&deleted).await?;
        backups::normal_file(&deleted)?;
        let deleted = tokio::fs::canonicalize(deleted).await?;
        if deleted.parent() != Some(root.as_path()) { return Err(invalid("Pasta de recuperação inválida.")); }
        if deleted.join(id).exists() { return Err(unavailable("Já existe uma cópia deste servidor na recuperação.")); }
        tokio::fs::rename(source, deleted.join(id)).await?;
        Ok(())
    }

    pub async fn restore_server(&self, id: &str) -> AppResult<()> {
        let _guard = self.change()?;
        if id.len() != 36 || uuid::Uuid::parse_str(id).is_err() { return Err(invalid("Servidor inválido.")); }
        let root = self.root().await?;
        let deleted = root.join(".luxmc-deleted");
        backups::normal_file(&deleted)?;
        let source = deleted.join(id);
        backups::normal_file(&source)?;
        let source = tokio::fs::canonicalize(source).await?;
        if source.parent() != Some(tokio::fs::canonicalize(&deleted).await?.as_path()) || !source.starts_with(&root) { return Err(invalid("Pasta de recuperação inválida.")); }
        if root.join(id).exists() { return Err(unavailable("Este servidor já existe na lista.")); }
        tokio::fs::rename(source, root.join(id)).await?;
        Ok(())
    }

    pub async fn deleted_servers(&self) -> AppResult<Vec<LocalServerConfig>> {
        let root = self.root().await?; let deleted = root.join(".luxmc-deleted");
        if !deleted.exists() { return Ok(Vec::new()); }
        backups::normal_file(&deleted)?;
        if tokio::fs::canonicalize(&deleted).await?.parent() != Some(root.as_path()) { return Err(invalid("Pasta de recuperação inválida.")); }
        let mut entries = tokio::fs::read_dir(deleted).await?; let mut servers = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let id = entry.file_name().to_string_lossy().into_owned();
            if uuid::Uuid::parse_str(&id).is_err() { continue; }
            backups::normal_file(&entry.path())?;
            if let Some(bytes) = self.read(&entry.path(), "luxmc-server.json").await? {
                let config: LocalServerConfig = serde_json::from_slice(&bytes)?;
                validate_config(&config)?;
                if config.id != id { return Err(invalid("Configuração de recuperação inválida.")); }
                servers.push(config);
            }
            if servers.len() >= 64 { break; }
        }
        Ok(servers)
    }

    pub async fn instance_report(&self, id: &str) -> AppResult<serde_json::Value> {
        let directory = self.directory(id).await?;
        let path = directory.join(".luxmc-instance-report.json");
        if !path.exists() {
            let catalog=directory.join(".luxmc-catalog.json");
            if catalog.exists() && backups::normal_file(&catalog)?.len()<=1024*1024 {
                let value:serde_json::Value=serde_json::from_slice(&tokio::fs::read(catalog).await?)?;
                return Ok(serde_json::json!({"prepared":value["report"]["prepared"].as_u64().unwrap_or(0),"excluded":value["report"]["excludedFiles"].as_array().cloned().unwrap_or_default(),"localCopy":false,"catalog":true,"resourcePackWarning":value["report"]["resourcePackWarning"]}));
            }
            return Ok(serde_json::json!({"prepared":0,"excluded":[],"localCopy":false}));
        }
        if backups::normal_file(&path)?.len() > 1024 * 1024 { return Err(invalid("Relatório de importação muito grande.")); }
        Ok(serde_json::from_slice(&tokio::fs::read(path).await?)?)
    }

    pub async fn browse_files(&self, id: &str, path: &str) -> AppResult<serde_json::Value> {
        let root = self.directory(id).await?;
        let relative = if path.is_empty() { PathBuf::new() } else { crate::core::mods::pack_download::relative_path(path)? };
        if relative.components().any(|part| part.as_os_str().to_string_lossy().starts_with(".luxmc-")) { return Err(invalid("Pasta interna do launcher.")); }
        let directory = root.join(relative);
        backups::normal_file(&directory)?;
        let directory = tokio::fs::canonicalize(directory).await?;
        if !directory.starts_with(&root) { return Err(invalid("Pasta fora do servidor.")); }
        let mut entries = tokio::fs::read_dir(directory).await?; let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with(".luxmc-") || name == "session.lock" { continue; }
            let metadata = backups::normal_file(&entry.path())?;
            files.push(serde_json::json!({"name":name,"directory":metadata.is_dir(),"bytes":metadata.len()}));
            if files.len() > 2048 { return Err(invalid("Pasta com mais de 2048 itens. Abra no gerenciador de arquivos.")); }
        }
        files.sort_by(|a,b| b["directory"].as_bool().cmp(&a["directory"].as_bool()).then_with(|| a["name"].as_str().cmp(&b["name"].as_str())));
        Ok(serde_json::json!({"path":path,"entries":files}))
    }

    pub async fn player_details(&self, id: &str) -> AppResult<serde_json::Value> {
        let directory = self.directory(id).await?;
        let current = {
            self.runtime
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref()
                .filter(|runtime| runtime.snapshot(u64::MAX).id == id)
                .cloned()
        };
        let players = if let Some(runtime) = current {
            runtime.snapshot(u64::MAX).players
        } else {
            let path = directory.join("luxmc-players.json");
            if path.exists() {
                let metadata = backups::normal_file(&path)?;
                if metadata.len() > 1024 * 1024 {
                    return Err(invalid("Histórico muito grande."));
                }
                let mut history: players::PlayerHistory =
                    serde_json::from_slice(&tokio::fs::read(path).await?)?;
                history.offline();
                history.records.into_values().collect()
            } else {
                Vec::new()
            }
        };
        let mut result = serde_json::json!({"players":players});
        for (name, filename) in [
            ("banned", "banned-players.json"),
            ("whitelist", "whitelist.json"),
            ("operators", "ops.json"),
        ] {
            let entries: serde_json::Value = self
                .read(&directory, filename)
                .await?
                .map(|bytes| serde_json::from_slice(&bytes))
                .transpose()?
                .unwrap_or_else(|| serde_json::json!([]));
            if !entries.is_array() {
                return Err(invalid("Lista de acesso inválida; arquivo preservado."));
            }
            result[name] = entries;
        }
        Ok(result)
    }

    pub async fn set_icon(&self, id: &str, path: PathBuf) -> AppResult<()> {
        let _guard = self.change()?;
        if self.current(id).is_ok() {
            return Err(unavailable("Pare o servidor para alterar o ícone."));
        }
        let directory = self.directory(id).await?;
        let metadata = backups::normal_file(&path)?;
        if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
            return Err(invalid("Escolha uma imagem de até 16 MiB."));
        }
        let png = tokio::task::spawn_blocking(move || -> AppResult<Vec<u8>> {
            let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
            let mut limits = image::Limits::default();
            limits.max_image_width = Some(8192);
            limits.max_image_height = Some(8192);
            limits.max_alloc = Some(128 * 1024 * 1024);
            reader.limits(limits);
            let image = reader
                .decode()
                .map_err(|error| invalid(format!("Imagem inválida: {error}")))?
                .resize_to_fill(64, 64, image::imageops::FilterType::Triangle);
            let mut bytes = std::io::Cursor::new(Vec::new());
            image
                .write_to(&mut bytes, image::ImageFormat::Png)
                .map_err(|error| invalid(error.to_string()))?;
            Ok(bytes.into_inner())
        })
        .await
        .map_err(|error| AppError::Internal(error.to_string()))??;
        if let Some(previous) = self.read(&directory, "server-icon.png").await? {
            crate::core::mods::pack_download::atomic_write(
                &directory.join("server-icon.previous.png"),
                &previous,
            )
            .await?;
        }
        crate::core::mods::pack_download::atomic_write(&directory.join("server-icon.png"), &png)
            .await?;
        Ok(())
    }

    pub async fn import_world(&self, id: &str, source: PathBuf) -> AppResult<()> {
        let _guard = self.change()?;
        if self.current(id).is_ok() || crate::core::launcher::get_active_game_pid() != 0 {
            return Err(unavailable("Feche o Minecraft e pare o servidor para copiar um mundo consistente. O original será preservado."));
        }
        let source = tokio::fs::canonicalize(source).await?;
        let session = source.join("session.lock");
        let _source_lock = if session.exists() {
            backups::normal_file(&session)?;
            let file = std::fs::OpenOptions::new().read(true).write(true).open(&session)?;
            file.try_lock().map_err(|_| unavailable("Feche o Minecraft que está usando este mundo antes de importá-lo."))?;
            Some(file)
        } else { None };
        backups::normal_file(&source)?;
        if !source.join("level.dat").is_file() {
            return Err(invalid("Escolha a pasta do mundo que contém level.dat."));
        }
        let directory = self.directory(id).await?;
        let expected_version = self.config(id).await?.mc_version;
        if source.starts_with(&directory) || directory.starts_with(&source) {
            return Err(invalid("Escolha um mundo fora da pasta deste servidor."));
        }
        let staging = directory.join(format!(".luxmc-world-import-{}", uuid::Uuid::new_v4()));
        let target = staging.clone();
        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let mut bytes = Vec::new();
            use std::io::Read;
            flate2::read::GzDecoder::new(std::fs::File::open(source.join("level.dat"))?).take(8 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > 8 * 1024 * 1024 { return Err(invalid("Metadados do mundo muito grandes.")); }
            let nbt: std::collections::HashMap<String, fastnbt::Value> = fastnbt::from_bytes(&bytes).map_err(|_| invalid("O arquivo level.dat está corrompido ou usa um formato não suportado."))?;
            if !matches!(nbt.get("Data"), Some(fastnbt::Value::Compound(_))) { return Err(invalid("O arquivo não contém os dados de um mundo Minecraft válido.")); }
            if let Some(fastnbt::Value::Compound(data)) = nbt.get("Data") { if let Some(fastnbt::Value::Compound(version)) = data.get("Version") { if let Some(fastnbt::Value::String(name)) = version.get("Name") { if name != &expected_version { return Err(invalid(format!("Este mundo foi salvo no Minecraft {name}. Crie um servidor nessa versão para preservar o original."))); } } } }
            let files = backups::tree_files(&source, 8 * 1024 * 1024 * 1024)?;
            std::fs::create_dir(&target)?;
            for (relative, _) in files { if relative == Path::new("session.lock") { continue; } let destination = target.join(&relative); std::fs::create_dir_all(destination.parent().ok_or_else(|| invalid("Caminho inválido."))?)?; std::fs::copy(source.join(relative), destination)?; }
            Ok(())
        }).await.map_err(|error| AppError::Internal(error.to_string()))??;
        let world = directory.join("world");
        let previous = directory.join(format!(".luxmc-world-before-{}", uuid::Uuid::new_v4()));
        if world.exists() {
            backups::normal_file(&world)?;
            tokio::fs::rename(&world, &previous).await?;
        }
        if let Err(error) = tokio::fs::rename(&staging, &world).await {
            if previous.exists() {
                let _ = tokio::fs::rename(&previous, &world).await;
            }
            return Err(error.into());
        }
        if previous.exists() {
            tokio::task::spawn_blocking(move || backups::remove_owned_tree(&directory, &previous))
                .await
                .map_err(|error| AppError::Internal(error.to_string()))??;
        }
        Ok(())
    }
}
