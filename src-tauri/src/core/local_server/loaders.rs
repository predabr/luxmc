use super::*;
use sha2::Digest;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerLoader {
    pub kind: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn official_installer_checksums_require_matching_bytes_for_both_digest_formats() {
        assert!(valid_installer_hash(b"abc","a9993e364706816aba3e25717850c26c9cd0d89d"));
        assert!(valid_installer_hash(b"abc","ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert!(!valid_installer_hash(b"changed","ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
        assert!(!valid_installer_hash(b"abc","invalid"));
    }

    #[test]
    fn java_receives_a_relative_installer_jar_for_every_loader_on_windows_paths() {
        let command=installer_command(Path::new("java"),Path::new(r"\\?\C:\Server Data"),&["--installServer".into()]);
        let args=command.as_std().get_args().map(|value|value.to_string_lossy().into_owned()).collect::<Vec<_>>();
        let index=args.iter().position(|value|value=="-jar").unwrap();
        assert_eq!(args[index+1],"luxmc-loader-installer.jar");
        #[cfg(windows)]
        assert_eq!(command.as_std().get_current_dir().unwrap(),Path::new(r"C:\Server Data"));
        assert!(args.iter().all(|value|!value.contains(r"\\?\")));
    }

    #[tokio::test]
    #[ignore = "Downloads and runs an official loader installer in a temporary directory; never starts Minecraft"]
    async fn official_server_loader_installs_from_canonical_directory_without_starting_minecraft() {
        let kind=std::env::var("LUXMC_LOADER_TEST_KIND").unwrap();
        let version=std::env::var("LUXMC_LOADER_TEST_VERSION").unwrap();
        let java=PathBuf::from(std::env::var("LUXMC_LOADER_TEST_JAVA").unwrap());
        let temporary=tempfile::tempdir().unwrap();
        let manager=LocalServerManager::new(temporary.path().join("servers"),reqwest::Client::new());
        let mut config=super::super::tests::fixture_config();config.mc_version=std::env::var("LUXMC_LOADER_TEST_MC").unwrap();
        let directory=super::super::tests::persist(&manager,&config).await;
        tokio::fs::write(directory.join("luxmc-loader.json"),serde_json::to_vec(&ServerLoader{kind:kind.clone(),version}).unwrap()).await.unwrap();
        let (runtime,_receiver)=super::super::tests::fixture_runtime(config.id.clone());
        let arguments=match manager.prepare_loader(&config,&directory,&java,&runtime).await {
            Ok(arguments)=>arguments,
            Err(error)=>{
                let path=temporary.keep();
                let log=std::fs::read(directory.join("luxmc-loader-installer.jar.log")).unwrap_or_default();
                let tail=String::from_utf8_lossy(&log[log.len().saturating_sub(8000)..]);
                panic!("{error}; preserved={}; officialLog={tail}; console: {:?}",path.display(),runtime.snapshot(0).console.iter().rev().take(20).map(|line|&line.text).collect::<Vec<_>>());
            }
        };
        assert!(!arguments.is_empty());assert!(directory.join("luxmc-loader-ready.json").exists());
        assert!(!directory.join("world/level.dat").exists());assert!(!directory.join("eula.txt").exists());
        let again=manager.prepare_loader(&config,&directory,&java,&runtime).await.unwrap();assert_eq!(arguments,again);
        eprintln!("OFFICIAL SERVER INSTALLER PASSED: {kind}, Minecraft {}, no game or world started",config.mc_version);
    }

    #[test]
    fn server_entrypoints_keep_loader_family_and_game_version_pinned() {
        let config = super::super::tests::fixture_config();
        for kind in ["fabric", "quilt", "forge", "neoforge"] {
            let loader = ServerLoader {
                kind: kind.into(),
                version: "fixture-version".into(),
            };
            let candidates = launch_candidates(&config, &loader);
            assert!(!candidates.is_empty());
            for arguments in candidates {
                assert!(!arguments.join(" ").contains("client"));
                assert!(
                    arguments.last().unwrap().ends_with(".jar")
                        || arguments.last().unwrap().ends_with("_args.txt")
                );
            }
        }
        let forge = launch_candidates(
            &config,
            &ServerLoader {
                kind: "forge".into(),
                version: "47.2.0".into(),
            },
        );
        assert!(forge
            .iter()
            .flatten()
            .any(|value| value.contains("1.21.1-47.2.0")));
        assert!(ServerLoader {
            kind: "forge".into(),
            version: "../../external".into()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn installer_java_is_detected_without_executing_the_jar() {
        let mut jar = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::FileOptions::default();
        jar.start_file("fixture/Main.class", options).unwrap();
        jar.write_all(&[0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 61])
            .unwrap();
        jar.start_file("META-INF/versions/25/fixture/Optional.class", options)
            .unwrap();
        jar.write_all(&[0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 69])
            .unwrap();
        let bytes = jar.finish().unwrap().into_inner();
        assert_eq!(installer_java(&bytes).unwrap(), 17);
    }

    #[tokio::test]
    async fn an_existing_world_is_not_silently_converted_to_another_loader_family() {
        let temporary = tempfile::tempdir().unwrap();
        let manager =
            LocalServerManager::new(temporary.path().join("servers"), reqwest::Client::new());
        let config = super::super::tests::fixture_config();
        let directory = super::super::tests::persist(&manager, &config).await;
        tokio::fs::create_dir(directory.join("world"))
            .await
            .unwrap();
        tokio::fs::write(directory.join("world/level.dat"), b"original")
            .await
            .unwrap();
        assert!(manager
            .set_loader(
                &config.id,
                ServerLoader {
                    kind: "fabric".into(),
                    version: "0.16.14".into()
                }
            )
            .await
            .is_err());
        assert_eq!(
            tokio::fs::read(directory.join("world/level.dat"))
                .await
                .unwrap(),
            b"original"
        );
        assert_eq!(manager.loader(&config.id).await.unwrap().kind, "vanilla");
    }
}
impl Default for ServerLoader {
    fn default() -> Self {
        Self {
            kind: "vanilla".into(),
            version: String::new(),
        }
    }
}

impl ServerLoader {
    pub(crate) fn validate(&self) -> AppResult<()> {
        if !["vanilla", "fabric", "quilt", "forge", "neoforge"].contains(&self.kind.as_str())
            || (self.kind != "vanilla" && self.version.is_empty())
            || self.version.len() > 96
            || !self
                .version
                .bytes()
                .all(|value| value.is_ascii_alphanumeric() || b"._-+".contains(&value))
        {
            return Err(invalid(
                "Selecione um loader e uma versão de servidor compatíveis.",
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Installed {
    loader: ServerLoader,
    arguments: Vec<String>,
    files: Vec<(String, String)>,
}

fn launch_candidates(config: &LocalServerConfig, loader: &ServerLoader) -> Vec<Vec<String>> {
    let platform = if cfg!(windows) {
        "win_args.txt"
    } else {
        "unix_args.txt"
    };
    match loader.kind.as_str() {
        "fabric" => vec![vec!["-jar".into(), "fabric-server-launch.jar".into()]],
        "quilt" => vec![vec!["-jar".into(), "quilt-server-launch.jar".into()]],
        "forge" => vec![
            vec![format!(
                "@libraries/net/minecraftforge/forge/{}-{}/{platform}",
                config.mc_version, loader.version
            )],
            vec![
                "-jar".into(),
                format!("forge-{}-{}.jar", config.mc_version, loader.version),
            ],
            vec![
                "-jar".into(),
                format!(
                    "forge-{}-{}-universal.jar",
                    config.mc_version, loader.version
                ),
            ],
        ],
        "neoforge" => vec![vec![format!(
            "@libraries/net/neoforged/neoforge/{}/{platform}",
            loader.version
        )]],
        _ => Vec::new(),
    }
}

fn installer_command(java: &Path, directory: &Path, arguments: &[String]) -> tokio::process::Command {
    let mut command = crate::core::process::tokio_command(java);
    command.current_dir(crate::core::process::java_working_directory(directory)).arg("-Djava.awt.headless=true").arg("-jar").arg("luxmc-loader-installer.jar").args(arguments);
    command
}

fn installer_java(bytes: &[u8]) -> AppResult<u32> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    if archive.len() > 40_000 {
        return Err(invalid("O instalador contém arquivos demais."));
    }
    let mut required = 8u32;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        if !file.name().ends_with(".class")
            || file.name().ends_with("module-info.class")
            || file.name().starts_with("META-INF/versions/")
        {
            continue;
        }
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;
        if header[..4] != [0xca, 0xfe, 0xba, 0xbe] {
            return Err(invalid("Classe do instalador inválida."));
        }
        required = required.max(
            u16::from_be_bytes([header[6], header[7]])
                .saturating_sub(44)
                .into(),
        );
    }
    if required > 99 {
        return Err(invalid(
            "O instalador requer uma versão Java não suportada.",
        ));
    }
    Ok(required)
}

fn valid_installer_hash(bytes:&[u8], expected:&str)->bool {
    if !expected.bytes().all(|byte|byte.is_ascii_hexdigit()) {return false;}
    match expected.len() {
        40=>format!("{:x}",sha1::Sha1::digest(bytes)).eq_ignore_ascii_case(expected),
        64=>format!("{:x}",sha2::Sha256::digest(bytes)).eq_ignore_ascii_case(expected),
        _=>false,
    }
}

impl LocalServerManager {
    pub async fn loader(&self, id: &str) -> AppResult<ServerLoader> {
        let directory = self.directory(id).await?;
        let loader = self
            .read(&directory, "luxmc-loader.json")
            .await?
            .map(|bytes| serde_json::from_slice(&bytes))
            .transpose()?
            .unwrap_or_default();
        ServerLoader::validate(&loader)?;
        Ok(loader)
    }

    pub async fn loader_versions(
        &self,
        id: &str,
        kind: &str,
    ) -> AppResult<Vec<crate::core::loaders::LoaderVersion>> {
        if !["fabric", "quilt", "forge", "neoforge"].contains(&kind) {
            return Err(invalid("Loader de servidor não suportado."));
        }
        crate::core::loaders::fetch_loader_versions(
            &self.http,
            kind,
            &self.config(id).await?.mc_version,
        )
        .await
    }

    pub async fn set_loader(&self, id: &str, loader: ServerLoader) -> AppResult<ServerLoader> {
        loader.validate()?;
        let _guard = self.change()?;
        if self.current(id).is_ok() {
            return Err(unavailable(
                "Pare e salve o servidor antes de configurar o loader.",
            ));
        }
        let directory = self.directory(id).await?;
        let previous = self.loader(id).await?;
        if previous == loader {
            let marker = directory.join("luxmc-loader-ready.json");
            if marker.exists() {
                backups::normal_file(&marker)?;
                tokio::fs::remove_file(marker).await?;
            }
            return Ok(loader);
        }
        if directory.join("world/level.dat").exists() && previous.kind != loader.kind {
            return Err(invalid("Este mundo já usa outro loader. Crie um servidor separado para evitar converter ou danificar o mundo."));
        }
        if loader.kind != "vanilla"
            && !self
                .loader_versions(id, &loader.kind)
                .await?
                .iter()
                .any(|version| version.id == loader.version)
        {
            return Err(invalid(
                "O provedor não confirmou esta versão do loader para o Minecraft selecionado.",
            ));
        }
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-loader.json"),
            &serde_json::to_vec_pretty(&loader)?,
        )
        .await?;
        Ok(loader)
    }

    async fn official_bytes(&self, url: &str, limit: usize) -> AppResult<Vec<u8>> {
        let url = url::Url::parse(url).map_err(|_| invalid("Endereço oficial inválido."))?;
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || !matches!(
                url.host_str(),
                Some(
                    "meta.fabricmc.net"
                        | "meta.quiltmc.org"
                        | "maven.fabricmc.net"
                        | "maven.quiltmc.org"
                        | "maven.minecraftforge.net"
                        | "maven.neoforged.net"
                )
            )
        {
            return Err(invalid(
                "O instalador não pertence a um provedor oficial suportado.",
            ));
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let mut response = client
            .get(url)
            .timeout(Duration::from_secs(120))
            .send()
            .await?
            .error_for_status()?;
        if response.status().is_redirection()
            || response
                .content_length()
                .is_some_and(|size| size > limit as u64)
        {
            return Err(invalid(
                "Download oficial excedeu o limite ou redirecionou para outro endereço.",
            ));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > limit {
                return Err(invalid("Download oficial muito grande."));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }

    pub(super) async fn prepare_loader(
        &self,
        config: &LocalServerConfig,
        directory: &Path,
        java: &Path,
        runtime: &Arc<Runtime>,
    ) -> AppResult<Vec<String>> {
        let loader = self.loader(&config.id).await?;
        if loader.kind == "vanilla" {
            return Ok(vec!["-jar".into(), "server.jar".into()]);
        }
        if let Some(bytes) = self.read(directory, "luxmc-loader-ready.json").await? {
            let installed: Installed = serde_json::from_slice(&bytes)?;
            if installed.loader == loader {
                if !launch_candidates(config, &loader).contains(&installed.arguments)
                    || installed.files.len() != 1
                    || installed
                        .arguments
                        .last()
                        .map(|value| value.trim_start_matches('@'))
                        != installed.files.first().map(|(path, _)| path.as_str())
                {
                    return Err(invalid("O registro de inicialização do loader está incompleto ou inválido. Reconfigure a mesma versão para reparar."));
                }
                for (relative, expected) in &installed.files {
                    let relative_path = Path::new(relative);
                    if relative_path.is_absolute()
                        || relative_path
                            .components()
                            .any(|value| !matches!(value, std::path::Component::Normal(_)))
                    {
                        return Err(invalid("Registro do loader inválido."));
                    }
                    let path = directory.join(relative_path);
                    backups::normal_file(&path)?;
                    if !tokio::fs::canonicalize(&path).await?.starts_with(directory) {
                        return Err(invalid("Arquivo do loader fora do servidor."));
                    }
                    if format!("{:x}", sha2::Sha256::digest(tokio::fs::read(path).await?))
                        != *expected
                    {
                        return Err(invalid("Um arquivo do loader foi alterado. Reinstale a mesma versão antes de continuar."));
                    }
                }
                return Ok(installed.arguments);
            }
        }
        runtime.log("luxmc", &format!("Preparando servidor {} {} com o instalador oficial. Nenhum script importado será executado.", loader.kind, loader.version));
        let (url, hash, installer_arguments, launch) = match loader.kind.as_str() {
            "fabric" | "quilt" => {
                let meta = if loader.kind == "fabric" {
                    "https://meta.fabricmc.net/v2/versions/installer"
                } else {
                    "https://meta.quiltmc.org/v3/versions/installer"
                };
                let entries: Vec<serde_json::Value> =
                    serde_json::from_slice(&self.official_bytes(meta, 2 * 1024 * 1024).await?)?;
                let entry = entries
                    .first()
                    .ok_or_else(|| unavailable("O provedor não retornou um instalador."))?;
                let url = entry
                    .get("url")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| invalid("Instalador sem URL."))?
                    .to_string();
                let hash = if loader.kind == "quilt" {
                    String::from_utf8(self.official_bytes(&format!("{url}.sha256"),4096).await?).map_err(|_|invalid("Checksum SHA-256 oficial inválido."))?.split_whitespace().next().unwrap_or_default().to_string()
                } else if let Some(hash) = entry
                    .pointer("/hashes/sha1")
                    .and_then(|value| value.as_str())
                {
                    hash.into()
                } else {
                    String::from_utf8(self.official_bytes(&format!("{url}.sha1"), 4096).await?)
                        .map_err(|_| invalid("Hash oficial inválido."))?
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string()
                };
                if loader.kind == "fabric" {
                    (
                        url,
                        hash,
                        vec![
                            "server".into(),
                            "-mcversion".into(),
                            config.mc_version.clone(),
                            "-loader".into(),
                            loader.version.clone(),
                            "-dir".into(),
                            ".".into(),
                        ],
                        vec!["-jar".into(), "fabric-server-launch.jar".into()],
                    )
                } else {
                    (
                        url,
                        hash,
                        vec![
                            "install".into(),
                            "server".into(),
                            config.mc_version.clone(),
                            loader.version.clone(),
                            "--install-dir=.".into(),
                        ],
                        vec!["-jar".into(), "quilt-server-launch.jar".into()],
                    )
                }
            }
            "forge" | "neoforge" => {
                let coordinate = if loader.kind == "forge" {
                    format!(
                        "net/minecraftforge/forge/{}-{}/forge-{}-{}",
                        config.mc_version, loader.version, config.mc_version, loader.version
                    )
                } else {
                    format!(
                        "net/neoforged/neoforge/{}/neoforge-{}",
                        loader.version, loader.version
                    )
                };
                let base = if loader.kind == "forge" {
                    "https://maven.minecraftforge.net"
                } else {
                    "https://maven.neoforged.net/releases"
                };
                let url = format!("{base}/{coordinate}-installer.jar");
                let hash =
                    String::from_utf8(self.official_bytes(&format!("{url}.sha1"), 4096).await?)
                        .map_err(|_| invalid("Hash oficial inválido."))?
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string();
                let platform = if cfg!(windows) {
                    "win_args.txt"
                } else {
                    "unix_args.txt"
                };
                let arguments = if loader.kind == "forge" {
                    format!(
                        "libraries/net/minecraftforge/forge/{}-{}/{platform}",
                        config.mc_version, loader.version
                    )
                } else {
                    format!(
                        "libraries/net/neoforged/neoforge/{}/{platform}",
                        loader.version
                    )
                };
                (
                    url,
                    hash,
                    vec!["--installServer".into()],
                    vec![format!("@{arguments}")],
                )
            }
            _ => return Err(invalid("Loader inválido.")),
        };
        if ![40,64].contains(&hash.len()) || !hash.bytes().all(|value| value.is_ascii_hexdigit()) || (loader.kind=="quilt" && hash.len()!=64) {
            return Err(invalid(
                "O provedor retornou um hash de instalador inválido.",
            ));
        }
        let bytes = self.official_bytes(&url, 120 * 1024 * 1024).await?;
        if !valid_installer_hash(&bytes,&hash) {
            return Err(invalid(
                "O instalador não passou na verificação de integridade.",
            ));
        }
        let detected_java = installer_java(&bytes)?;
        let required_java = (match detected_java {
            0..=8 => 8,
            9..=17 => 17,
            18..=21 => 21,
            value => value,
        })
        .max(config.artifact.java_major);
        let installer_java = if required_java > config.artifact.java_major {
            runtime.log(
                "luxmc",
                &format!(
                    "O instalador usa Java {required_java}; o servidor continuará usando Java {}.",
                    config.artifact.java_major
                ),
            );
            let data = self
                .root
                .parent()
                .ok_or_else(|| invalid("Pasta de dados inválida."))?;
            crate::core::java::JavaRuntimeManager::new(self.http.clone(), data.to_path_buf())
                .ensure_java(required_java)
                .await?
        } else {
            java.to_path_buf()
        };
        let installer = directory.join("luxmc-loader-installer.jar");
        crate::core::mods::pack_download::atomic_write(&installer, &bytes).await?;
        let mut command = installer_command(&installer_java, directory, &installer_arguments);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(target_os = "linux")]
        for variable in [
            "LD_LIBRARY_PATH",
            "LD_PRELOAD",
            "APPDIR",
            "APPIMAGE",
            "OWD",
            "GIO_MODULE_DIR",
            "GTK_PATH",
            "GSETTINGS_SCHEMA_DIR",
        ] {
            command.env_remove(variable);
        }
        #[cfg(target_os = "linux")]
        process_guard::prepare(&mut command);
        let mut child = command.spawn()?;
        #[cfg(windows)]
        let _job = process_guard::contain(&child)?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| unavailable("O instalador não abriu a saída."))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| unavailable("O instalador não abriu a saída de erros."))?;
        let readers = [
            tokio::spawn(read_console(stdout, runtime.clone(), "installer")),
            tokio::spawn(read_console(stderr, runtime.clone(), "stderr")),
        ];
        let mut stop = runtime.stop.subscribe();
        let result = tokio::select! { biased; _ = wait_stop(&mut stop) => { let _ = child.start_kill(); let _ = child.wait().await; Err(unavailable("Preparação do loader cancelada; mundo preservado.")) }, result = tokio::time::timeout(Duration::from_secs(300), child.wait()) => match result { Ok(Ok(status)) if status.success() => Ok(()), Ok(Ok(status)) => Err(unavailable(format!("Não foi possível preparar {} {} (código {}). Confira os detalhes no console.", loader.kind, loader.version, status.code().unwrap_or(-1)))), Ok(Err(error)) => Err(error.into()), Err(_) => { let _ = child.start_kill(); let _ = child.wait().await; Err(unavailable("O instalador excedeu cinco minutos. Confira sua conexão e tente novamente.")) } } };
        for mut reader in readers {
            if tokio::time::timeout(Duration::from_secs(1), &mut reader)
                .await
                .is_err()
            {
                reader.abort();
            }
        }
        result?;
        let mut launch = launch;
        if loader.kind == "forge"
            && launch.first().is_some_and(|argument| {
                argument.starts_with('@')
                    && !directory.join(argument.trim_start_matches('@')).exists()
            })
        {
            if let Some(candidate) =
                launch_candidates(config, &loader)
                    .into_iter()
                    .find(|candidate| {
                        candidate.len() == 2
                            && candidate
                                .last()
                                .is_some_and(|name| directory.join(name).is_file())
                    })
            {
                launch = candidate;
            }
        }
        let mut files = Vec::new();
        for argument in &launch {
            if argument == "-jar" {
                continue;
            }
            let relative = argument.trim_start_matches('@');
            let path = directory.join(relative);
            backups::normal_file(&path)?;
            if !tokio::fs::canonicalize(&path).await?.starts_with(directory) {
                return Err(invalid("O instalador retornou um caminho externo."));
            }
            files.push((
                relative.into(),
                format!("{:x}", sha2::Sha256::digest(tokio::fs::read(path).await?)),
            ));
        }
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-loader-ready.json"),
            &serde_json::to_vec(&Installed {
                loader,
                arguments: launch.clone(),
                files,
            })?,
        )
        .await?;
        Ok(launch)
    }
}
