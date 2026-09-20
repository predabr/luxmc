<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import {
		X,
		Home,
		PenTool,
		Settings,
		Info,
		Search,
		Shield,
		Activity,
		Crosshair,
		Compass,
		Zap,
		Keyboard,
		Cpu,
		Image,
		Wifi,
		Clock,
		FlaskConical,
		Gauge,
		ListOrdered,
		Globe,
		Sun,
		Bomb,
		Waves,
		MapPin,
		CloudRain,
		ZoomIn,
		Layers,
		Eye,
		Users,
		MessageSquare,
		Sparkles,
		Heart,
		Terminal,
		Smile,
		Wand2,
		Palette,
		Sliders,
		Check,
		RotateCcw
	} from "lucide-svelte";
	import { clientMods, type ClientModsConfig } from "$lib/stores/clientMods.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let searchQuery = $state("");
	let activeSection = $state<"all" | "hud" | "visual" | "combat" | "utility">("all");

	type ModuleDef = {
		key: keyof ClientModsConfig;
		name: string;
		icon: typeof import("lucide-svelte").Circle;
		category: "hud" | "visual" | "combat" | "utility";
		desc?: string;
	};

	const ALL_MODULES: ModuleDef[] = [
		// Screenshots 1-5 exact modules
		{ key: "armorHud", name: "Armor HUD", icon: Shield, category: "hud", desc: "Status de armadura e durabilidade em tempo real" },
		{ key: "bossbar", name: "Bossbar", icon: Activity, category: "hud", desc: "Barra de chefe customizável" },
		{ key: "comboDisplay", name: "Combo Display", icon: Zap, category: "combat", desc: "Contador de golpes combinados consecutivos" },
		{ key: "coordinates", name: "Coordinates", icon: Compass, category: "hud", desc: "Exibição limpa de X, Y, Z e bioma" },
		{ key: "cps", name: "CPS", icon: Activity, category: "hud", desc: "Cliques por segundo no botão esquerdo e direito" },
		{ key: "directionHud", name: "Direction Hud", icon: Compass, category: "hud", desc: "Bússola visual de orientação (N, S, E, W)" },
		{ key: "fps", name: "FPS", icon: Gauge, category: "hud", desc: "Taxa de quadros por segundo em tempo real" },
		{ key: "keystrokes", name: "Keystrokes", icon: Keyboard, category: "hud", desc: "Teclas W, A, S, D, LMB e RMB na tela" },
		{ key: "memory", name: "Memory", icon: Cpu, category: "hud", desc: "Uso de memória RAM consumida pelo jogo" },
		{ key: "packOverlay", name: "Pack Overlay", icon: Image, category: "visual", desc: "Exibição do ícone da textura ativa" },
		{ key: "pingDisplay", name: "Ping Display", icon: Wifi, category: "hud", desc: "Latência em milissegundos para o servidor" },
		{ key: "playTime", name: "Play Time", icon: Clock, category: "hud", desc: "Tempo decorrido na sessão atual" },
		{ key: "potionCounter", name: "Potion Counter", icon: FlaskConical, category: "combat", desc: "Contador de poções no inventário" },
		{ key: "potionEffects", name: "Potion Effects", icon: FlaskConical, category: "hud", desc: "Efeitos ativos com contagem regressiva" },
		{ key: "reachDisplay", name: "Reach Display", icon: Activity, category: "combat", desc: "Distância do alcance do último golpe" },
		{ key: "scoreboard", name: "Scoreboard", icon: ListOrdered, category: "hud", desc: "Placar do servidor com suporte a cores e fontes" },
		{ key: "serverAddress", name: "Server Address", icon: Globe, category: "hud", desc: "Endereço do servidor em que está jogando" },
		{ key: "speedometer", name: "Speedometer", icon: Gauge, category: "hud", desc: "Velocidade de movimento do jogador em blocos/s" },
		{ key: "timeDisplay", name: "Time Display", icon: Clock, category: "hud", desc: "Relógio do mundo real e hora in-game" },
		{ key: "toggleSprint", name: "Toggle Sprint", icon: Zap, category: "combat", desc: "Correr automaticamente sem segurar Ctrl" },

		{ key: "twoDItems", name: "2D Items", icon: Image, category: "visual", desc: "Itens dropados renderizados no estilo clássico 2D" },
		{ key: "threeDSkinLayers", name: "3D Skin Layers", icon: Layers, category: "visual", desc: "Camada externa da skin em relevo 3D" },
		{ key: "animations", name: "Animations", icon: Sparkles, category: "visual", desc: "Animações fluidas de espada e transição" },
		{ key: "autoFriend", name: "Auto Friend", icon: Users, category: "utility", desc: "Aceita pedidos de amizade de aliados" },
		{ key: "autoGG", name: "AutoGG", icon: MessageSquare, category: "utility", desc: "Envia 'gg' automaticamente ao fim da partida" },
		{ key: "autoText", name: "AutoText", icon: MessageSquare, category: "utility", desc: "Macros rápidas de chat para comandos" },
		{ key: "blockOverlay", name: "Block Overlay", icon: Image, category: "visual", desc: "Destaque colorido no bloco mirado" },
		{ key: "chat", name: "Chat", icon: MessageSquare, category: "hud", desc: "Histórico de chat customizável e sem delay" },
		{ key: "cosmetics", name: "Cosmetics", icon: Wand2, category: "visual", desc: "Asas, bandanas e cosméticos visuais 3D" },
		{ key: "crosshair", name: "Crosshair", icon: Crosshair, category: "visual", desc: "Mira personalizada (círculo, cruz ou ponto)" },
		{ key: "damageTint", name: "Damage Tint", icon: Heart, category: "combat", desc: "Coloração avermelhada ao receber dano" },
		{ key: "debugScreen", name: "Debug Screen", icon: Terminal, category: "hud", desc: "Tela F3 minimalista sem poluição visual" },
		{ key: "discordRP", name: "Discord RP", icon: MessageSquare, category: "utility", desc: "Exibe status do jogo e servidor no Discord" },
		{ key: "emotes", name: "Emotes", icon: Smile, category: "visual", desc: "Dança e gestos expressivos in-game" },
		{ key: "fullbright", name: "Full Bright", icon: Sun, category: "visual", desc: "Brilho máximo em cavernas e ambientes escuros" },
		{ key: "glintColorizer", name: "Glint Colorizer", icon: Palette, category: "visual", desc: "Personalização da cor de encantamento" },
		{ key: "hitbox", name: "Hitbox", icon: Shield, category: "combat", desc: "Caixa de colisão dos alvos e entidades" },
		{ key: "hitColor", name: "Hit Color", icon: Palette, category: "combat", desc: "Cor de piscar ao acertar o oponente" },
		{ key: "inputFix", name: "InputFix", icon: Keyboard, category: "utility", desc: "Correção de caracteres especiais e acentos" },
		{ key: "itemPhysics", name: "Item Physics", icon: Sparkles, category: "visual", desc: "Física realista para itens caídos no chão" },
		{ key: "motionBlur", name: "Motion Blur", icon: Eye, category: "visual", desc: "Efeito de desfoque cinematográfico de movimento" },
		{ key: "nametags", name: "Nametags", icon: Users, category: "visual", desc: "Nomes de jogadores legíveis e escalonados" },
		{ key: "oldAnimations", name: "Old Animations", icon: Sparkles, category: "visual", desc: "Animações clássicas 1.7 de segurar espada (Blockhit)" },
		{ key: "particles", name: "Particles", icon: Sparkles, category: "visual", desc: "Multiplicador de partículas de acerto crítico" },
		{ key: "perspective", name: "Perspective", icon: Eye, category: "visual", desc: "Visão livre 360° em terceira pessoa sem virar o boneco" },
		{ key: "shinyPots", name: "Shiny Pots", icon: FlaskConical, category: "combat", desc: "Frascos de poção com brilho e textura destacada" },
		{ key: "skins", name: "Skins", icon: Layers, category: "visual", desc: "Renderização de skins em alta resolução" },
		{ key: "tab", name: "Tab", icon: ListOrdered, category: "hud", desc: "Lista de jogadores Tab estilizada com ping" },
		{ key: "timeChanger", name: "Time Changer", icon: Clock, category: "visual", desc: "Mude a hora do dia localmente (dia/noite)" },
		{ key: "tntTimer", name: "TNT Timer", icon: Bomb, category: "combat", desc: "Contagem regressiva de explosão da TNT" },
		{ key: "waveyCapes", name: "Wavey Capes", icon: Waves, category: "visual", desc: "Física ondulada para capas do jogador" },
		{ key: "waypoints", name: "Waypoints", icon: MapPin, category: "utility", desc: "Marcadores de coordenadas e pontos de interesse" },
		{ key: "weatherChanger", name: "Weather Changer", icon: CloudRain, category: "visual", desc: "Controle visual de chuva e neve" },
		{ key: "zoom", name: "Zoom", icon: ZoomIn, category: "visual", desc: "Zoom suave estilo OptiFine" }
	];

	const filteredModules = $derived.by(() => {
		let list = ALL_MODULES;
		if (activeSection !== "all") {
			list = list.filter((m) => m.category === activeSection);
		}
		if (!searchQuery.trim()) return list;
		const q = searchQuery.toLowerCase().trim();
		return list.filter((m) => m.name.toLowerCase().includes(q) || (m.desc && m.desc.toLowerCase().includes(q)));
	});

	function handleClose() {
		clientMods.close();
	}

	function handleToggle(key: keyof ClientModsConfig) {
		clientMods.toggle(key);
		playSound("click");
	}

	function handleReset() {
		clientMods.reset();
		toast("Módulos restaurados para o padrão.", "info");
	}
</script>

{#if clientMods.isMenuOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
		transition:fade={{ duration: 150 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div
			class="relative flex w-full max-w-5xl h-[680px] max-h-[92vh] rounded-3xl bg-bg/95 border border-fg/10 shadow-2xl shadow-black/90 overflow-hidden"
			transition:scale={{ duration: 180, start: 0.96 }}
		>
			<div class="flex flex-col items-center justify-between w-16 py-6 border-r border-fg/5 bg-bg">
				<div class="flex flex-col items-center gap-6">
					<button
						onclick={handleClose}
						class="w-10 h-10 rounded-2xl flex items-center justify-center text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
						aria-label="Fechar"
						title="Fechar"
					>
						<X class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "all"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'all' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title="Todos os Módulos"
					>
						<Home class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "hud"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'hud' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title="Módulos de HUD"
					>
						<PenTool class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "visual"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'visual' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title="Visual & Animações"
					>
						<Eye class="w-5 h-5" />
					</button>

					<button
						onclick={() => { activeSection = "combat"; }}
						class="w-10 h-10 rounded-2xl flex items-center justify-center transition-colors {activeSection === 'combat' ? 'text-emerald-400 bg-emerald-500/10 border border-emerald-500/30' : 'text-fg/50 hover:text-fg hover:bg-fg/10'}"
						title="Combate & PvP"
					>
						<Shield class="w-5 h-5" />
					</button>
				</div>

				<div class="flex flex-col items-center gap-4">
					<button
						onclick={handleReset}
						class="w-9 h-9 rounded-xl flex items-center justify-center text-fg/40 hover:text-fg hover:bg-fg/10 transition-colors"
						title="Restaurar padrões"
					>
						<RotateCcw class="w-4 h-4" />
					</button>

					<div
						class="w-9 h-9 rounded-xl flex items-center justify-center text-fg/30"
						title="No Minecraft, este menu abre com Shift Direito"
					>
						<Info class="w-4 h-4" />
					</div>
				</div>
			</div>

			<div class="flex flex-1 flex-col overflow-hidden">
				<div class="flex items-center justify-between px-8 py-4 border-b border-fg/5 bg-bg-elevated">
					<div class="flex items-center gap-3">
						<span class="text-xs font-bold uppercase tracking-widest text-emerald-400">Luxmc Client</span>
						<span class="text-fg/20">/</span>
						<span class="text-xs text-fg/60">
							{filteredModules.length} {filteredModules.length === 1 ? 'módulo' : 'módulos'}
						</span>
					</div>

					<div class="relative w-72">
						<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/40" />
						<input
							type="text"
							bind:value={searchQuery}
							placeholder="Search..."
							class="w-full pl-10 pr-4 py-2 rounded-2xl bg-fg/5 border border-fg/10 text-sm text-fg placeholder-fg/40 focus:outline-none focus:border-emerald-400/50 transition-colors"
						/>
						{#if searchQuery}
							<button
								onclick={() => searchQuery = ""}
								class="absolute right-3 top-1/2 -translate-y-1/2 text-fg/40 hover:text-fg"
							>
								<X class="w-3.5 h-3.5" />
							</button>
						{/if}
					</div>
				</div>

				<div class="flex-1 overflow-y-auto custom-scrollbar p-8">
					<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3.5">
						{#each filteredModules as mod (mod.key)}
							{@const isActive = !!clientMods.config[mod.key]}
							<div
								class="group relative flex items-center justify-between px-4 py-3 rounded-2xl border transition-all duration-200 cursor-pointer select-none {isActive ? 'border-fg/70 bg-fg/[0.07] shadow-lg shadow-black/40' : 'border-fg/10 bg-bg-elevated/60 hover:border-fg/25 hover:bg-fg/[0.03]'}"
								onclick={() => handleToggle(mod.key)}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") handleToggle(mod.key); }}
							>
								<!-- Left: Icon + Separator + Label -->
								<div class="flex items-center gap-3 min-w-0">
									<div class="flex items-center justify-center w-6 h-6 shrink-0 {isActive ? 'text-fg' : 'text-fg/50 group-hover:text-fg/80'}">
										<mod.icon class="w-5 h-5" strokeWidth={1.8} />
									</div>

									<div class="h-4 w-px bg-fg/15 shrink-0"></div>

									<span class="text-sm font-medium truncate {isActive ? 'text-fg font-semibold' : 'text-fg/70 group-hover:text-fg'}">
										{mod.name}
									</span>
								</div>

								<!-- Right: Settings Cog -->
								<div class="flex items-center gap-1.5 shrink-0 pl-2">
									<div class="w-6 h-6 rounded-lg flex items-center justify-center text-fg/30 group-hover:text-fg/70 hover:!text-emerald-400 transition-colors">
										<Settings class="w-3.5 h-3.5" />
									</div>
								</div>
							</div>
						{/each}
					</div>

					{#if filteredModules.length === 0}
						<div class="flex flex-col items-center justify-center h-64 text-center">
							<Search class="w-10 h-10 text-fg/20 mb-3" />
							<p class="text-fg/60 font-medium">Nenhum módulo encontrado</p>
							<p class="text-xs text-fg/40 mt-1">Tente pesquisar por outro termo como "FPS", "Armor" ou "Zoom"</p>
						</div>
					{/if}
				</div>

				<!-- Bottom Status Bar -->
				<div class="flex items-center justify-between px-8 py-3.5 border-t border-fg/5 bg-bg text-xs text-fg/40">
					<div class="flex items-center gap-2">
						<span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
						<span>No Minecraft, este menu abre in-game pressionando <kbd class="px-1.5 py-0.5 rounded bg-fg/10 text-fg font-mono text-[10px]">Shift Direito</kbd></span>
					</div>

					<div class="flex items-center gap-4">
						<span>PvP & FPS Suite v1.7.6</span>
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}
