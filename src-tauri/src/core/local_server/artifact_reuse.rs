use super::*;

impl LocalServerManager {
    pub(super) async fn reuse_server_artifact(&self,config:&LocalServerConfig,target:&Path)->AppResult<bool> {
        if target.exists(){return Ok(false);}
        let root=self.root().await?;let mut entries=tokio::fs::read_dir(root).await?;let mut checked=0;
        while let Some(entry)=entries.next_entry().await? {
            let id=entry.file_name().to_string_lossy().into_owned();
            if id==config.id || uuid::Uuid::parse_str(&id).is_err(){continue;}
            checked+=1;if checked>32{break;}
            let Ok(other)=self.config(&id).await else{continue};
            if other.artifact.sha1!=config.artifact.sha1 || other.artifact.size!=config.artifact.size{continue;}
            let Ok(source_directory)=self.directory(&id).await else{continue};
            let source=source_directory.join("server.jar");
            if backups::normal_file(&source).is_err(){continue;}
            if !crate::core::mods::pack_download::verify_existing(&source,Some(config.artifact.size),Some(&config.artifact.sha1),None,true).await {continue;}
            let temporary=target.with_file_name(format!(".luxmc-server-copy-{}",uuid::Uuid::new_v4()));
            let result=async {tokio::fs::copy(source,&temporary).await?;tokio::fs::rename(&temporary,target).await?;Ok::<_,AppError>(())}.await;
            if result.is_err() {let _=tokio::fs::remove_file(temporary).await;return Ok(false);}
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use sha1::Digest;

    #[tokio::test]
    async fn a_verified_existing_server_jar_is_reused_without_a_download_or_shared_leftovers() {
        let temporary=tempfile::tempdir().unwrap();let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
        let mut archive=zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));archive.start_file("META-INF/MANIFEST.MF",zip::write::FileOptions::default()).unwrap();archive.write_all(b"Manifest-Version: 1.0\n").unwrap();let bytes=archive.finish().unwrap().into_inner();
        let mut original=super::super::tests::fixture_config();original.artifact.sha1=format!("{:x}",sha1::Sha1::digest(&bytes));original.artifact.size=bytes.len() as u64;
        let source=super::super::tests::persist(&manager,&original).await;tokio::fs::write(source.join("server.jar"),&bytes).await.unwrap();
        let mut next=original.clone();next.id=uuid::Uuid::new_v4().to_string();let destination=super::super::tests::persist(&manager,&next).await;let target=destination.join("server.jar");
        assert!(manager.reuse_server_artifact(&next,&target).await.unwrap());assert_eq!(tokio::fs::read(&target).await.unwrap(),bytes);assert!(!manager.reuse_server_artifact(&next,&target).await.unwrap());
        manager.delete_server(&next.id,&next.name,true).await.unwrap();assert!(source.join("server.jar").exists());
    }
}
