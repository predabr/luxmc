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
			version: "1.3.0-beta".into(),
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
		ChangelogEntry {
			version: "1.3.1-alpha".into(),
			date: "2026-09-12".into(),
			title: "Correções Críticas Windows/Linux, CurseForge Edge CDN, Microsoft OAuth & Estabilidade de RAM".into(),
			highlights: vec![
				"Isolamento completo do ambiente Linux AppImage (LD_LIBRARY_PATH_ORIG) evitando colisão de drivers gráficos na inicialização do Minecraft e modpacks".into(),
				"Extração in-process nativa de natives e pacotes via Rust zip::ZipArchive eliminando a dependência do executável 'unzip'".into(),
				"Edge CDN Fallback para a API do CurseForge contornando erros 403 em mods com bloqueio de distribuição de terceiros".into(),
				"Autenticação Microsoft Azure conectada ao Tenant oficial (78c2f825-9437-484a-aaa6-4d61631e106b), corrigindo 'unauthorized_client'".into(),
				"Otimização de RAM no sistema e JVM: inicialização leve de heap (-Xms) dinâmico e downloads de skins via Blob URLs no WebKit".into(),
				"Correção de geometria, física e posicionamento das capas 3D no visualizador com caimento dinâmico correto".into(),
				"Proteção contra Zip-Slip e sanitização de Path Traversal em downloads temporários e importação de instâncias".into(),
				"Auto-recuperação de versões do Minecraft direto do manifesto oficial caso não existam no banco de dados local".into(),
			],
		},
		ChangelogEntry {
			version: "1.5.0-beta".into(),
			date: "2026-09-12".into(),
			title: "Atualizador Automático In-App Nativo, Suporte Global a Modpacks & Performance Estável".into(),
			highlights: vec![
				"Atualizador automático embutido: download com streaming, barra de progresso em tempo real, substituição atômica de binário e reinicialização instantânea".into(),
				"Correção definitiva de importação de modpacks (BetterMC, Cobblemon, Pixelmon) com download paralelo otimizado e tratamento de arquivos ausentes".into(),
				"Injeção correta de skins e capas customizadas em instâncias offline e online".into(),
				"Ajustes de parâmetros de memória e GC (G1GC com ZGC fallback) para evitar travamentos e picos de RAM".into(),
				"Exibição correta de capas e nomes de instâncias criadas a partir de modpacks na biblioteca".into(),
				"Pipeline de release automatizado com suporte a Linux (AppImage, deb, rpm), Windows (exe, msi) e macOS (dmg)".into(),
			],
		},
		ChangelogEntry {
			version: "1.5.1-beta".into(),
			date: "2026-09-12".into(),
			title: "Correções Críticas: GLFW Wayland, NeoForge WGL OpenGL, Sessão Permanente e RAM de Servidores".into(),
			highlights: vec![
				"Correção de crash GLFW 65548 no Linux AppImage em Wayland através de compatibilidade transparente XWayland".into(),
				"Correção de crash WGL OpenGL em modpacks NeoForge e Forge no Windows desativando early display conflitante".into(),
				"Sessão Microsoft permanente: refresh transparente de token em segundo plano sem expiração ou deslogamento automático".into(),
				"Otimização da aba de servidores: sockets assíncronos não-bloqueantes com cache TTL de 60s, eliminando consumo excessivo de RAM".into(),
				"Aplicação aprimorada de skins customizadas para todas as versões do Minecraft com suporte a formatos legados e modernos".into(),
				"Remoção de campos legados e simplificação da configuração de mods e integrações".into(),
			],
		},
		ChangelogEntry {
			version: "1.5.2-beta".into(),
			date: "2026-09-12".into(),
			title: "Estabilidade Completa: GLFW Wayland, NeoForge Early Display, Injeção Total de Skins e Otimização de Servidores".into(),
			highlights: vec![
				"Correção de compilação e caminhos de assets embutidos no empacotamento nativo do launcher".into(),
				"Eliminação de consumo de RAM na lista de servidores via semáforo de concorrência e buffers de 64KB".into(),
				"Cancelamento imediato de consultas concorrentes na troca de páginas e filtros de servidores".into(),
				"Interface de mods limpa e profissional com remoção de alertas intrusivos de chave de API".into(),
				"Sincronização ponta-a-ponta de skins personalizadas em todas as rotas de inicialização de jogo".into(),
			],
		},
		ChangelogEntry {
			version: "1.5.3-beta".into(),
			date: "2026-09-12".into(),
			title: "CurseForge Direto, Passthrough de Skins Mojang Oficiais, Aparência em Tempo Real & Performance Instantânea".into(),
			highlights: vec![
				"Resolução direta e download de mods e dependências do CurseForge com fallback automático Edge CDN oficial da Forge".into(),
				"Suporte nativo a skins de contas oficiais Mojang/Microsoft: carregamento autêntico direto via sessionserver sem interferência de pacotes locais".into(),
				"Upload multipart direto para api.minecraftservices.com ao trocar skin no launcher em contas oficiais".into(),
				"Otimização da navegação entre abas: remoção de destruição pesada de DOM e transições bloqueantes para troca instantânea de páginas".into(),
				"Ajustes de aparência e temas (Dark/Light e tons neon) aplicados imediatamente em tempo real em todos os elementos da interface".into(),
				"Redução estrita de cache em memória na busca e ping de servidores multiplayer, eliminando vazamento de RAM".into(),
			],
		},
		ChangelogEntry {
			version: "1.5.4-beta".into(),
			date: "2026-09-12".into(),
			title: "Correção de Modpacks CurseForge, Abertura de Pastas no AppImage, Galeria de Screenshots & Novo Organizador de Layout".into(),
			highlights: vec![
				"Correção de inicialização imediata em modpacks do CurseForge: preservação de versão e loader reais do manifest e integridade do classpath".into(),
				"Limpeza de variáveis de ambiente do AppImage evitando erros de saída 4 (unix_wait_status 1024) no xdg-open e navegadores".into(),
				"Novo módulo Organizador de Layout: personalização completa e reordenação drag-and-drop de seções da Home com persistência local".into(),
				"Correção de miniaturas e visualização de screenshots com suporte a streaming de assets locais".into(),
				"Remoção completa de código legado de servidores para máxima otimização e estabilidade de memória".into(),
			],
		},
	])
}
