use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerSettings {
    pub motd: String,
    pub difficulty: String,
    pub gamemode: String,
    pub force_gamemode: bool,
    pub pvp: bool,
    pub whitelist: bool,
    pub view_distance: u8,
    pub simulation_distance: u8,
    pub port: u16,
    pub autosave_minutes: u16,
    pub resource_pack_url: String,
    pub resource_pack_file: String,
    pub resource_pack_sha1: String,
    pub require_resource_pack: bool,
    pub seed: Option<String>,
    pub hardcore: bool,
    pub generate_structures: bool,
    pub spawn_protection:u16,
    pub allow_flight:bool,
    pub command_blocks:bool,
    pub idle_timeout:u16,
    pub economy_mode:bool,
    pub network_compression_threshold:i32,
    pub entity_broadcast_range_percentage:u16,
    pub max_tick_time:i64,
    pub op_permission_level:u8,
    pub hide_online_players:bool,
    pub listen_on_network: bool,
    pub online_mode: bool,
    pub keep_inventory: bool,
    pub daylight_cycle: bool,
    pub natural_regeneration: bool,
    pub mob_spawning: bool,
    pub animal_spawning: bool,
    pub monster_spawning: bool,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            motd: "Luxmc Local Server".into(),
            difficulty: "normal".into(),
            gamemode: "survival".into(),
            force_gamemode: true,
            pvp: true,
            whitelist: false,
            view_distance: 6,
            simulation_distance: 4,
            port: 25565,
            autosave_minutes: 5,
            resource_pack_url: String::new(),
            resource_pack_file: String::new(),
            resource_pack_sha1: String::new(),
            require_resource_pack: false,
            seed: None,
            hardcore: false,
            generate_structures: true,
            spawn_protection:16,allow_flight:false,command_blocks:false,idle_timeout:0,economy_mode:true,network_compression_threshold:256,entity_broadcast_range_percentage:100,max_tick_time:60000,op_permission_level:4,hide_online_players:false,
            listen_on_network: false,
            online_mode: true,
            keep_inventory: false,
            daylight_cycle: true,
            natural_regeneration: true,
            mob_spawning: true,
            animal_spawning: true,
            monster_spawning: true,
        }
    }
}

fn property<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim())
        .last()
}

fn escaped(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            character if !character.is_ascii() => {
                let mut bytes = [0; 2];
                for unit in character.encode_utf16(&mut bytes) {
                    output.push_str(&format!("\\u{unit:04x}"));
                }
            }
            character => output.push(character),
        }
    }
    output
}

fn unescaped(value: &str) -> String {
    let mut characters = value.chars();
    let mut units = Vec::new();
    while let Some(character) = characters.next() {
        let character = if character == '\\' {
            match characters.next() {
                Some('u') => {
                    let digits = characters.by_ref().take(4).collect::<String>();
                    if let Ok(unit) = u16::from_str_radix(&digits, 16) {
                        units.push(unit);
                        continue;
                    }
                    for character in format!("\\u{digits}").chars() {
                        let mut buffer = [0; 2];
                        units.extend_from_slice(character.encode_utf16(&mut buffer));
                    }
                    continue;
                }
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some(value) => value,
                None => '\\',
            }
        } else {
            character
        };
        let mut buffer = [0; 2];
        units.extend_from_slice(character.encode_utf16(&mut buffer));
    }
    String::from_utf16_lossy(&units)
}

impl ServerSettings {
    pub fn startup_commands(&self, version: &str) -> Vec<String> {
        let mut commands = vec![format!("defaultgamemode {}", self.gamemode), format!("difficulty {}", self.difficulty)];
        let parts = version.split(['.', '-']).take(3).map(|part| part.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>();
        let snapshot = version.split_once('w').and_then(|(year, week)| Some((year.parse::<u32>().ok()?, week.get(..2)?.parse::<u32>().ok()?)));
        let modern = parts.first().is_some_and(|major| *major >= 26)
            || (parts.first() == Some(&1) && (parts.get(1).is_some_and(|minor| *minor > 21) || (parts.get(1) == Some(&21) && parts.get(2).is_some_and(|patch| *patch >= 9))))
            || snapshot.is_some_and(|(year, week)| year > 25 || (year == 25 && week >= 31));
        let snake = parts.first().is_some_and(|major| *major >= 26)
            || (parts.first() == Some(&1) && parts.get(1) == Some(&21) && parts.get(2).is_some_and(|patch| *patch >= 11))
            || snapshot.is_some_and(|(year, week)| year > 25 || (year == 25 && week >= 44));
        let basic = parts.first().is_some_and(|major| *major >= 26)
            || (parts.first() == Some(&1) && (parts.get(1).is_some_and(|minor| *minor > 6) || (parts.get(1) == Some(&6) && parts.get(2).is_some_and(|patch| *patch >= 1))))
            || snapshot.is_some_and(|(year, week)| year > 13 || (year == 13 && week >= 24));
        if basic {
            if snake {
                commands.push(format!("gamerule keep_inventory {}",self.keep_inventory));
                commands.push(format!("gamerule advance_time {}",self.daylight_cycle));
                commands.push(format!("gamerule natural_health_regeneration {}",self.natural_regeneration));
                commands.push(format!("gamerule spawn_mobs {}",self.mob_spawning));
            } else {
                commands.push(format!("gamerule keepInventory {}",self.keep_inventory));
                commands.push(format!("gamerule doDaylightCycle {}",self.daylight_cycle));
                commands.push(format!("gamerule naturalRegeneration {}",self.natural_regeneration));
                commands.push(format!("gamerule doMobSpawning {}",self.mob_spawning));
            }
        }
        if modern {
            commands.push(format!("gamerule pvp {}",self.pvp));
            if snake {
                commands.push(format!("gamerule command_blocks_work {}",self.command_blocks));
                commands.push(format!("gamerule spawn_monsters {}",self.monster_spawning));
            } else {
                if snapshot.map_or(true, |(_, week)| week >= 37) {
                    commands.push(format!("gamerule commandBlocksEnabled {}",self.command_blocks));
                }
                if snapshot.map_or(true, |(_, week)| week >= 35) {
                    commands.push(format!("gamerule spawnMonsters {}",self.monster_spawning));
                }
            }
        }
        commands
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.listen_on_network && self.port == 0 {
            return Err(invalid(
                "Defina uma porta fixa antes de habilitar o acesso externo.",
            ));
        }
        if !(-1..=1048576).contains(&self.network_compression_threshold) || !(10..=1000).contains(&self.entity_broadcast_range_percentage) || !(-1..=600000).contains(&self.max_tick_time) || self.op_permission_level>4 {return Err(invalid("Ajustes avançados fora do limite permitido."));}

        if self.spawn_protection>64 || self.idle_timeout>120 {return Err(invalid("Use proteção de spawn de 0 a 64 blocos e inatividade de 0 a 120 minutos."));}
        if !self.resource_pack_file.is_empty() && (self.resource_pack_file.len()>180 || self.resource_pack_file.contains(['/', '\\', ':']) || self.resource_pack_file.starts_with('.') || !self.resource_pack_file.to_ascii_lowercase().ends_with(".zip") || self.resource_pack_file.chars().any(char::is_control)) { return Err(invalid("Arquivo de texturas inválido.")); }
        if self.seed.as_ref().is_some_and(|seed| seed.len() > 128 || seed.chars().any(char::is_control)) { return Err(invalid("Seed inválida: use até 128 caracteres sem quebras de linha.")); }
        if self.motd.chars().count() > 256
            || self
                .motd
                .chars()
                .any(|character| character.is_control() && character != '\n')
        {
            return Err(invalid(
                "A descrição deve ter até 256 caracteres, sem controles além de quebra de linha.",
            ));
        }
        if !["peaceful", "easy", "normal", "hard"].contains(&self.difficulty.as_str())
            || !["survival", "creative", "adventure", "spectator"].contains(&self.gamemode.as_str())
        {
            return Err(invalid("Dificuldade ou modo de jogo inválido."));
        }
        if !(2..=32).contains(&self.view_distance)
            || !(2..=32).contains(&self.simulation_distance)
            || self.autosave_minutes > 60
        {
            return Err(invalid(
                "Use distância de 2 a 32 chunks e salvamento de 0 a 60 minutos.",
            ));
        }
        if !self.resource_pack_url.is_empty() {
            let url = url::Url::parse(&self.resource_pack_url)
                .map_err(|_| invalid("Informe uma URL HTTPS pública para o pacote de recursos."))?;
            let managed = !self.resource_pack_file.is_empty() && crate::network::resource_packs::is_managed_url(&self.resource_pack_url,&self.resource_pack_sha1);
            if (url.scheme() != "https" && !managed)
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || self.resource_pack_url.len() > 2048
                || self.resource_pack_url.chars().any(char::is_control)
            {
                return Err(invalid(
                    "Informe uma URL HTTPS válida, sem credenciais, para o pacote de recursos.",
                ));
            }
            if self.resource_pack_sha1.len() != 40
                || !self
                    .resource_pack_sha1
                    .bytes()
                    .all(|value| value.is_ascii_hexdigit())
            {
                return Err(invalid(
                    "Informe o SHA-1 de 40 caracteres do pacote de recursos.",
                ));
            }
        } else if self.resource_pack_file.is_empty() && (self.require_resource_pack || !self.resource_pack_sha1.is_empty()) {
            return Err(invalid(
                "Defina a URL e o hash antes de exigir um pacote de recursos.",
            ));
        }
        Ok(())
    }

    pub fn from_properties(text: &str) -> Self {
        let mut result = Self::default();
        for (name, target) in [
            ("motd", &mut result.motd),
            ("difficulty", &mut result.difficulty),
            ("gamemode", &mut result.gamemode),
            ("resource-pack", &mut result.resource_pack_url),
            ("resource-pack-sha1", &mut result.resource_pack_sha1),
        ] {
            if let Some(value) = property(text, name) {
                *target = unescaped(value);
            }
        }
        result.seed = property(text, "level-seed").map(unescaped);
        result.network_compression_threshold=property(text,"network-compression-threshold").and_then(|value|value.parse().ok()).unwrap_or(256);
        result.entity_broadcast_range_percentage=property(text,"entity-broadcast-range-percentage").and_then(|value|value.parse().ok()).unwrap_or(100);
        result.max_tick_time=property(text,"max-tick-time").and_then(|value|value.parse().ok()).unwrap_or(60000);
        result.op_permission_level=property(text,"op-permission-level").and_then(|value|value.parse().ok()).unwrap_or(4);
        result.hide_online_players=property(text,"hide-online-players").is_some_and(|value|value=="true");
        result.spawn_protection=property(text,"spawn-protection").and_then(|value|value.parse().ok()).unwrap_or(16);
        result.idle_timeout=property(text,"player-idle-timeout").and_then(|value|value.parse().ok()).unwrap_or(0);
        result.difficulty = match result.difficulty.as_str() {
            "0" => "peaceful",
            "1" => "easy",
            "2" => "normal",
            "3" => "hard",
            value => value,
        }
        .into();
        result.gamemode = match result.gamemode.as_str() {
            "0" => "survival",
            "1" => "creative",
            "2" => "adventure",
            "3" => "spectator",
            value => value,
        }
        .into();
        for (name, target) in [
            ("white-list", &mut result.whitelist),
            ("pvp", &mut result.pvp),
            ("force-gamemode", &mut result.force_gamemode),
            ("require-resource-pack", &mut result.require_resource_pack),
            ("hardcore", &mut result.hardcore),
            ("generate-structures", &mut result.generate_structures),
            ("allow-flight", &mut result.allow_flight),("enable-command-block", &mut result.command_blocks),
            ("online-mode", &mut result.online_mode),
        ] {
            if let Some(value) = property(text, name) {
                *target = value.eq_ignore_ascii_case("true");
            }
        }
        if let Some(value) = property(text, "server-port").and_then(|value| value.parse().ok()) {
            result.port = value;
        }
        if let Some(value) = property(text, "view-distance")
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|value| (2..=32).contains(value))
        {
            result.view_distance = value;
        }
        if let Some(value) = property(text, "simulation-distance")
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|value| (2..=32).contains(value))
        {
            result.simulation_distance = value;
        }
        result
    }

    pub fn apply(&self, previous: &str) -> String {
        let mut replacements = vec![
            ("motd", escaped(&self.motd)),
            ("network-compression-threshold",self.network_compression_threshold.to_string()),
            ("entity-broadcast-range-percentage",self.entity_broadcast_range_percentage.to_string()),
            ("max-tick-time",self.max_tick_time.to_string()),
            ("op-permission-level",self.op_permission_level.to_string()),
            ("hide-online-players",self.hide_online_players.to_string()),
            ("difficulty", self.difficulty.clone()),
            ("gamemode", self.gamemode.clone()),
            ("hardcore", self.hardcore.to_string()),
            ("generate-structures", self.generate_structures.to_string()),
            ("spawn-protection",self.spawn_protection.to_string()),("allow-flight",self.allow_flight.to_string()),("enable-command-block",self.command_blocks.to_string()),("player-idle-timeout",self.idle_timeout.to_string()),
            ("force-gamemode", self.force_gamemode.to_string()),
            ("pause-when-empty-seconds", if self.economy_mode {"60"} else {"0"}.into()),
            ("pvp", self.pvp.to_string()),
            ("online-mode", self.online_mode.to_string()),
            ("white-list", self.whitelist.to_string()),
            ("enforce-whitelist", self.whitelist.to_string()),
            ("view-distance", self.view_distance.to_string()),
            ("simulation-distance", self.simulation_distance.to_string()),
            ("resource-pack", escaped(&self.resource_pack_url)),
            (
                "resource-pack-sha1",
                self.resource_pack_sha1.to_ascii_lowercase(),
            ),
            (
                "require-resource-pack",
                self.require_resource_pack.to_string(),
            ),
        ];
        if let Some(seed) = &self.seed { replacements.push(("level-seed", escaped(seed))); }
        let mut result = previous
            .lines()
            .filter(|line| {
                !replacements.iter().any(|(key, _)| {
                    line.trim_start().split(['=', ':', ' ', '\t']).next() == Some(key)
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
        for (key, value) in replacements {
            result.push_str(&format!("\n{key}={value}"));
        }
        result.push('\n');
        result
    }
}

impl LocalServerManager {
    pub async fn include_owner(&self, id: &str, uuid: &str, name: &str) -> AppResult<()> {
        if uuid::Uuid::parse_str(uuid).is_err()
            || name.is_empty()
            || name.len() > 16
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(invalid("Identidade Minecraft inválida."));
        }
        let _guard = self.change()?;
        if self.current(id).is_ok() {
            return Err(unavailable("O servidor já está em execução."));
        }
        let directory = self.directory(id).await?;
        let previous = self.read(&directory, "whitelist.json").await?;
        let mut entries: Vec<serde_json::Value> = previous
            .as_ref()
            .map(|bytes| serde_json::from_slice(bytes))
            .transpose()?
            .unwrap_or_default();
        if entries.len() > 1024 {
            return Err(invalid("A whitelist excede o limite de edição automática. Abra a pasta do servidor para conferir."));
        }
        entries.retain(|entry| {
            entry
                .get("uuid")
                .and_then(|value| value.as_str())
                .and_then(|value| uuid::Uuid::parse_str(value).ok())
                != uuid::Uuid::parse_str(uuid).ok()
        });
        entries.push(serde_json::json!({"uuid":uuid,"name":name}));
        if let Some(bytes) = previous {
            crate::core::mods::pack_download::atomic_write(
                &directory.join("whitelist.previous.json"),
                &bytes,
            )
            .await?;
        }
        crate::core::mods::pack_download::atomic_write(
            &directory.join("whitelist.json"),
            &serde_json::to_vec_pretty(&entries)?,
        )
        .await?;
        Ok(())
    }

    pub async fn settings(&self, id: &str) -> AppResult<ServerSettings> {
        let directory = self.directory(id).await?;
        let properties = self
            .read(&directory, "server.properties")
            .await?
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default();
        let imported = ServerSettings::from_properties(&properties);
        if let Some(bytes) = self.read(&directory, "luxmc-settings.json").await? {
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let mut settings: ServerSettings = serde_json::from_value(value.clone())?;
            if value.get("networkCompressionThreshold").is_none(){settings.network_compression_threshold=imported.network_compression_threshold;}
            if value.get("entityBroadcastRangePercentage").is_none(){settings.entity_broadcast_range_percentage=imported.entity_broadcast_range_percentage;}
            if value.get("maxTickTime").is_none(){settings.max_tick_time=imported.max_tick_time;}
            if value.get("opPermissionLevel").is_none(){settings.op_permission_level=imported.op_permission_level;}
            if value.get("hideOnlinePlayers").is_none(){settings.hide_online_players=imported.hide_online_players;}
            if value.get("seed").is_none() { settings.seed = imported.seed; }
            if value.get("hardcore").is_none() { settings.hardcore = imported.hardcore; }
            if value.get("generateStructures").is_none() { settings.generate_structures = imported.generate_structures; }
            if value.get("spawnProtection").is_none() {settings.spawn_protection=imported.spawn_protection;}
            if value.get("idleTimeout").is_none() {settings.idle_timeout=imported.idle_timeout;}
            if value.get("allowFlight").is_none() {settings.allow_flight=imported.allow_flight;}
            if value.get("commandBlocks").is_none() {settings.command_blocks=imported.command_blocks;}
            return Ok(settings);
        }
        Ok(imported)
    }

    pub async fn update_settings(
        &self,
        id: &str,
        settings: ServerSettings,
    ) -> AppResult<ServerSettings> {
        settings.validate()?;
        let _guard = self.change()?;
        let directory = self.directory(id).await?;
        if let Some(previous) = self.read(&directory, "luxmc-settings.json").await? {
            crate::core::mods::pack_download::atomic_write(
                &directory.join("luxmc-settings.previous.json"),
                &previous,
            )
            .await?;
        }
        crate::core::mods::pack_download::atomic_write(
            &directory.join("luxmc-settings.json"),
            &serde_json::to_vec_pretty(&settings)?,
        )
        .await?;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advanced_network_and_watchdog_settings_preserve_imported_choices_and_roundtrip() {
        let input="network-compression-threshold=1024\nentity-broadcast-range-percentage=75\nmax-tick-time=-1\nop-permission-level=2\nhide-online-players=true\nlevel-name=world\n";
        let mut settings=ServerSettings::from_properties(input);settings.validate().unwrap();
        assert_eq!(settings.max_tick_time,-1);assert_eq!(settings.entity_broadcast_range_percentage,75);assert!(settings.hide_online_players);
        let output=settings.apply(input);assert!(output.contains("level-name=world"));assert!(output.contains("max-tick-time=-1"));assert_eq!(output.matches("network-compression-threshold=").count(),1);
        assert_eq!(ServerSettings::from_properties(&output).op_permission_level,2);
        settings.op_permission_level=5;assert!(settings.validate().is_err());
    }

    #[test]
    fn external_listening_is_opt_in_and_requires_a_fixed_port() {
        let defaults = ServerSettings::default();
        assert!(!defaults.listen_on_network);
        assert!(!defaults.validate().is_err());

        let legacy: ServerSettings = serde_json::from_str(r#"{"port":25565}"#).unwrap();
        assert!(!legacy.listen_on_network);

        let mut external = defaults;
        external.port = 0;
        external.listen_on_network = true;
        assert!(external.validate().is_err());
        external.port = 25565;
        assert!(external.validate().is_ok());
        let encoded = serde_json::to_string(&external).unwrap();
        let restored: ServerSettings = serde_json::from_str(&encoded).unwrap();
        assert!(restored.listen_on_network);
        assert_eq!(restored.port, 25565);
    }

    #[test]
    fn pvp_is_applied_as_a_world_rule_only_on_versions_that_support_it() {
        let mut settings = ServerSettings::default(); settings.pvp = false;
        for version in ["1.21.9", "1.21.11", "26.3", "25w31a"] { assert!(settings.startup_commands(version).contains(&"gamerule pvp false".into())); }
        for version in ["1.12.2", "1.21.1", "25w30a"] { assert!(!settings.startup_commands(version).iter().any(|command| command.starts_with("gamerule pvp"))); }
    }

    #[test]
    fn gamerule_names_follow_the_version_registry() {
        let mut settings = ServerSettings::default();
        settings.keep_inventory = true;
        settings.daylight_cycle = false;
        settings.natural_regeneration = false;
        settings.mob_spawning = false;
        settings.monster_spawning = false;
        settings.command_blocks = false;
        for version in ["1.21.11", "26.3", "26.1", "25w44a", "26w01a"] {
            let commands = settings.startup_commands(version);
            for expected in ["gamerule keep_inventory true", "gamerule advance_time false", "gamerule natural_health_regeneration false", "gamerule spawn_mobs false", "gamerule spawn_monsters false"] {
                assert!(commands.contains(&expected.into()), "{version} should emit {expected}, got {commands:?}");
            }
            assert!(!commands.iter().any(|command| command.contains("doAnimalSpawning")), "{version} must not emit a rule that does not exist in Java");
        }
        for version in ["1.21.9", "1.21.10", "25w35a", "25w31a"] {
            let commands = settings.startup_commands(version);
            for expected in ["gamerule keepInventory true", "gamerule doDaylightCycle false", "gamerule naturalRegeneration false", "gamerule doMobSpawning false"] {
                assert!(commands.contains(&expected.into()), "{version} should emit {expected}, got {commands:?}");
            }
            assert!(!commands.iter().any(|command| command.contains("doAnimalSpawning")), "{version} must not emit a rule that does not exist in Java");
        }
        assert!(settings.startup_commands("1.21.9").contains(&"gamerule spawnMonsters false".into()));
        assert!(!settings.startup_commands("25w31a").iter().any(|command| command.contains("spawnMonsters")));
        for version in ["1.12.2", "1.21.8", "25w30a"] {
            let commands = settings.startup_commands(version);
            for expected in ["gamerule keepInventory true", "gamerule doDaylightCycle false", "gamerule naturalRegeneration false", "gamerule doMobSpawning false"] {
                assert!(commands.contains(&expected.into()), "{version} should emit {expected}, got {commands:?}");
            }
            assert!(!commands.iter().any(|command| command.contains("pvp") || command.contains("Monsters") || command.contains("commandBlock") || command.contains("command_blocks")), "{version} must not emit rules that do not exist yet");
        }
        for version in ["1.5.2", "1.4.7"] {
            assert!(!settings.startup_commands(version).iter().any(|command| command.starts_with("gamerule")), "{version} predates the basic gamerules used here");
        }
        assert!(settings.startup_commands("26.3").contains(&"gamerule command_blocks_work false".into()));
        assert!(settings.startup_commands("1.21.9").contains(&"gamerule commandBlocksEnabled false".into()));
    }

    #[test]
    fn changing_mode_preserves_world_and_empty_pause_follows_economy_choice() {
        let mut settings = ServerSettings::default();
        settings.gamemode = "creative".into();
        settings.simulation_distance = 12;
        let result = settings.apply("gamemode=survival\nforce-gamemode=false\npause-when-empty-seconds=60\nsimulation-distance=6\nlevel-name=world\n");
        assert_eq!(property(&result,"gamemode"),Some("creative"));
        assert_eq!(property(&result,"force-gamemode"),Some("true"));
        assert_eq!(property(&result,"pause-when-empty-seconds"),Some("60"));
        settings.economy_mode = false;
        assert_eq!(property(&settings.apply(&result),"pause-when-empty-seconds"),Some("0"));
        assert_eq!(property(&result,"simulation-distance"),Some("12"));
        assert_eq!(property(&result,"level-name"),Some("world"));
    }

    #[test]
    fn new_servers_are_open_and_existing_whitelist_is_imported_explicitly() {
        assert!(!ServerSettings::default().whitelist);
        let imported =
            ServerSettings::from_properties("white-list=true\nserver-port=41234\nlevel-seed=42\n");
        assert!(imported.whitelist);
        assert_eq!(imported.port, 41234);
        let open = ServerSettings::default()
            .apply("white-list=true\nenforce-whitelist=true\nlevel-seed=42\n");
        assert!(open.contains("white-list=false") && open.contains("enforce-whitelist=false"));
        assert!(!open.contains("white-list=true"));
        assert!(open.contains("level-seed=42"));
    }

    #[test]
    fn properties_cannot_inject_commands_or_switch_off_authentication() {
        let mut settings = ServerSettings::default();
        settings.motd = "Hello\nonline-mode=false".into();
        assert!(settings.validate().is_ok());
        let encoded = settings.apply("online-mode=true\n");
        assert!(!encoded.lines().any(|line| line == "online-mode=false"));
        assert!(encoded.contains("motd=Hello\\nonline-mode=false"));
        settings.motd = "Olá = mundo".into();
        assert!(settings.validate().is_ok());
        assert!(settings
            .apply("online-mode=true\n")
            .contains("online-mode=true"));
        settings.resource_pack_url = "file:///secret.zip".into();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn offline_mode_round_trips_and_defaults_to_original_accounts() {
        assert!(ServerSettings::default().online_mode);
        let encoded = ServerSettings::default().apply("online-mode=true
");
        assert!(encoded.lines().any(|line| line == "online-mode=true"));
        let mut offline = ServerSettings::default();
        offline.online_mode = false;
        let encoded = offline.apply(&encoded);
        assert_eq!(encoded.lines().filter(|line| line.starts_with("online-mode=")).count(), 1);
        assert!(encoded.lines().any(|line| line == "online-mode=false"));
        assert!(!ServerSettings::from_properties(&encoded).online_mode);
    }

    #[tokio::test]
    async fn settings_changes_preserve_existing_world_and_keep_a_previous_copy() {
        let root = tempfile::tempdir().unwrap();
        let manager = LocalServerManager::new(root.path().into(), reqwest::Client::new());
        let id = uuid::Uuid::new_v4().to_string();
        let folder = root.path().join(&id);
        tokio::fs::create_dir(&folder).await.unwrap();
        tokio::fs::write(
            folder.join("server.properties"),
            "white-list=true\nserver-port=41234\n",
        )
        .await
        .unwrap();
        tokio::fs::write(folder.join("world-marker"), b"keep")
            .await
            .unwrap();
        assert!(manager.settings(&id).await.unwrap().whitelist);
        let open = ServerSettings::default();
        manager.update_settings(&id, open.clone()).await.unwrap();
        manager.update_settings(&id, open.clone()).await.unwrap();
        assert_eq!(manager.settings(&id).await.unwrap(), open);
        assert!(folder.join("luxmc-settings.previous.json").exists());
        assert_eq!(
            tokio::fs::read(folder.join("world-marker")).await.unwrap(),
            b"keep"
        );
    }

    #[test]
    fn java_properties_import_preserves_unicode_escaped_urls_and_legacy_numeric_modes() {
        assert_eq!(unescaped(r"Ol\u00e1 \ud83d\ude00"), "Olá 😀");
        let text = format!("motd={}\ndifficulty=3\ngamemode=1\nresource-pack=https\\://example.org/pack.zip\nresource-pack-sha1={}\n", r"Ol\u00e1", "a".repeat(40));
        let settings = ServerSettings::from_properties(&text);
        assert_eq!(settings.motd, "Olá");
        assert_eq!(settings.difficulty, "hard");
        assert_eq!(settings.gamemode, "creative");
        assert_eq!(settings.resource_pack_url, "https://example.org/pack.zip");
        assert!(settings.validate().is_ok());
        assert_eq!(
            ServerSettings::from_properties(&settings.apply("online-mode=true\n")).motd,
            settings.motd
        );
    }
}
