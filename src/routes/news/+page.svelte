<script lang="ts">
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import ChangelogPanel from "$lib/components/ui/ChangelogPanel.svelte";
	import { 
		Newspaper, 
		Sparkles, 
		ExternalLink, 
		Radio, 
		Tag, 
		Calendar, 
		Flame, 
		ShieldCheck, 
		Cpu, 
		Globe, 
		CheckCircle2,
		Sliders
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";

	type LauncherArticle = {
		id: string;
		title: string;
		tag: string;
		tagColor: string;
		date: string;
		summary: string;
		highlights: string[];
		version?: string;
		link?: string;
		image: string;
	};

	const officialNews: LauncherArticle[] = [
		{
			id: "mc-1-21-5-pale-garden",
			title: "Minecraft Drop Oficial: The Pale Garden & The Creaking",
			tag: "Minecraft Oficial",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "Setembro, 2026",
			image: "https://images.unsplash.com/photo-1518709268805-4e9042af9f23?q=80&w=800&auto=format&fit=crop",
			summary: "A Mojang Studios revelou o aguardado drop oficial com o misterioso bioma Pale Garden, o aterrorizante Creaking que só se move quando você não está olhando, e novos blocos de madeira e resina.",
			highlights: [
				"Novo bioma Pale Garden: atmosfera cinzenta e silenciosa onde as copas das árvores bloqueiam a luz",
				"Mob The Creaking: imune a ataques frontais enquanto mantiver contato visual; destrua o Coração do Creaking na árvore para derrotá-lo",
				"Nova madeira Pale Oak (Carvalho Pálido) com conjunto completo de tábuas, cercas, portas e barcos",
				"Novo recurso: Resina encontrada nas árvores do Pale Garden para fabricação de tijolos e ornamentos",
				"Compatibilidade nativa imediata no Luxmc para snapshots e lançamentos oficiais"
			],
			link: "https://www.minecraft.net"
		},
		{
			id: "modrinth-api-v3",
			title: "Modrinth API v3 & Novos Servidores de CDN na América do Sul",
			tag: "Mods & Comunidade",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "Setembro, 2026",
			image: "https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=800&auto=format&fit=crop",
			summary: "A plataforma Modrinth expandiu sua rede de borda com servidores dedicados em São Paulo (GRU), garantindo downloads de modpacks e shaders até 4 vezes mais rápidos no Luxmc.",
			highlights: [
				"Baixa latência para jogadores do Brasil e América Latina",
				"Pesquisa instantânea de Shaders e Texturas pelo catálogo integrado",
				"Distribuição automatizada de arquivos .mrpack com verificação de integridade SHA-512",
				"Catálogo com mais de 80.000 modificações atualizadas diariamente"
			],
			link: "https://modrinth.com"
		},
		{
			id: "sodium-iris-vulkan",
			title: "Sodium 0.6 & Iris Shaders: Salto Gráfico com Multi-Draw no Linux",
			tag: "Desempenho & Gráficos",
			tagColor: "text-cyan-400 bg-cyan-500/10 border-cyan-500/30",
			date: "Setembro, 2026",
			image: "https://images.unsplash.com/photo-1550745165-9bc0b252726f?q=80&w=800&auto=format&fit=crop",
			summary: "O motor de renderização Sodium e o carregador Iris Shaders receberam otimizações profundas de OpenGL Multi-Draw e Zink Vulkan, alcançando 144+ FPS estáveis mesmo em mundos pesados.",
			highlights: [
				"Pipeline gráfico reconstruído para placas AMD, Intel e Nvidia",
				"Redução de até 60% na carga de CPU durante a renderização de chunks distantes",
				"Compatibilidade total com pacotes famosos como Complimentary Reimagined e BSL",
				"Renderização fluida em monitores de alta taxa de atualização (144Hz a 360Hz)"
			],
			link: "https://modrinth.com/mod/sodium"
		},
		{
			id: "v1.9.2",
			title: "Luxmc v1.9.2 — Pacotes .RPM, Correções de Modpacks e Personalização Total",
			tag: "Oficial",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "20 de Setembro, 2026",
			version: "v1.9.2",
			image: "/news_1.jpg",
			summary: "Atualização v1.9.2 com suporte nativo a pacotes .rpm (Fedora/RHEL/openSUSE), correção de inicialização de modpacks pesados, skins 3D pixel-perfect e importação de wallpapers animados.",
			highlights: [
				"Atualizador automático oficial do Tauri 2 com verificação em background",
				"Interface com layout fluído e personalizador de ponta a ponta",
				"Controle estrito de memória e zero vazamentos de RAM",
				"Suporte aprimorado a instâncias, capas e modpacks pesados",
				"Transições suaves entre abas e renderização acelerada por hardware"
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.7",
			title: "Luxmc v1.7.7 — Capas In-Game, Resolução Inteligente ATM10 & Shift Direito em Mods",
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "20 de Setembro, 2026",
			version: "v1.7.7",
			image: "/news_1.jpg",
			summary: "Correção definitiva da colisão de dependências do modpack All The Mods 10 (ATM10), persistência de capas customizadas in-game e suporte a Shift Direito em instâncias com mods.",
			highlights: [
				"Resolução inteligente de mods com prioridade a overrides e semver para ATM10",
				"Persistência de capas do launcher (Enderman Minecon, Luxmc) in-game para contas MSA",
				"Menu in-game com Shift Direito habilitado em instâncias modded (NeoForge, Forge, Fabric, Quilt)",
				"Debounce atômico de 500ms prevenindo saída acidental do modo tela cheia",
				"Correção de miniaturas quebradas na visualização de mods da instância"
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.6",
			title: "Luxmc v1.7.6 — Barra Lateral Translúcida, Suporte NeoForge ATM10 & Zero GLFW Crash",
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",

			date: "20 de Setembro, 2026",
			version: "v1.7.6",
			image: "/news_1.jpg",
			summary: "Atualização focada na sincronização estética da barra lateral com os temas e wallpapers dinâmicos, aceleração OpenGL WGL para modpacks NeoForge como All The Mods 11 e estabilidade total no Linux Wayland.",
			highlights: [
				"Barra lateral com transparência adaptativa e desfoque ultra suave (backdrop-blur-2xl)",
				"Resolução da falha de driver OpenGL WGL no Windows para modpacks NeoForge",
				"Eliminação de travamentos de ícones GLFW no Linux Wayland nativo",
				"Renovação perpétua e invisível de tokens Microsoft",
				"Consumo reduzido para menos de 85 MB de RAM em repouso"
			],
			link: "https://luxmc-r92.pages.dev/#destaques"
		},
		{
			id: "v1.7.5",
			title: "Luxmc v1.7.5 — GPU Acceleration, Suporte ao Espanhol & Modo Streamer",
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "19 de Setembro, 2026",
			version: "v1.7.5",
			image: "/news_1.jpg",
			summary: "Nova versão trazendo aceleração de hardware nativa no WebKit, clique integral nos cards da biblioteca, novo idioma Espanhol (Español), painel de privacidade completo e capas de modpack dinâmicas.",
			highlights: [
				"Composição por GPU ativada com DMA-BUF / EGL sem travamentos ou lag de interface",
				"Biblioteca responsiva: clique em qualquer lugar do card para abrir a instância",
				"Novo idioma Espanhol (Español) integrado em toda a interface",
				"Painel de Privacidade avançado com Modo Streamer e discrição no Discord Rich Presence",
				"Tratamento automático de capas e banners para todos os modpacks CurseForge e Modrinth",
				"Adicionar amigo simplificado com auto-busca e salvamento instantâneo"
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.4",
			title: "Luxmc v1.7.4 — Correção Definitiva de Modpacks & CDN Resiliente",
			tag: "Oficial",
			tagColor: "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
			date: "18 de Setembro, 2026",
			version: "v1.7.4",
			image: "/news_1.jpg",
			summary: "Atualização com foco dedicado na estabilidade total do CurseForge e Modrinth, eliminação do bug de exclusão de mods, importação nativa de .mrpack e reparo automático no lançamento.",
			highlights: [
				"Correção definitiva da importação de modpacks CurseForge com preservação de 100% dos arquivos .jar",
				"Download concorrente com pool de 6 workers e espelhos de CDN resilientes (Edge, Mediafilez)",
				"Importação completa de modpacks Modrinth (.mrpack) com extração integral de overrides",
				"Detecção inteligente de versão e arquitetura entre NeoForge e Forge clássico",
				"Reparo automático de mods faltantes antes do lançamento da instância"
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.3",
			title: "Luxmc v1.7.3 — Obsidian Performance & Paridade Total",
			tag: "Oficial",
			tagColor: "text-blue-400 bg-blue-500/10 border-blue-500/30",
			date: "18 de Setembro, 2026",
			version: "v1.7.3",
			image: "/news_1.jpg",
			summary: "Atualização essencial focada na estabilidade da inicialização, paridade completa com Wayland/X11, novo sistema de amizades P2P com aceitação de convites em tempo real e visual de alto contraste.",
			highlights: [
				"Correção de inicialização direta do Minecraft Vanilla e Modpacks com tratamento de resolução adaptativa",
				"Novo Hub de Amigos com gerenciamento de convites, aceitar/recusar solicitações e Direct Join UPnP",
				"Renderização 3D de skins sem artefatos ou blocos pretos nas capas e texturas de braço",
				"Botões de ação da instância redesenhados com contraste elevado e badges luminosos",
				"Integração bidirecional direta com o site oficial e Studio 3D (luxmc-r92.pages.dev)"
			],
			link: "https://luxmc-r92.pages.dev/#releases"
		},
		{
			id: "v1.7.2",
			title: "Lançamento do Portal Web e Studio 3D no Cloudflare Pages",
			tag: "Ecossistema",
			tagColor: "text-sky-400 bg-sky-500/10 border-sky-500/30",
			date: "17 de Setembro, 2026",
			version: "v1.7.2",
			image: "/news_2.jpg",
			summary: "O Luxmc agora possui um portal web moderno em luxmc-r92.pages.dev com estúdio tridimensional de skins, catálogo de modificações e downloads rápidos para Linux e Windows.",
			highlights: [
				"Visualizador e customizador 3D em tempo real na nuvem",
				"Sincronização de skins do launcher com o portal",
				"Bento grid interativo com métricas e benchmarks de memória RAM",
				"Deploy global ultrarrápido na borda da Cloudflare"
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v1.7.0",
			title: "Motor Zero-Lag e Hospedagem P2P com UPnP Nativo",
			tag: "Recurso",
			tagColor: "text-purple-400 bg-purple-500/10 border-purple-500/30",
			date: "14 de Setembro, 2026",
			version: "v1.7.0",
			image: "/news_3.jpg",
			summary: "Jogue com seus amigos em qualquer mundo de Minecraft sem precisar de portas manuais no roteador, Hamachi ou programas de terceiros.",
			highlights: [
				"Mapeamento dinâmico de portas UPnP residencial com zero configuração",
				"Compartilhamento de link próprio de conexão direta luxmc://host/...",
				"Radar de amigos em segundo plano com Ghost Ping de baixa latência"
			],
			link: "https://luxmc-r92.pages.dev"
		},
		{
			id: "v1.6.5",
			title: "Otimização Aikar's Flags e Ghost Mode de Memória RAM",
			tag: "Otimização",
			tagColor: "text-amber-400 bg-amber-500/10 border-amber-500/30",
			date: "10 de Setembro, 2026",
			version: "v1.6.5",
			image: "/news_4.jpg",
			summary: "Inclusão de rotinas automáticas de redução de consumo de memória. O Luxmc libera memória ociosa assim que o jogo é lançado, garantindo FPS máximo.",
			highlights: [
				"Chamada de malloc_trim e liberação de cache em background",
				"Aikar's Flags calculadas de acordo com a quantidade de RAM da sua máquina",
				"Integração com GameMode do Linux para priorização de núcleos de CPU"
			],
			link: "https://luxmc-r92.pages.dev"
		}
	];

	let selectedTab = $state<"news" | "changelog">("news");

	async function openWebsite(url?: string) {
		const target = url || "https://luxmc-r92.pages.dev";
		try {
			await openUrl(target);
		} catch {
			window.open(target, "_blank");
		}
		toast("Abrindo no navegador...", "info");
	}
</script>

<div class="mx-auto flex h-full max-w-5xl flex-col gap-6 select-none overflow-y-auto custom-scrollbar pb-12">
	<!-- Header -->
	<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-fg/10 pb-5">
		<div class="flex items-center gap-3">
			<div class="p-2.5 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">
				<Newspaper class="h-6 w-6" />
			</div>
			<div>
				<h1 class="text-2xl font-black text-fg tracking-tight flex items-center gap-2.5">
					Notícias do Luxmc
					<span class="text-xs font-bold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
						Oficial
					</span>
				</h1>
				<p class="text-xs text-fg/50 mt-0.5">Notas de atualização, novos recursos, melhorias do launcher e avisos da comunidade</p>
			</div>
		</div>

		<div class="flex items-center gap-3">
			<button
				type="button"
				onclick={() => openWebsite("https://luxmc-r92.pages.dev")}
				class="flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-fg/10 hover:bg-fg/20 text-fg text-xs font-bold border border-fg/15 transition-all cursor-pointer shadow-md active:scale-[0.98]"
			>
				<Globe class="w-4 h-4 text-emerald-400" />
				<span>Visitar Site Oficial</span>
				<ExternalLink class="w-3 h-3 text-fg/40" />
			</button>
		</div>
	</header>

	<!-- Tabs -->
	<div class="flex items-center gap-2 p-1.5 rounded-2xl bg-bg-elevated border border-fg/10 w-fit">
		<button
			type="button"
			onclick={() => selectedTab = "news"}
			class="px-5 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {selectedTab === 'news' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
		>
			Atualizações do Launcher ({officialNews.length})
		</button>
		<button
			type="button"
			onclick={() => selectedTab = "changelog"}
			class="px-5 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer {selectedTab === 'changelog' ? 'bg-fg/15 text-fg shadow-sm' : 'text-fg/60 hover:text-fg'}"
		>
			Changelog Técnico
		</button>
	</div>

	{#if selectedTab === "news"}
		<!-- Featured Article -->
		{#if officialNews.length > 0}
			{@const featured = officialNews[0]}
			<div class="rounded-3xl bg-bg-elevated border border-blue-500/30 shadow-2xl relative overflow-hidden group flex flex-col lg:flex-row">
				<div class="lg:w-1/2 h-56 lg:h-auto relative overflow-hidden">
					<img 
						src={featured.image} 
						alt={featured.title} 
						class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
					/>
					<div class="absolute inset-0 bg-gradient-to-t lg:bg-gradient-to-r from-transparent via-bg-elevated/60 to-bg-elevated"></div>
				</div>

				<div class="lg:w-1/2 p-6 md:p-8 relative z-10 space-y-4 flex flex-col justify-between">
					<div class="space-y-3">
						<div class="flex items-center gap-2.5 flex-wrap">
							<span class="text-xs font-extrabold px-3 py-1 rounded-full {featured.tagColor}">
								{featured.tag}
							</span>
							<span class="text-xs font-mono font-bold bg-fg/10 px-2.5 py-0.5 rounded-full text-fg/90">
								{featured.version}
							</span>
							<span class="text-xs text-fg/40 flex items-center gap-1">
								<Calendar class="w-3.5 h-3.5" />
								{featured.date}
							</span>
						</div>

						<h2 class="text-xl md:text-2xl font-black text-fg group-hover:text-blue-300 transition-colors">
							{featured.title}
						</h2>

						<p class="text-xs md:text-sm text-fg/70 leading-relaxed">
							{featured.summary}
						</p>

						<div class="pt-2">
							<h3 class="text-xs font-bold uppercase tracking-wider text-blue-400 mb-2.5 flex items-center gap-1.5">
								<Sparkles class="w-3.5 h-3.5" /> Principais Destaques:
							</h3>
							<div class="grid grid-cols-1 gap-2">
								{#each featured.highlights.slice(0, 3) as hl}
									<div class="flex items-start gap-2 text-xs text-fg/80 bg-fg/[0.03] border border-fg/5 p-2.5 rounded-xl">
										<CheckCircle2 class="w-3.5 h-3.5 text-blue-400 shrink-0 mt-0.5" />
										<span>{hl}</span>
									</div>
								{/each}
							</div>
						</div>
					</div>

					<div class="pt-3 flex items-center justify-end">
						<button
							type="button"
							onclick={() => openWebsite(featured.link)}
							class="flex items-center gap-2 px-5 py-2.5 rounded-2xl bg-blue-600 hover:bg-blue-500 text-fg text-xs font-black transition-all shadow-lg shadow-blue-600/20 active:scale-[0.98] cursor-pointer"
						>
							<span>Ver Notas Completas</span>
							<ExternalLink class="w-3.5 h-3.5" />
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Other Updates Grid -->
		<div class="space-y-3 pt-2">
			<h3 class="text-xs font-bold uppercase tracking-wider text-fg/50">Histórico de Atualizações</h3>

			<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
				{#each officialNews.slice(1) as article (article.id)}
					<div class="rounded-3xl bg-bg-elevated border border-fg/10 hover:border-blue-500/30 transition-all flex flex-col justify-between overflow-hidden shadow-lg group">
						<div class="w-full h-36 relative overflow-hidden bg-bg">
							<img 
								src={article.image} 
								alt={article.title} 
								class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
							/>
							<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-transparent to-transparent"></div>
							<div class="absolute top-3 left-3">
								<span class="text-[10px] font-bold px-2.5 py-0.5 rounded-full {article.tagColor}">
									{article.tag}
								</span>
							</div>
						</div>

						<div class="p-5 flex flex-col justify-between flex-1 gap-4">
							<div class="space-y-2">
								<div class="flex items-center justify-between text-[10px] text-fg/40">
									<span class="font-mono font-bold text-fg/50">{article.version}</span>
									<span>{article.date}</span>
								</div>

								<h4 class="text-xs font-bold text-fg group-hover:text-blue-300 transition-colors leading-snug line-clamp-2">
									{article.title}
								</h4>

								<p class="text-[11px] text-fg/50 leading-relaxed line-clamp-3">
									{article.summary}
								</p>
							</div>

							<div class="pt-2 border-t border-fg/5 flex items-center justify-between">
								<span class="text-[10px] text-fg/40">Luxmc Launcher</span>
								<button
									type="button"
									onclick={() => openWebsite(article.link)}
									class="text-xs font-bold text-blue-400 hover:text-blue-300 flex items-center gap-1 cursor-pointer transition-colors"
								>
									<span>Detalhes</span>
									<ExternalLink class="w-3 h-3" />
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>

	{:else}
		<Card>
			{#snippet header()}
				<div class="flex items-center gap-2">
					<Sliders class="h-4 w-4 text-emerald-400" />
					<span class="font-medium text-fg">Changelog Oficial do Projeto</span>
				</div>
			{/snippet}
			<ChangelogPanel />
		</Card>
	{/if}
</div>
