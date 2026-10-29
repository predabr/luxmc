use super::*;
use tokio::io::{AsyncReadExt,AsyncSeekExt};

impl LocalServerManager {
    pub async fn connection_diagnostic(&self,id:&str)->AppResult<serde_json::Value> {
        self.config(id).await?;
        let runtime=self.runtime.lock().unwrap_or_else(|error|error.into_inner()).as_ref().filter(|runtime|runtime.snapshot(u64::MAX).id==id).cloned();
        let Some(runtime)=runtime else{return Ok(serde_json::json!({"minecraftResponding":false,"address":null,"phase":"stopped","tcpReachable":false}));};
        let state=runtime.snapshot(u64::MAX);let Some(port)=state.port else{return Ok(serde_json::json!({"minecraftResponding":false,"address":null,"phase":state.phase,"tcpReachable":false}));};
        let status=tokio::time::timeout(Duration::from_secs(2),crate::core::server::ping_async(&self.http,"127.0.0.1",port)).await;
        let responding=status.as_ref().is_ok_and(|result|result.as_ref().is_ok_and(|status|status.online));
        let tcp=responding || tokio::time::timeout(Duration::from_millis(300),tokio::net::TcpStream::connect(("127.0.0.1",port))).await.is_ok_and(|result|result.is_ok());
        Ok(serde_json::json!({"minecraftResponding":responding,"address":responding.then(||format!("127.0.0.1:{port}")),"phase":state.phase,"tcpReachable":tcp,"port":port,"publicAddress":null}))
    }

    pub async fn console_text(&self,id:&str)->AppResult<serde_json::Value> {
        let directory=self.directory(id).await?;
        let lines=self.runtime.lock().unwrap_or_else(|error|error.into_inner()).as_ref().filter(|runtime|runtime.snapshot(u64::MAX).id==id).map(|runtime|runtime.snapshot(0).console).unwrap_or_default();
        let path=directory.join("logs/latest.log");let mut text=String::new();let mut truncated=false;
        if path.exists() {
            backups::normal_file(&directory.join("logs"))?;let metadata=backups::normal_file(&path)?;
            if !metadata.is_file() || !tokio::fs::canonicalize(&path).await?.starts_with(&directory){return Err(invalid("Arquivo de console fora do servidor."));}
            let mut file=tokio::fs::File::open(path).await?;let limit=2*1024*1024;
            if metadata.len()>limit{truncated=true;file.seek(std::io::SeekFrom::End(-(limit as i64))).await?;}
            let mut bytes=Vec::new();file.take(limit).read_to_end(&mut bytes).await?;
            text=String::from_utf8_lossy(&bytes).lines().map(crate::core::process::sanitize_diagnostic).collect::<Vec<_>>().join("\n");
        }
        let extra=lines.iter().filter(|line|text.is_empty() || line.stream!="stdout").map(|line|format!("[{}] {}",line.stream,line.text)).collect::<Vec<_>>().join("\n");
        if !extra.is_empty(){if !text.is_empty(){text.push_str("\n\n--- Luxmc ---\n");}text.push_str(&extra);}
        Ok(serde_json::json!({"text":text,"truncated":truncated}))
    }

    pub async fn deletion_preview(&self,id:&str)->AppResult<serde_json::Value> {
        let directory=self.directory(id).await?;let root=self.root().await?;let id=id.to_string();
        tokio::task::spawn_blocking(move ||->AppResult<_>{
            let mut files=backups::tree_files(&directory,u64::MAX)?;
            let backup=root.join("backups").join(id);
            if backup.exists(){backups::normal_file(&root.join("backups"))?;files.extend(backups::tree_files(&backup,u64::MAX)?);}
            Ok(serde_json::json!({"files":files.len(),"bytes":files.iter().map(|(_,bytes)|*bytes).sum::<u64>()}))
        }).await.map_err(|error|AppError::Internal(error.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_and_saving_require_trusted_server_messages() {
        let (runtime,_)=super::super::tests::fixture_runtime("fixture".into());
        runtime.log("stderr","Done (1s)!");
        runtime.log("stdout","[Server thread/INFO]: <Player> Done (1s)!");
        assert!(!runtime.ready_log.load(Ordering::Acquire));
        runtime.log("stdout","[Server thread/INFO]: Done (1s)! For help, type help");
        assert!(runtime.ready_log.load(Ordering::Acquire));
        runtime.log("stdout","[Server thread/INFO]: <Player> All dimensions are saved");
        assert!(runtime.snapshot(0).last_saved_at.is_none());
        runtime.log("stdout","[Server thread/INFO]: ThreadedAnvilChunkStorage: All dimensions are saved");
        assert!(runtime.snapshot(0).last_saved_at.is_some());
        let (forge,_)=super::super::tests::fixture_runtime("forge".into());
        forge.log("stdout","[21:00:00] [Server thread/INFO] [minecraft/MinecraftServer]: ThreadedAnvilChunkStorage: All dimensions are saved");
        assert!(forge.snapshot(0).last_saved_at.is_some());
    }

    #[tokio::test]
    async fn complete_deletion_preview_includes_owned_backups_and_preserves_other_servers() {
        let temporary=tempfile::tempdir().unwrap();
        let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
        let config=super::super::tests::fixture_config();let other=super::super::tests::fixture_config();
        let directory=super::super::tests::persist(&manager,&config).await;let preserved=super::super::tests::persist(&manager,&other).await;
        for name in ["world/region/world.mca","mods/mod.jar","libraries/a/a.jar"] {let path=directory.join(name);tokio::fs::create_dir_all(path.parent().unwrap()).await.unwrap();tokio::fs::write(path,b"owned").await.unwrap();}
        let backup=directory.parent().unwrap().join("backups").join(&config.id);tokio::fs::create_dir_all(&backup).await.unwrap();tokio::fs::write(backup.join("backup.zip"),b"backup").await.unwrap();
        let preview=manager.deletion_preview(&config.id).await.unwrap();assert_eq!(preview["files"],5);assert!(preview["bytes"].as_u64().unwrap()>21);
        manager.delete_server(&config.id,&config.name,true).await.unwrap();assert!(!directory.exists());assert!(!backup.exists());assert!(preserved.join("luxmc-server.json").exists());
    }
}
