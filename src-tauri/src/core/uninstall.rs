use std::{fs, io, path::{Path, PathBuf}};

fn checked_targets(roaming: &Path, local: &Path, documents: &Path) -> io::Result<Vec<PathBuf>> {
    let documents = documents.canonicalize()?;
    let mut targets = Vec::new();
    for (base, relative) in [(roaming,"github/Luxmc"),(roaming,"io.github.luxmc.Luxmc"),(local,"github/Luxmc"),(local,"io.github.luxmc.Luxmc"),(local,"LuxmcRecovery")] {
        let target = base.join(relative);
        if !target.exists() { continue; }
        let canonical_base = base.canonicalize()?;
        let canonical = target.canonicalize()?;
        if canonical == canonical_base || !canonical.starts_with(&canonical_base) || canonical.starts_with(&documents) || documents.starts_with(&canonical) {
            return Err(io::Error::other("Pasta de limpeza inválida. Documentos e projeto foram preservados."));
        }
        targets.push(canonical);
    }
    Ok(targets)
}

pub fn remove_data(roaming: &Path, local: &Path, documents: &Path) -> io::Result<usize> {
    let targets = checked_targets(roaming, local, documents)?;
    for target in &targets { fs::remove_dir_all(target)?; }
    Ok(targets.len())
}

#[cfg(any(windows, test))]
fn managed_packages(value: &serde_json::Value) -> io::Result<Vec<(String, String)>> {
    let mut packages = Vec::new();
    for item in value.get("managed").and_then(serde_json::Value::as_array).into_iter().flatten() {
        let family = item.get("app_id").and_then(serde_json::Value::as_str).unwrap_or_default().split('_').next().unwrap_or_default();
        let version = item.get("package_version").and_then(serde_json::Value::as_str).unwrap_or_default();
        if !matches!(family, "Microsoft.MinecraftUWP" | "Microsoft.MinecraftWindowsBeta") || version.is_empty() || version.len() > 32 || !version.bytes().all(|byte| byte.is_ascii_digit() || byte == b'.') {
            return Err(io::Error::other("Identidade Bedrock inválida; limpeza interrompida."));
        }
        let package = (family.to_owned(), version.to_owned());
        if !packages.contains(&package) { packages.push(package); }
    }
    Ok(packages)
}

#[cfg(windows)]
fn uninstall_managed_bedrock(targets: &[PathBuf]) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    let mut packages = Vec::new();
    for target in targets {
        let state = target.join("data/bedrock-provider.json");
        if !state.exists() { continue; }
        let canonical = state.canonicalize()?;
        if !canonical.starts_with(target) || fs::metadata(&canonical)?.len() > 4 * 1024 * 1024 { return Err(io::Error::other("Configuração Bedrock inválida")); }
        let value = serde_json::from_slice(&fs::read(canonical)?)?;
        packages.extend(managed_packages(&value)?);
    }
    if packages.is_empty() { return Ok(()); }
    let packages: Vec<_> = packages.into_iter().map(|(family, version)| serde_json::json!({ "family": family, "version": version })).collect();
    let shell = PathBuf::from(std::env::var_os("SystemRoot").ok_or_else(|| io::Error::other("Windows indisponível"))?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = std::process::Command::new(shell)
        .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference='Stop'; if(Get-Process -Name Minecraft.Windows,Minecraft.WindowsBeta,gamingservicesui -ErrorAction SilentlyContinue | Where-Object {$_.MainWindowTitle -like 'Minecraft*'}){throw 'Feche o Minecraft Bedrock antes de remover os dados.'}; foreach($item in (ConvertFrom-Json $env:LUXMC_OWNED_BEDROCK)){ $packages=@(Get-AppxPackage -Name $item.family | Where-Object {$_.Version.ToString() -eq $item.version}); foreach($package in $packages){Remove-AppxPackage -Package $package.PackageFullName -ErrorAction Stop}; if(Get-AppxPackage -Name $item.family | Where-Object {$_.Version.ToString() -eq $item.version}){throw 'Minecraft ainda instalado.'} }"])
        .env("LUXMC_OWNED_BEDROCK", serde_json::to_string(&packages)?)
        .creation_flags(0x08000000).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
    loop {
        if let Some(status) = child.try_wait()? { return if status.success() { Ok(()) } else { Err(io::Error::other("O Windows não concluiu a remoção do Bedrock; feche o jogo e tente novamente.")) }; }
        if std::time::Instant::now() >= deadline { let _ = child.kill(); let _ = child.wait(); return Err(io::Error::other("Tempo esgotado ao remover o Bedrock")); }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[cfg(windows)]
fn known_folder(data1: u32, data2: u16, data3: u16, data4: [u8;8]) -> io::Result<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    #[repr(C)] struct Guid { data1:u32, data2:u16, data3:u16, data4:[u8;8] }
    #[link(name="shell32")] unsafe extern "system" { fn SHGetKnownFolderPath(id:*const Guid, flags:u32, token:*mut std::ffi::c_void, path:*mut *mut u16) -> i32; }
    #[link(name="ole32")] unsafe extern "system" { fn CoTaskMemFree(value:*mut std::ffi::c_void); }
    let mut pointer=std::ptr::null_mut();
    let id=Guid{data1,data2,data3,data4};
    if unsafe { SHGetKnownFolderPath(&id,0,std::ptr::null_mut(),&mut pointer) } < 0 || pointer.is_null() { return Err(io::Error::other("Pasta do Windows indisponível")); }
    let mut length=0;
    unsafe { while length<32768 && *pointer.add(length)!=0 {length+=1;} }
    let path=unsafe { std::ffi::OsString::from_wide(std::slice::from_raw_parts(pointer,length)) };
    unsafe { CoTaskMemFree(pointer.cast()); }
    Ok(PathBuf::from(path))
}

pub fn uninstall_data() -> io::Result<usize> {
    #[cfg(windows)] {
        let roaming=known_folder(0x3eb685db,0x65f9,0x4cf6,[0xa0,0x3a,0xe3,0xef,0x65,0x72,0x9f,0x3d])?;
        let local=known_folder(0xf1b32785,0x6fba,0x4fcf,[0x9d,0x55,0x7b,0x8e,0x7f,0x15,0x70,0x91])?;
        let documents=known_folder(0xfdd39ad0,0x238f,0x46af,[0xad,0xb4,0x6c,0x85,0x48,0x03,0x69,0xc7])?;
        let targets = checked_targets(&roaming, &local, &documents)?;
        uninstall_managed_bedrock(&targets)?;
        remove_data(&roaming,&local,&documents)
    }
    #[cfg(not(windows))] { Err(io::Error::other("Limpeza do desinstalador disponível no Windows")) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_removes_only_known_data_and_preserves_documents_and_other_apps() {
        let root=std::env::temp_dir().join(format!("luxmc-cleanup-test-{}",std::process::id()));
        let roaming=root.join("roaming");let local=root.join("local");let documents=root.join("documents");
        for directory in [roaming.join("github/Luxmc/data/instances/test"),local.join("LuxmcRecovery"),local.join("OtherApplication"),documents.join("Luxmc/src")] {fs::create_dir_all(directory).unwrap();}
        fs::write(documents.join("Luxmc/src/project.txt"),"keep").unwrap();
        assert_eq!(remove_data(&roaming,&local,&documents).unwrap(),2);
        assert!(!roaming.join("github/Luxmc").exists());
        assert!(local.join("OtherApplication").exists());
        assert_eq!(fs::read_to_string(documents.join("Luxmc/src/project.txt")).unwrap(),"keep");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cleanup_rejects_targets_containing_documents_before_any_removal() {
        let root = std::env::temp_dir().join(format!("luxmc-cleanup-protected-{}", std::process::id()));
        let roaming = root.join("roaming"); let local = root.join("local"); let documents = roaming.join("github/Luxmc/Documents");
        fs::create_dir_all(&documents).unwrap(); fs::create_dir_all(local.join("LuxmcRecovery")).unwrap();
        fs::write(documents.join("project.txt"), "keep").unwrap();
        assert!(remove_data(&roaming, &local, &documents).is_err());
        assert!(local.join("LuxmcRecovery").exists()); assert!(documents.join("project.txt").exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn bedrock_cleanup_accepts_only_exact_minecraft_package_versions() {
        let value = serde_json::json!({"managed":[{"app_id":"Microsoft.MinecraftUWP_8wekyb3d8bbwe!App", "package_version":"1.20.8101.0"}]});
        assert_eq!(managed_packages(&value).unwrap(), vec![("Microsoft.MinecraftUWP".into(), "1.20.8101.0".into())]);
        for family in ["Microsoft.OtherApp", "Microsoft.MinecraftUWP;calc"] {
            assert!(managed_packages(&serde_json::json!({"managed":[{"app_id":family,"package_version":"1.0"}]})).is_err());
        }
    }
}
