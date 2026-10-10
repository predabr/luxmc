use std::path::Path;
use crate::error::{AppError, AppResult};
use crate::core::mods::pack_download::{atomic_write, destination};

const NAME: &str = "LuxmcEssentials";
const SPLASHES: &str = "Seu mundo. Sua turma.\nFeito para jogar juntos!\nUma aventura de cada vez.\nLuxmc: pronto para explorar.\n";
const ICON: &[u8] = include_bytes!("../../icons/128x128.png");

async fn write_changed(path: &Path, bytes: &[u8]) -> AppResult<()> {
    if tokio::fs::metadata(path).await.is_ok_and(|meta| meta.len() == bytes.len() as u64) && tokio::fs::read(path).await.is_ok_and(|old| old == bytes) { return Ok(()); }
    atomic_write(path,bytes).await
}

fn enable_java(existing: &str, legacy: bool) -> AppResult<String> {
    let tag=if legacy {NAME.to_owned()} else {format!("file/{NAME}")};
    let mut lines=existing.lines().map(str::to_owned).collect::<Vec<_>>();
    if let Some(line)=lines.iter_mut().find(|line|line.starts_with("resourcePacks:")) {
        let mut packs:Vec<String>=serde_json::from_str(line.trim_start_matches("resourcePacks:")).map_err(|_|AppError::InvalidInput("A lista de pacotes em options.txt está inválida; ela foi preservada.".into()))?;
        if !packs.contains(&tag) {packs.insert(0,tag);}
        *line=format!("resourcePacks:{}",serde_json::to_string(&packs)?);
    } else {
        let mut packs=if legacy {Vec::new()} else {vec!["vanilla".to_owned()]};
        packs.push(tag);
        lines.push(format!("resourcePacks:{}",serde_json::to_string(&packs)?));
    }
    Ok(lines.join("\n")+"\n")
}

pub async fn prepare_java(root:&Path,version:&str) -> AppResult<bool> {
    let clean=crate::core::launcher::extract_mc_version(version);
    if clean.starts_with(['a','b','c']) || clean.starts_with("rd-") || clean.strip_prefix("1.").and_then(|value|value.split('.').next()).and_then(|value|value.parse::<u32>().ok()).is_some_and(|minor|minor<6) {return Ok(false);}
    prepare_java_format(root,crate::core::launcher::get_pack_format_for_version(version)).await
}

async fn prepare_java_format(root:&Path,format:u32) -> AppResult<bool> {
    let pack=destination(root,&format!("resourcepacks/{NAME}"))?;
    let marker=destination(root,&format!("resourcepacks/{NAME}/.luxmc-owned"))?;
    if tokio::fs::try_exists(&pack).await? && !tokio::fs::try_exists(&marker).await? {return Ok(false);}
    let mut metadata=crate::core::launcher::skin_pack_metadata(format);
    metadata["pack"]["description"]=serde_json::json!("Luxmc Essentials · Identidade, menu e boas aventuras");
    for (relative,bytes) in [("pack.mcmeta",serde_json::to_vec_pretty(&metadata)?),("pack.png",ICON.to_vec()),("assets/minecraft/texts/splashes.txt",SPLASHES.as_bytes().to_vec()),(".luxmc-owned",b"1\n".to_vec())] {
        let path=destination(root,&format!("resourcepacks/{NAME}/{relative}"))?;
        write_changed(&path,&bytes).await?;
    }
    let activated=destination(root,".luxmc/essentials-installed")?;
    if !tokio::fs::try_exists(&activated).await? {
        let options=destination(root,"options.txt")?;
        let existing=match tokio::fs::read_to_string(&options).await {Ok(value)=>value,Err(error) if error.kind()==std::io::ErrorKind::NotFound=>String::new(),Err(error)=>return Err(error.into())};
        write_changed(&options,enable_java(&existing,format<=3)?.as_bytes()).await?;
        write_changed(&activated,b"1\n").await?;
    }
    Ok(true)
}

pub async fn prepare_bedrock(root:&Path) -> AppResult<()> {
    let base=destination(root,&format!("resource_packs/{NAME}"))?;
    if tokio::fs::try_exists(&base).await? && !tokio::fs::try_exists(base.join(".luxmc-owned")).await? {return Ok(());}
    let metadata=serde_json::json!({"format_version":2,"header":{"name":"Luxmc Essentials","description":"Seu mundo. Sua turma. Identidade Luxmc no menu.","uuid":"f229d367-dc89-4141-8a2f-761f7727c050","version":[1,0,0],"min_engine_version":[1,16,0]},"modules":[{"type":"resources","uuid":"6d189428-09be-4ab1-ac89-f6a3e3130f74","version":[1,0,0]}],"metadata":{"authors":["Luxmc"]}});
    let splashes=serde_json::json!({"splashes":SPLASHES.lines().collect::<Vec<_>>()});
    for (relative,bytes) in [("manifest.json",serde_json::to_vec_pretty(&metadata)?),("pack_icon.png",ICON.to_vec()),("splashes.json",serde_json::to_vec_pretty(&splashes)?),(".luxmc-owned",b"1\n".to_vec())] {
        write_changed(&destination(root,&format!("resource_packs/{NAME}/{relative}"))?,&bytes).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn preserves_packs_options_worlds_and_user_disabled_selection() {
        let root=tempfile::tempdir().unwrap();
        tokio::fs::write(root.path().join("options.txt"),"lang:pt_br\nresourcePacks:[\"vanilla\",\"file/MyPack.zip\"]\n").await.unwrap();
        tokio::fs::create_dir_all(root.path().join("saves/Keep")).await.unwrap();
        tokio::fs::write(root.path().join("saves/Keep/level.dat"),b"world").await.unwrap();
        prepare_java_format(root.path(),34).await.unwrap();
        let options=tokio::fs::read_to_string(root.path().join("options.txt")).await.unwrap();
        assert!(options.contains("lang:pt_br"));assert!(options.contains("file/MyPack.zip"));assert!(options.contains("file/LuxmcEssentials"));
        tokio::fs::write(root.path().join("options.txt"),"resourcePacks:[\"vanilla\"]\n").await.unwrap();
        prepare_java_format(root.path(),65).await.unwrap();
        assert_eq!(tokio::fs::read_to_string(root.path().join("options.txt")).await.unwrap(),"resourcePacks:[\"vanilla\"]\n");
        assert_eq!(tokio::fs::read(root.path().join("saves/Keep/level.dat")).await.unwrap(),b"world");
        let metadata:serde_json::Value=serde_json::from_slice(&tokio::fs::read(root.path().join("resourcepacks/LuxmcEssentials/pack.mcmeta")).await.unwrap()).unwrap();
        assert_eq!(metadata["pack"]["min_format"],65);
    }
    #[tokio::test]
    async fn protects_unowned_packs_invalid_options_and_unsupported_legacy() {
        let root=tempfile::tempdir().unwrap();
        tokio::fs::create_dir_all(root.path().join("resourcepacks/LuxmcEssentials")).await.unwrap();
        tokio::fs::write(root.path().join("resourcepacks/LuxmcEssentials/keep.txt"),b"personal").await.unwrap();
        assert!(!prepare_java_format(root.path(),34).await.unwrap());
        assert_eq!(tokio::fs::read(root.path().join("resourcepacks/LuxmcEssentials/keep.txt")).await.unwrap(),b"personal");
        let broken=tempfile::tempdir().unwrap();
        tokio::fs::write(broken.path().join("options.txt"),"resourcePacks:broken\n").await.unwrap();
        assert!(prepare_java_format(broken.path(),34).await.is_err());
        assert_eq!(tokio::fs::read_to_string(broken.path().join("options.txt")).await.unwrap(),"resourcePacks:broken\n");
        assert!(!prepare_java(broken.path(),"1.5.2").await.unwrap());
    }
    #[tokio::test]
    async fn bedrock_uses_distinct_manifest_and_does_not_change_world_pack_lists() {
        let root=tempfile::tempdir().unwrap();
        tokio::fs::write(root.path().join("global_resource_packs.json"),b"[]").await.unwrap();
        prepare_bedrock(root.path()).await.unwrap();
        let value:serde_json::Value=serde_json::from_slice(&tokio::fs::read(root.path().join("resource_packs/LuxmcEssentials/manifest.json")).await.unwrap()).unwrap();
        assert_eq!(value["format_version"],2);assert_ne!(value["header"]["uuid"],value["modules"][0]["uuid"]);
        assert_eq!(tokio::fs::read(root.path().join("global_resource_packs.json")).await.unwrap(),b"[]");
    }
}
