use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub title: String,
    pub highlights: Vec<String>,
}

#[tauri::command]
pub async fn changelog_get() -> AppResult<Vec<ChangelogEntry>> {
    Ok(vec![
		ChangelogEntry {
			version: "0.2.0".into(),
			date: "14 Oct 2024".into(),
			title: "Initial Linux launcher".into(),
			highlights: vec![
				"Multi-instance profiles with custom game directories".into(),
				"Live Mojang version catalog with full release / snapshot / old_beta / old_alpha support".into(),
				"Automatic Java runtime provisioning (Java 8 → 25)".into(),
				"Tauri 2 desktop app with Svelte 5 UI".into(),
				"Microsoft, Dev and Offline authentication modes".into(),
			],
		},
		ChangelogEntry {
			version: "0.2.0".into(),
			date: "2026-09-07".into(),
			title: "Linux polish & crash safety".into(),
			highlights: vec![
				"OS-aware JVM argument filtering (no more -XstartOnFirstThread or Windows heap-dump path on Linux)".into(),
				"Per-instance JVM args editor with safe validation per OS".into(),
				"Crash summary classifier for narrator / native / auth / memory / Java version / port errors".into(),
				"Per-instance saves backup and restore to .zip".into(),
				"Full instance export and import as .zip for sharing setups".into(),
				"Repair action that re-downloads client + libraries + natives".into(),
				"Persistent launch log history with search by level (INFO/WARN/ERROR)".into(),
				"Account status indicator: online / expiring / expired / offline".into(),
			],
		},
		ChangelogEntry {
			version: "1.1.0-beta".into(),
			date: "2026-09-10".into(),
			title: "Otimização Inteligente & Suporte Completo a Mods".into(),
			highlights: vec![
				"Sistema de Otimização Inteligente Luxmc baseado nas Aikar's Flags (G1GC) com dimensionamento dinâmico de regiões de heap e threads por RAM".into(),
				"Pacote de Performance Essencial (Sodium, Lithium, FerriteCore) com instalação em 1-clique e resolução oficial via Modrinth API".into(),
				"Aceleração gráfica Mesa Zink / Vulkan nativa no Linux para GPUs AMD e Intel via driver Gallium Zink".into(),
				"Detecção de hardware no Linux via subsistema DRM (/sys/class/drm) com identificação precisa de placa gráfica e aceleração".into(),
				"Correção na injeção de parâmetros JVM: flags dinâmicas respeitando a memória configurada sem duplicatas ou bloqueios".into(),
				"Suporte estável aos mod loaders Fabric, Forge, NeoForge e Quilt com verificação de integridade e carregamento em jogo comprovado".into(),
				"Refinamento visual com tema luxury dark (#141518 / #caa97c) em todas as telas e painéis do launcher".into(),
			],
		},
		ChangelogEntry {
			version: "1.2.0-alpha".into(),
			date: "2026-09-10".into(),
			title: "Discord RPC Dinâmico, Otimização Extrema & Redesign SKlauncher".into(),
			highlights: vec![
				"Discord Rich Presence dinâmico no launcher acompanhando todas as telas (Início, Instâncias, Mods, Skins, Servidores) e exibição detalhada de jogo".into(),
				"Correção de compatibilidade com arRPC e Vesktop no Linux eliminando erros 404 de Client ID".into(),
				"Otimização profunda de consumo de memória: eliminação de passes secundários de renderização 3D no fundo e limites estritos de log e imagens".into(),
				"Sistema de botões redesenhado no padrão SKlauncher com gradientes champagne, inner highlights e animações de spring nos cliques".into(),
				"Personalizador de skins 3D ultra-nítido com NearestFilter, fim do blur de pixel art e controles de zoom direto na tela".into(),
				"Barra de downloads flutuante em vidro fumê com indicador de velocidade real em MB/s e tempo restante estimado".into(),
				"Segurança máxima: ofuscação de chaves CurseForge em tempo de compilação via script Rust sem segredos em texto claro no binário".into(),
			],
		},
	])
}
