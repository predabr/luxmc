use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashDiagnosis {
    pub has_error: bool,
    pub title: String,
    pub message: String,
    pub solution: String,
    pub category: String,
    pub offending_mod: Option<String>,
    pub recommended_action: Option<String>,
    pub log_snippet: Option<String>,
}

impl Default for CrashDiagnosis {
    fn default() -> Self {
        Self {
            has_error: false,
            title: "Nenhum Erro Detectado".to_string(),
            message: "O jogo fechou normalmente ou não foram encontrados erros nos relatórios recentes.".to_string(),
            solution: "Tudo parece em ordem. Você pode iniciar o jogo novamente.".to_string(),
            category: "none".to_string(),
            offending_mod: None,
            recommended_action: None,
            log_snippet: None,
        }
    }
}

pub fn analyze_crash_text(log: &str) -> CrashDiagnosis {
    let trimmed = log.trim();
    if trimmed.is_empty() {
        return CrashDiagnosis::default();
    }

    for (needle, title, message, solution, category) in [
        ("Invalid session", "Sessão de jogo inválida", "O servidor solicitou uma sessão oficial e a sessão enviada não foi aceita.", "Em uma sala privada Luxmc, atualize e reinicie o Minecraft dos dois participantes. Em servidores oficiais, entre novamente com a conta Microsoft que possui o jogo.", "authentication"),
        ("Client shutdown from post-main", "O jogo terminou, mas o processo não fechou", "O watchdog detectou threads ainda abertas depois do encerramento do Minecraft.", "Atualize o launcher e os mods. Abra o relatório de threads para identificar o componente que mantém o processo ativo.", "shutdown"),
        ("UnknownHostException", "Endereço do servidor não resolvido", "O computador não conseguiu resolver o nome do servidor no DNS.", "Confira o endereço e a conexão. Não desative IPv6 automaticamente; teste o domínio e verifique os registros DNS/SRV do servidor.", "network_dns"),
        ("Connection refused", "O destino recusou a conexão", "Não havia um servidor aceitando conexões no endereço e porta usados.", "Abra o mundo em LAN novamente e aguarde a atualização da sala. Verifique se o hospedeiro continua conectado e se a versão e o modpack são compatíveis.", "network_refused"),
    ] {
        if trimmed.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
            return CrashDiagnosis { has_error: true, title: title.into(), message: message.into(), solution: solution.into(), category: category.into(), offending_mod: None, recommended_action: Some("open_folder".into()), log_snippet: extract_snippet(trimmed, needle) };
        }
    }

    let re_optifine_create = Regex::new(r"(?i)(create.*optifine|optifine.*create|Flywheel.*OptiFine|net\.coderbot\.iris.*optifine|com\.simibubi\.create.*optifine)").unwrap();
    if re_optifine_create.is_match(trimmed) {
        return CrashDiagnosis {
            has_error: true,
            title: "Incompatibilidade: Create & OptiFine".to_string(),
            message: "O mod Create (e seu motor gráfico Flywheel) é incompatível com o OptiFine. O OptiFine quebra a renderização moderna e causa fechamento imediato do jogo.".to_string(),
            solution: "Desative o OptiFine na lista de mods. Instale o Embeddium (no Forge) ou Sodium (no Fabric) + Oculus/Iris caso queira suporte a shaders.".to_string(),
            category: "mod_conflict".to_string(),
            offending_mod: Some("OptiFine".to_string()),
            recommended_action: Some("disable_optifine".to_string()),
            log_snippet: extract_snippet(trimmed, "OptiFine"),
        };
    }

    let re_optifine_sodium = Regex::new(r"(?i)(sodium.*optifine|optifine.*sodium)").unwrap();
    if re_optifine_sodium.is_match(trimmed) {
        return CrashDiagnosis {
            has_error: true,
            title: "Conflito: Sodium & OptiFine".to_string(),
            message: "Sodium e OptiFine realizam modificações concorrentes no motor gráfico e não podem ser usados juntos.".to_string(),
            solution: "Desative o OptiFine e instale o mod Iris Shaders para ter shaders com máximo FPS.".to_string(),
            category: "mod_conflict".to_string(),
            offending_mod: Some("OptiFine".to_string()),
            recommended_action: Some("disable_optifine".to_string()),
            log_snippet: extract_snippet(trimmed, "optifine"),
        };
    }

    let re_embeddium_rubidium = Regex::new(r"(?i)(rubidium.*embeddium|embeddium.*rubidium)").unwrap();
    if re_embeddium_rubidium.is_match(trimmed) {
        return CrashDiagnosis {
            has_error: true,
            title: "Mods Duplicados: Rubidium & Embeddium".to_string(),
            message: "Rubidium e Embeddium são versões conflitantes da mesma biblioteca de renderização.".to_string(),
            solution: "Desative o Rubidium na lista de mods e mantenha apenas o Embeddium ativo.".to_string(),
            category: "mod_conflict".to_string(),
            offending_mod: Some("Rubidium".to_string()),
            recommended_action: Some("disable_mod".to_string()),
            log_snippet: extract_snippet(trimmed, "rubidium"),
        };
    }

    let re_oom = Regex::new(r"(?i)(java\.lang\.OutOfMemoryError|GC overhead limit exceeded|insufficient memory for the Java Runtime)").unwrap();
    if re_oom.is_match(trimmed) {
        return CrashDiagnosis {
            has_error: true,
            title: "Memória RAM Esgotada (Out of Memory)".to_string(),
            message: "O Minecraft encerrou porque a memória RAM alocada para esta instância esgotou durante o carregamento de texturas e modelos.".to_string(),
            solution: "Aumente a quantidade de RAM nas configurações da instância para pelo menos 6 GB ou 8 GB (ou utilize o Lux Boost para otimizar o consumo).".to_string(),
            category: "out_of_memory".to_string(),
            offending_mod: None,
            recommended_action: Some("increase_ram".to_string()),
            log_snippet: extract_snippet(trimmed, "OutOfMemoryError"),
        };
    }

    let re_jvm_opt = Regex::new(r"(?i)Unrecognized VM option '([^']+)'").unwrap();
    if let Some(caps) = re_jvm_opt.captures(trimmed) {
        let opt = caps.get(1).map(|m| m.as_str()).unwrap_or("desconhecida");
        return CrashDiagnosis {
            has_error: true,
            title: format!("Opção JVM Inválida: '{opt}'"),
            message: format!("A máquina virtual Java encerrou pois não reconhece a opção '{opt}' nesta versão do Java."),
            solution: "O Luxmc removeu a flag incompatível dos argumentos para você.".to_string(),
            category: "jvm_option".to_string(),
            offending_mod: Some(opt.to_string()),
            recommended_action: Some("fix_jvm_args".to_string()),
            log_snippet: extract_snippet(trimmed, "Unrecognized VM option"),
        };
    }

    let re_java_ver = Regex::new(r"(?i)UnsupportedClassVersionError: .* has been compiled by a more recent version of the Java Runtime \(class file version (\d+)").unwrap();
    if let Some(caps) = re_java_ver.captures(trimmed) {
        let version_code = caps.get(1).map(|m| m.as_str()).unwrap_or("65");
        let (needed, current) = match version_code {
            "65" => ("Java 21", "Java 17 ou inferior"),
            "61" => ("Java 17", "Java 8 ou 11"),
            "52" => ("Java 8", "Java 7 ou inferior"),
            _ => ("Java mais recente", "versão antiga do Java"),
        };
        return CrashDiagnosis {
            has_error: true,
            title: "Versão do Java Incompatível".to_string(),
            message: format!("Um dos mods requer {needed}, mas a instância está executando com {current}."),
            solution: format!("Altere a versão do Java nas configurações da instância para {needed}."),
            category: "java_version".to_string(),
            offending_mod: None,
            recommended_action: Some("install_java".to_string()),
            log_snippet: extract_snippet(trimmed, "UnsupportedClassVersionError"),
        };
    }

    let re_jpms = Regex::new(r"(?i)java\.lang\.module\.ResolutionException: Modules ([^\s]+) and ([^\s]+) export package ([^\s]+)").unwrap();
    if let Some(caps) = re_jpms.captures(trimmed) {
        let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("A");
        let m2 = caps.get(2).map(|m| m.as_str()).unwrap_or("B");
        let pkg = caps.get(3).map(|m| m.as_str()).unwrap_or("net.minecraft");
        return CrashDiagnosis {
            has_error: true,
            title: "Conflito de Arquivos Duplicados (JPMS)".to_string(),
            message: format!("Dois arquivos ({m1} e {m2}) exportam simultaneamente o pacote {pkg}."),
            solution: "Verifique a pasta de mods da instância e remova arquivos .jar duplicados ou redundantes.".to_string(),
            category: "mod_conflict".to_string(),
            offending_mod: None,
            recommended_action: Some("open_folder".to_string()),
            log_snippet: extract_snippet(trimmed, "ResolutionException"),
        };
    }

    let re_forge_dep = Regex::new(r"(?i)Mod ID: '([^']+)', Requested by: '([^']+)', Expected range: '[^']*', Actual version: '\[MISSING\]'").unwrap();
    if let Some(caps) = re_forge_dep.captures(trimmed) {
        let missing_mod = caps.get(1).map(|m| m.as_str()).unwrap_or("desconhecido");
        let requested_by = caps.get(2).map(|m| m.as_str()).unwrap_or("um mod");
        return CrashDiagnosis {
            has_error: true,
            title: "Mod Faltando no Modpack".to_string(),
            message: format!("O mod '{requested_by}' requer o mod '{missing_mod}', mas ele não foi encontrado na pasta de mods."),
            solution: format!("Use a opção 'Reparar Modpack' na instância ou instale '{missing_mod}' na aba de Mods."),
            category: "missing_dependency".to_string(),
            offending_mod: Some(missing_mod.to_string()),
            recommended_action: Some("repair_modpack".to_string()),
            log_snippet: extract_snippet(trimmed, "MISSING"),
        };
    }

    let re_dep = Regex::new(r"(?i)(?:ModResolutionException|Missing or unsupported mandatory dependencies:).*?requires (?:mod |version )?([a-zA-Z0-9_\-]+)").unwrap();
    if let Some(caps) = re_dep.captures(trimmed) {
        let dep = caps.get(1).map(|m| m.as_str()).unwrap_or("dependência");
        return CrashDiagnosis {
            has_error: true,
            title: "Mod de Dependência Ausente".to_string(),
            message: format!("Um dos mods instalados requer o mod '{dep}' para funcionar corretamente."),
            solution: format!("Acesse a aba de Mods e instale '{dep}' para resolver a pendência."),
            category: "missing_dependency".to_string(),
            offending_mod: Some(dep.to_string()),
            recommended_action: Some("install_mod".to_string()),
            log_snippet: extract_snippet(trimmed, "requires"),
        };
    }

    let re_mixin = Regex::new(r"(?i)MixinApplyError:.*?from mod ([a-zA-Z0-9_\-]+)").unwrap();
    if let Some(caps) = re_mixin.captures(trimmed) {
        let mod_name = caps.get(1).map(|m| m.as_str()).unwrap_or("desconhecido");
        return CrashDiagnosis {
            has_error: true,
            title: format!("Falha no Mod '{mod_name}' (Mixin)"),
            message: format!("O mod '{mod_name}' falhou ao injetar suas modificações de código no Minecraft."),
            solution: format!("Desative o mod '{mod_name}' pelo interruptor liga/desliga ou procure por uma versão atualizada."),
            category: "mod_conflict".to_string(),
            offending_mod: Some(mod_name.to_string()),
            recommended_action: Some("disable_mod".to_string()),
            log_snippet: extract_snippet(trimmed, "MixinApplyError"),
        };
    }

    let re_gpu = Regex::new(r"(?i)(GLFW error 65542|WGL: The driver does not appear to support OpenGL|org\.lwjgl\.opengl|vkCreateInstance failed|Mesa:.*error)").unwrap();
    if re_gpu.is_match(trimmed) {
        return CrashDiagnosis {
            has_error: true,
            title: "Falha de Driver Gráfico (OpenGL / Vulkan)".to_string(),
            message: "O motor gráfico não conseguiu inicializar o contexto de vídeo através do driver atual.".to_string(),
            solution: "Desative a opção de aceleração Vulkan (Zink) nas configurações da instância ou atualize os drivers da sua placa de vídeo.".to_string(),
            category: "driver_issue".to_string(),
            offending_mod: None,
            recommended_action: Some("disable_vulkan".to_string()),
            log_snippet: extract_snippet(trimmed, "GLFW"),
        };
    }

    let re_bad_ctor = Regex::new(r"(?i)Invalid class .+ no constructor taking").unwrap();
    if re_bad_ctor.is_match(trimmed) {
        let re_from = Regex::new(r"(?i)Caught exception from (.+?) \(").unwrap();
        let mod_label = re_from
            .captures(trimmed)
            .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .unwrap_or_else(|| "um dos mods instalados".to_string());
        return CrashDiagnosis {
            has_error: true,
            title: "Mod incompatível com a versão do Minecraft".to_string(),
            message: format!(
                "O mod \"{mod_label}\" travou ao registrar suas entidades. Isso ocorre quando o .jar foi compilado para outra versão do Minecraft (ou outro loader) daquela configurada nesta instância."
            ),
            solution: "Confira a versão escrita no nome do arquivo do mod (ex.: ...-1.12.2-forge...) e edite a instância para essa mesma versão do Minecraft, ou substitua o .jar por um build compatível com a versão atual.".to_string(),
            category: "version_mismatch".to_string(),
            offending_mod: Some(mod_label),
            recommended_action: Some("fix_mc_version".to_string()),
            log_snippet: extract_snippet(trimmed, "no constructor taking"),
        };
    }

    if trimmed.contains("Exception") || trimmed.contains("Error") || trimmed.contains("FATAL") {
        let re_cause = Regex::new(r"(?m)^Caused by: ([^\n\r]+)").unwrap();
        let cause = re_cause
            .captures(trimmed)
            .and_then(|c| c.get(1).map(|m| m.as_str()))
            .unwrap_or("O processo do Minecraft encerrou com código de erro não especificado.");

        return CrashDiagnosis {
            has_error: true,
            title: "Erro de Execução no Minecraft".to_string(),
            message: cause.to_string(),
            solution: "Analise o log completo ou desative mods adicionados recentemente para isolar o problema.".to_string(),
            category: "unknown".to_string(),
            offending_mod: None,
            recommended_action: Some("open_folder".to_string()),
            log_snippet: extract_snippet(trimmed, "Caused by"),
        };
    }

    CrashDiagnosis::default()
}

fn extract_snippet(log: &str, target: &str) -> Option<String> {
    let lines: Vec<&str> = log.lines().collect();
    let lower_target = target.to_lowercase();
    let idx = lines.iter().position(|l| l.to_lowercase().contains(&lower_target))?;
    let start = idx.saturating_sub(2);
    let end = std::cmp::min(lines.len(), idx + 6);
    Some(lines[start..end].join("\n"))
}

pub fn mod_compatibility_warnings(
    mods_dir: &Path,
    mc_version: &str,
    loader: &str,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if mc_version.trim().is_empty() {
        return warnings;
    }
    let instance_version = mc_version.trim().to_ascii_lowercase();
    let instance_loader = loader.trim().to_ascii_lowercase();

    let incompatible: &[&str] = match instance_loader.as_str() {
        "forge" => &["fabric", "quilt", "neoforge"],
        "fabric" => &["forge", "quilt"],
        "quilt" => &["forge", "neoforge"],
        "neoforge" => &["fabric", "quilt"],
        _ => &[],
    };

    let re_mc = Regex::new(r"(?i)(?:^|[-_+ .])(?:mc|minecraft)[-_ ]?(\d{1,2}\.\d{1,2}(?:\.\d{1,2})?)(?:[-_+ .]|$)|^optifine[-_](\d{1,2}\.\d{1,2}(?:\.\d{1,2})?)[-_]|(?:forge|fabric|quilt|neoforge)[-_](\d{1,2}\.\d{1,2}(?:\.\d{1,2})?)[-_]v\d").unwrap();

    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return warnings;
    };
    for entry in entries.flatten() {
        if warnings.len() >= 4 {
            break;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".disabled") {
            continue;
        }
        if !lower.ends_with(".jar") {
            continue;
        }

        let metadata = crate::core::mods::validation::mod_metadata(&entry.path());
        let file_version = if metadata.as_ref().is_some_and(|metadata| metadata.minecraft_declared) { None } else {
            re_mc.captures(&lower).and_then(|caps| (1..=3).find_map(|index| caps.get(index))).map(|value| value.as_str().to_string())
        };

        if let Some(found) = file_version {
            if found != instance_version && !instance_version.starts_with(&format!("{found}.")) {
                warnings.push(format!(
                    "\"{name}\" parece ser para o Minecraft {found}, mas esta instância roda {instance_version}."
                ));
                continue;
            }
        }

        if let Some(metadata) = metadata.filter(|metadata| !metadata.loaders.is_empty()) {
            if !metadata.loaders.contains(&instance_loader) && !(instance_loader == "quilt" && metadata.loaders.contains("fabric")) && metadata.loaders.iter().all(|loader| incompatible.contains(&loader.as_str())) {
                warnings.push(format!("\"{name}\" declara suporte a {}, mas esta instância usa {instance_loader}.", metadata.loaders.into_iter().collect::<Vec<_>>().join(" / ")));
            }
            continue;
        }
        let stripped = lower.replace("neoforge", "");
        let has_own_loader = match instance_loader.as_str() {
            "forge" => stripped.contains("forge"),
            "fabric" => lower.contains("fabric"),
            "quilt" => lower.contains("quilt"),
            "neoforge" => lower.contains("neoforge"),
            _ => true,
        };
        if !has_own_loader {
            if let Some(foreign) = incompatible.iter().find(|l| lower.contains(**l)) {
                warnings.push(format!(
                    "\"{name}\" é um mod para {foreign}, mas esta instância usa {instance_loader}."
                ));
            }
        }
    }
    warnings
}

#[cfg(test)]
mod compatibility_filename_tests {
    use super::*;
    #[test]
    fn declared_compatibility_takes_precedence_over_filename_hints() {
        use std::io::Write;
        let directory = tempfile::tempdir().unwrap();
        for name in ["dynamic-fps-3.11.4+minecraft-1.21.0-fabric.jar", "reeses-sodium-options-fabric-1.8.3+mc1.21.4.jar", "forge-config-fabric-mc1.20.1.jar"] {
            let file = std::fs::File::create(directory.path().join(name)).unwrap();
            let mut archive = zip::ZipWriter::new(file);
            archive.start_file("fabric.mod.json", zip::write::FileOptions::default()).unwrap();
            archive.write_all(br#"{"id":"example","depends":{"minecraft":">=1.21 <=1.21.4"}}"#).unwrap();
            archive.finish().unwrap();
        }
        assert!(mod_compatibility_warnings(directory.path(), "1.21.1", "fabric").is_empty());
        assert!(mod_compatibility_warnings(directory.path(), "1.21.1", "quilt").is_empty());
        assert_eq!(mod_compatibility_warnings(directory.path(), "1.21.1", "forge").len(), 3);
    }
    #[test]
    fn mod_versions_are_not_minecraft_versions() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["BetterThirdPerson-Fabric-1.21-1.9.0.jar", "citresewn-1.2.2+1.21.jar", "Cobblemon-fabric-1.8.1+1.21.1.jar", "emi-1.1.24+1.21.1+fabric.jar"] { std::fs::write(directory.path().join(name), b"fixture").unwrap(); }
        assert!(mod_compatibility_warnings(directory.path(), "1.21.1", "fabric").is_empty());
        assert!(mod_compatibility_warnings(directory.path(), "1.21.1", "quilt").is_empty());
        std::fs::write(directory.path().join("example-mc1.19.2-fabric.jar"), b"fixture").unwrap();
        let warnings = mod_compatibility_warnings(directory.path(), "1.21.1", "fabric");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("1.19.2"));
    }
}

pub fn diagnose_instance(game_dir: &Path) -> CrashDiagnosis {
    let crash_dir = game_dir.join("crash-reports");
    if crash_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&crash_dir) {
            let mut reports: Vec<_> = entries
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
                .collect();
            reports.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
            if let Some(latest) = reports.last() {
                if let Ok(content) = std::fs::read_to_string(latest.path()) {
                    let d = analyze_crash_text(&content);
                    if d.has_error {
                        return d;
                    }
                }
            }
        }
    }

    let stderr_file = game_dir.join("logs").join("stderr_stream.log");
    if stderr_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&stderr_file) {
            let d = analyze_crash_text(&content);
            if d.has_error {
                return d;
            }
        }
    }

    let latest_log = game_dir.join("logs").join("latest.log");
    if latest_log.exists() {
        if let Ok(content) = std::fs::read_to_string(&latest_log) {
            let d = analyze_crash_text(&content);
            if d.has_error {
                return d;
            }
        }
    }

    CrashDiagnosis::default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModConflict {
    pub title: String,
    pub description: String,
    pub mod_a: String,
    pub mod_b: String,
    pub recommended_action: String,
    pub file_to_disable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreLaunchCheckResult {
    pub has_conflicts: bool,
    pub conflicts: Vec<ModConflict>,
    pub duplicates: Vec<String>,
}

pub fn check_mod_conflicts(jar_filenames: &[String]) -> PreLaunchCheckResult {
    check_mod_conflicts_with_paths(jar_filenames, &[])
}

pub fn check_mod_conflicts_with_paths(
    jar_filenames: &[String],
    jar_paths: &[std::path::PathBuf],
) -> PreLaunchCheckResult {
    let mut conflicts = Vec::new();
    let mut duplicates = Vec::new();
    let mut seen_mods: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let mut has_optifine = None;
    let mut has_iris = None;
    let mut has_oculus = None;
    let mut has_sodium = None;
    let mut has_embeddium = None;
    let mut has_create = None;

    for (idx, fname) in jar_filenames.iter().enumerate() {
        let lower = fname.to_lowercase();
        if lower.ends_with(".disabled") {
            continue;
        }

        let base = if let Some(path) = jar_paths.get(idx) {
            crate::core::mods::validation::primary_ids(path)
                .map(|ids| ids.into_iter().collect::<Vec<_>>().join("+"))
                .unwrap_or_else(|| fname.to_lowercase())
        } else {
            let stem = fname.strip_suffix(".jar").unwrap_or(fname).to_lowercase();
            let parts: Vec<&str> = stem.split(&['-', '_'][..]).collect();
            let non_version_parts: Vec<&str> = parts
                .into_iter()
                .take_while(|part| !part.chars().any(|c| c.is_ascii_digit()))
                .collect();
            non_version_parts.join("-")
        };

        const COMMON_SINGLE_PREFIXES: &[&str] = &[
            "fabric", "forge", "neoforge", "quilt", "ftb", "kubejs", "create", 
            "yungs", "macaws", "allthe", "better", "simple"
        ];

        if base.len() >= 3 && !COMMON_SINGLE_PREFIXES.contains(&base.as_str()) {
            if let Some(prev) = seen_mods.get(&base) {
                duplicates.push(format!("Duplicata de mod detectada: '{}' e '{}'", prev, fname));
            } else {
                seen_mods.insert(base, fname.clone());
            }
        }

        if lower.contains("optifine") { has_optifine = Some(fname.clone()); }
        if lower.contains("iris") { has_iris = Some(fname.clone()); }
        if lower.contains("oculus") { has_oculus = Some(fname.clone()); }
        if lower.contains("sodium") { has_sodium = Some(fname.clone()); }
        if lower.contains("embeddium") { has_embeddium = Some(fname.clone()); }
        if lower.contains("create-") || lower.contains("create_") { has_create = Some(fname.clone()); }
    }

    if let (Some(ref opti), Some(ref iris)) = (&has_optifine, &has_iris) {
        conflicts.push(ModConflict {
            title: "Conflito: OptiFine & Iris Shaders".into(),
            description: "Ambos os mods tentam controlar o pipeline de shaders. Executar ambos juntos causa tela preta ou crash imediato.".into(),
            mod_a: opti.clone(),
            mod_b: iris.clone(),
            recommended_action: "Desativar OptiFine (Iris oferece compatibilidade moderna e maior FPS)".into(),
            file_to_disable: opti.clone(),
        });
    }

    if let (Some(ref opti), Some(ref oculus)) = (&has_optifine, &has_oculus) {
        conflicts.push(ModConflict {
            title: "Conflito: OptiFine & Oculus".into(),
            description: "Oculus e OptiFine entram em conflito de inicialização gráfica no Forge/NeoForge.".into(),
            mod_a: opti.clone(),
            mod_b: oculus.clone(),
            recommended_action: "Desativar OptiFine".into(),
            file_to_disable: opti.clone(),
        });
    }

    if let (Some(ref opti), Some(ref sod)) = (&has_optifine, &has_sodium) {
        conflicts.push(ModConflict {
            title: "Conflito: OptiFine & Sodium".into(),
            description: "Sodium e OptiFine reescrevem o motor de renderização concorrentemente.".into(),
            mod_a: opti.clone(),
            mod_b: sod.clone(),
            recommended_action: "Desativar OptiFine (Sodium dobra o FPS do jogo)".into(),
            file_to_disable: opti.clone(),
        });
    }

    if let (Some(ref opti), Some(ref emb)) = (&has_optifine, &has_embeddium) {
        conflicts.push(ModConflict {
            title: "Conflito: OptiFine & Embeddium".into(),
            description: "Embeddium e OptiFine são incompatíveis.".into(),
            mod_a: opti.clone(),
            mod_b: emb.clone(),
            recommended_action: "Desativar OptiFine".into(),
            file_to_disable: opti.clone(),
        });
    }

    if let (Some(ref opti), Some(ref crt)) = (&has_optifine, &has_create) {
        conflicts.push(ModConflict {
            title: "Incompatibilidade: OptiFine & Create (Flywheel)".into(),
            description: "O motor de renderização de engrenagens do Create crasha ao detectar o OptiFine.".into(),
            mod_a: opti.clone(),
            mod_b: crt.clone(),
            recommended_action: "Desativar OptiFine e utilizar Embeddium/Sodium".into(),
            file_to_disable: opti.clone(),
        });
    }

    let has_conflicts = !conflicts.is_empty();
    PreLaunchCheckResult {
        has_conflicts,
        conflicts,
        duplicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_shutdown_and_network_failures_have_specific_diagnoses() {
        for (log, category) in [("Failed to log in: Invalid session", "authentication"), ("java.lang.Error: Watchdog (Client shutdown from post-main)", "shutdown"), ("java.net.UnknownHostException: server.example", "network_dns"), ("java.net.ConnectException: Connection refused", "network_refused")] {
            let diagnosis = analyze_crash_text(log);
            assert!(diagnosis.has_error);
            assert_eq!(diagnosis.category, category);
            assert!(!diagnosis.solution.is_empty());
        }
    }

    #[test]
    fn test_diagnose_optifine_create_conflict() {
        let log = "net.minecraftforge.fml.ModLoadingException: com.simibubi.create and optifine are incompatible";
        let diag = analyze_crash_text(log);
        assert!(diag.has_error);
        assert_eq!(diag.category, "mod_conflict");
        assert_eq!(diag.offending_mod.as_deref(), Some("OptiFine"));
    }

    #[test]
    fn test_diagnose_missing_forge_dependency() {
        let log = "Mod ID: 'jei', Requested by: 'atm10', Expected range: '[15.0,)', Actual version: '[MISSING]'";
        let diag = analyze_crash_text(log);
        assert!(diag.has_error);
        assert_eq!(diag.category, "missing_dependency");
        assert_eq!(diag.offending_mod.as_deref(), Some("jei"));
        assert_eq!(diag.recommended_action.as_deref(), Some("repair_modpack"));
    }

    #[test]
    fn test_diagnose_out_of_memory() {
        let log = "java.lang.OutOfMemoryError: Java heap space";
        let diag = analyze_crash_text(log);
        assert!(diag.has_error);
        assert_eq!(diag.category, "out_of_memory");
        assert_eq!(diag.recommended_action.as_deref(), Some("increase_ram"));
    }

    #[test]
    fn test_diagnose_jpms_conflict() {
        let log = "java.lang.module.ResolutionException: Modules minecraft and _1._21._1 export package net.minecraft.util to module quack";
        let diag = analyze_crash_text(log);
        assert!(diag.has_error);
        assert_eq!(diag.category, "mod_conflict");
    }

    #[test]
    fn test_check_mod_conflicts_optifine_iris() {
        let files = vec![
            "OptiFine_1.20.1_HD_U_I6.jar".to_string(),
            "iris-mc1.20.1-1.6.11.jar".to_string(),
            "sodium-fabric-mc1.20.1-0.5.8.jar".to_string(),
        ];
        let res = check_mod_conflicts(&files);
        assert!(res.has_conflicts);
        assert!(res.conflicts.iter().any(|c| c.title.contains("OptiFine & Iris")));
        assert!(res.conflicts.iter().any(|c| c.title.contains("OptiFine & Sodium")));
    }

    #[test]
    fn test_check_mod_conflicts_duplicates() {
        let files = vec![
            "jei-1.20.1-forge-15.0.0.jar".to_string(),
            "jei-1.20.1-forge-15.1.0.jar".to_string(),
        ];
        let res = check_mod_conflicts(&files);
        assert_eq!(res.duplicates.len(), 1);
    }

    #[test]
    fn test_diagnose_entity_constructor_version_mismatch() {
        let log = "net.minecraftforge.fml.common.LoaderExceptionModCrash: Caught exception from SchnurriTV's Sexmod (sexmod)\nCaused by: java.lang.RuntimeException: Invalid class class com.schnurritv.sexmod.cQ no constructor taking net.minecraft.world.World";
        let diag = analyze_crash_text(log);
        assert!(diag.has_error);
        assert_eq!(diag.category, "version_mismatch");
        assert_eq!(diag.recommended_action.as_deref(), Some("fix_mc_version"));
        assert!(diag.offending_mod.as_deref().unwrap().contains("Sexmod"));
    }

    fn write_mod(dir: &Path, name: &str) {
        std::fs::write(dir.join(name), b"jar").unwrap();
    }

    #[test]
    fn test_mod_compatibility_warnings() {
        let dir = std::env::temp_dir().join(format!("luxmc_compat_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_mod(&dir, "Jenny-Mod-Forge-1.12.2-v1.9.0.jar");
        write_mod(&dir, "jei-1.12-4.8.0.1.jar");
        write_mod(&dir, "sodium-0.5.8+mc1.12.2-fabric.jar");
        write_mod(&dir, "OptiFine_1.12.2_HD_U_G5.jar");

        let warnings = mod_compatibility_warnings(&dir, "1.12", "forge");
        let joined = warnings.join("\n");
        assert!(joined.contains("Jenny-Mod-Forge-1.12.2-v1.9.0.jar"), "{joined}");
        assert!(joined.contains("sodium"), "{joined}");
        assert!(joined.contains("OptiFine"), "{joined}");
        assert!(!joined.contains("jei-1.12"), "{joined}");

        let ok = mod_compatibility_warnings(&dir, "1.12.2", "forge");
        let joined_ok = ok.join("\n");
        assert!(!joined_ok.contains("Jenny-Mod-Forge-1.12.2-v1.9.0.jar"), "{joined_ok}");
        assert!(!joined_ok.contains("OptiFine"), "{joined_ok}");
        assert!(joined_ok.contains("sodium"), "{joined_ok}");

        let wrong_loader = mod_compatibility_warnings(&dir, "1.12.2", "fabric");
        let joined_loader = wrong_loader.join("\n");
        assert!(joined_loader.contains("Jenny-Mod-Forge-1.12.2-v1.9.0.jar"), "{joined_loader}");
        assert!(!joined_loader.contains("sodium"), "{joined_loader}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
