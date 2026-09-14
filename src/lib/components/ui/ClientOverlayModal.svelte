<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import {
		Eye,
		Sparkles,
		Bomb,
		Keyboard,
		Crosshair,
		Gauge,
		Footprints,
		ShieldCheck,
		Sun,
		RotateCcw,
		X,
		Sliders,
		Check,
		Zap
	} from "lucide-svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let activeTab = $state<"pvp" | "hud" | "visual" | "anticrash">("pvp");

	function handleClose() {
		clientMods.close();
	}

	function handleReset() {
		clientMods.reset();
		toast("Configurações do Cliente restauradas para o padrão.", "info");
	}

	function toggle(key: keyof typeof clientMods.config) {
		const curr = clientMods.config[key];
		if (typeof curr === "boolean") {
			clientMods.update(key, !curr as any);
			playSound("click");
		}
	}
</script>

{#if clientMods.isMenuOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md p-4 select-none"
		transition:fade={{ duration: 180 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div
			class="relative flex flex-col w-full max-w-4xl h-[650px] max-h-[90vh] rounded-3xl bg-[#121316] border border-white/10 shadow-2xl shadow-black/80 overflow-hidden"
			transition:scale={{ duration: 200, start: 0.95 }}
		>
						<div class="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-[#16171b]">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-gradient-to-br from-amber-500/20 to-amber-700/10 border border-amber-500/30 flex items-center justify-center text-amber-400">
						<Sliders class="w-5 h-5" />
					</div>
					<div>
						<div class="flex items-center gap-2">
							<h2 class="text-base font-bold text-white tracking-wide">Luxmc Client Suite</h2>
							<span class="px-2 py-0.5 text-[10px] font-bold rounded-md bg-amber-500/20 text-amber-300 border border-amber-500/30">PvP & Utility</span>
						</div>
						<p class="text-xs text-white/50">Atalho rápido de acesso in-game e launcher: <kbd class="px-1.5 py-0.5 rounded bg-white/10 text-white font-mono text-[10px]">RShift</kbd> ou <kbd class="px-1.5 py-0.5 rounded bg-white/10 text-white font-mono text-[10px]">Insert</kbd></p>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<button
						onclick={handleReset}
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-xs text-white/60 hover:text-white transition-colors border border-white/5"
						title="Restaurar padrões"
					>
						<RotateCcw class="w-3.5 h-3.5" />
						<span>Padrão</span>
					</button>

					<button
						onclick={handleClose}
						class="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-white/60 hover:text-white transition-colors"
						aria-label="Fechar"
					>
						<X class="w-5 h-5" />
					</button>
				</div>
			</div>

						<div class="flex items-center gap-2 px-6 py-2.5 bg-[#0f1013] border-b border-white/5 overflow-x-auto custom-scrollbar">
				<button
					onclick={() => activeTab = "pvp"}
					class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-all {activeTab === 'pvp' ? 'bg-amber-500 text-black shadow-lg shadow-amber-500/20' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				>
					<Bomb class="w-4 h-4" />
					<span>Combate & PvP</span>
				</button>

				<button
					onclick={() => activeTab = "hud"}
					class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-all {activeTab === 'hud' ? 'bg-amber-500 text-black shadow-lg shadow-amber-500/20' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				>
					<Keyboard class="w-4 h-4" />
					<span>Keystrokes & HUD</span>
				</button>

				<button
					onclick={() => activeTab = "visual"}
					class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-all {activeTab === 'visual' ? 'bg-amber-500 text-black shadow-lg shadow-amber-500/20' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				>
					<Eye class="w-4 h-4" />
					<span>Perspective & Visual</span>
				</button>

				<button
					onclick={() => activeTab = "anticrash"}
					class="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold transition-all {activeTab === 'anticrash' ? 'bg-amber-500 text-black shadow-lg shadow-amber-500/20' : 'text-white/60 hover:text-white hover:bg-white/5'}"
				>
					<ShieldCheck class="w-4 h-4" />
					<span>Anti-Crash & Performance</span>
				</button>
			</div>

						<div class="flex-1 overflow-y-auto p-6 space-y-4 custom-scrollbar">
				{#if activeTab === "pvp"}
										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-red-500/10 border border-red-500/20 flex items-center justify-center text-red-400">
								<Bomb class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">TNT Timer</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-red-500/20 text-red-400">PvP Essential</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Exibe uma contagem regressiva em segundos/ticks em cima de blocos de TNT acesos antes de explodirem.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("tntTimer")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.tntTimer ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar TNT Timer"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.tntTimer ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
								<Eye class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Perspective Mod (360° Freelook)</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-sky-500/20 text-sky-400">F5 Dinâmico</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Permite girar a câmera em 360 graus livremente ao redor do seu personagem sem alterar a direção para onde você anda.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("perspectiveMod")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.perspectiveMod ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Perspective Mod"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.perspectiveMod ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
								<Footprints class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Toggle Sprint (Auto-Corrida)</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-400">Competitivo</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Mantém o sprint ativado automaticamente ao pressionar W, sem precisar dar toque duplo ou cansar o dedo.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("toggleSprint")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.toggleSprint ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Toggle Sprint"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.toggleSprint ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400">
								<Crosshair class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Mira Customizada (Crosshair)</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Substitui a cruz tradicional por uma mira customizada (ponto central, círculo, chevron ou cruz com cor customizável).</p>
							</div>
						</div>
						<button
							onclick={() => toggle("customCrosshair")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.customCrosshair ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Mira Customizada"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.customCrosshair ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

				{:else if activeTab === "hud"}
										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400">
								<Keyboard class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Keystrokes HUD</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-500/20 text-amber-400">WASD & Clicks</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Exibe as teclas WASD, barra de espaço e botões do mouse com efeito de clique animado na tela.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("keystrokes")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.keystrokes ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Keystrokes"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.keystrokes ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
								<Gauge class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Contador de FPS & Ping</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Mostra a taxa de quadros e latência da conexão com o servidor no canto da tela de forma discreta.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("fpsHud")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.fpsHud ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar FPS & Ping"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.fpsHud ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
								<Zap class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Contador de CPS (Clicks Per Second)</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Registra e exibe os cliques por segundo tanto do botão esquerdo (ataque) quanto direito (blockhit/ponte).</p>
							</div>
						</div>
						<button
							onclick={() => toggle("cpsHud")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.cpsHud ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar CPS Counter"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.cpsHud ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-orange-500/10 border border-orange-500/20 flex items-center justify-center text-orange-400">
								<ShieldCheck class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Status de Armadura & Poções Ativas</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Exibe durabilidade das 4 peças da armadura e tempo restante de Speed, Força, Resistência e outras poções.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("armorStatusHud")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.armorStatusHud ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Status de Armadura"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.armorStatusHud ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

				{:else if activeTab === "visual"}
										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400">
								<Sparkles class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Camadas 3D da Skin & Cosméticos</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Renderiza a jaqueta, chapéu, calças e mangas com volume tridimensional real de 1 pixel em vez de textura plana.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("threeDSkinLayers")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.threeDSkinLayers ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Camadas 3D da Skin"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.threeDSkinLayers ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400">
								<Sparkles class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Física Suave de Capas</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Suaviza as curvas de movimentação da capa com física inercial realista durante corridas e saltos.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("animatedCapes")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.animatedCapes ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Capas Animadas"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.animatedCapes ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400">
								<Sun class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Fullbright (Gama 1000%)</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-500/20 text-amber-400">Visão Noturna</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Ilumina todas as cavernas e noites como se estivesse de dia, sem precisar posicionar tochas.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("fullbright")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.fullbright ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Fullbright"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.fullbright ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

				{:else if activeTab === "anticrash"}
										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
								<ShieldCheck class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Guarda Anti-Crash Ativo</h3>
									<span class="px-2 py-0.5 rounded text-[10px] font-bold bg-emerald-500/20 text-emerald-400">Proteção Ativa</span>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Detecta e previne erros comuns de inicialização (OpenGL incompatível, drivers desatualizados, GLFW Wayland e flags JVM).</p>
							</div>
						</div>
						<button
							onclick={() => toggle("antiCrashGuard")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.antiCrashGuard ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Anti-Crash Guard"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.antiCrashGuard ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
								<Gauge class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Auto-Limpeza de RAM em Segundo Plano</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Reduz o consumo de RAM do launcher e do sistema enquanto o Minecraft estiver aberto, devolvendo memória ao SO.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("autoTrimMemory")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.autoTrimMemory ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Auto Trim Memory"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.autoTrimMemory ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>

										<div class="p-5 rounded-2xl bg-[#16171b] border border-white/5 flex items-center justify-between">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center text-purple-400">
								<Zap class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h3 class="text-sm font-bold text-white">Auto-Verificação de Dependências Faltantes</h3>
								</div>
								<p class="text-xs text-white/50 mt-0.5">Verifica automaticamente se faltam bibliotecas obrigatórias (como Curios, GeckoLib, Balm) antes de dar play para evitar crashes.</p>
							</div>
						</div>
						<button
							onclick={() => toggle("preflightModCheck")}
							class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {clientMods.config.preflightModCheck ? 'bg-amber-500' : 'bg-white/10'}"
							aria-label="Alternar Verificação de Dependências"
						>
							<span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {clientMods.config.preflightModCheck ? 'translate-x-6' : 'translate-x-1'}"></span>
						</button>
					</div>
				{/if}
			</div>

						<div class="flex items-center justify-between px-6 py-3.5 bg-[#16171b] border-t border-white/5 text-xs text-white/60">
				<div class="flex items-center gap-2">
					<span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
					<span>Suporte universal: Vanilla, 1.8.9 PvP, Fabric, NeoForge, Forge e Quilt</span>
				</div>
				<button
					onclick={handleClose}
					class="flex items-center gap-2 px-5 py-2 rounded-xl bg-amber-500 hover:bg-amber-400 text-black font-bold transition-all shadow-md hover:scale-105"
				>
					<Check class="w-4 h-4" />
					<span>Salvar & Aplicar</span>
				</button>
			</div>
		</div>
	</div>
{/if}
