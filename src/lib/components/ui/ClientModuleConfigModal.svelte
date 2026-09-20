<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import {
		X,
		Shield,
		Keyboard,
		Activity,
		Zap,
		Compass,
		FlaskConical,
		RotateCcw,
		Check,
		Sliders,
		Eye,
		AlertTriangle,
		Gauge,
		Layers
	} from "lucide-svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";
	import { playSound } from "$lib/utils/sound";

	let { moduleKey, onClose }: { moduleKey: string; onClose: () => void } = $props();

	function handleClose() {
		onClose();
	}

	function resetCurrentModule() {
		if (moduleKey === "armorHud") {
			clientMods.moduleSettings.armorHud = {
				orientation: "vertical",
				durabilityMode: "percent",
				warningLowDurability: true,
				showMainHand: true,
				showOffHand: true,
				showItemCount: true,
				scale: 1.0,
				position: "bottom-right"
			};
		} else if (moduleKey === "keystrokes") {
			clientMods.moduleSettings.keystrokes = {
				showCps: true,
				showSpace: true,
				showMouse: true,
				colorMode: "emerald",
				opacity: 0.75
			};
		} else if (moduleKey === "cps") {
			clientMods.moduleSettings.cps = {
				showRight: true,
				colorMode: "emerald",
				suffix: "CPS"
			};
		} else if (moduleKey === "toggleSprint") {
			clientMods.moduleSettings.toggleSprint = {
				text: "[Correndo (Ativo)]",
				style: "text",
				color: "#34d399"
			};
		} else if (moduleKey === "directionHud") {
			clientMods.moduleSettings.directionHud = {
				style: "bar",
				showBiome: true
			};
		} else if (moduleKey === "potionEffects") {
			clientMods.moduleSettings.potionEffects = {
				showDuration: true,
				blinkOnExpire: true,
				compactMode: false
			};
		}
		clientMods.save();
		playSound("click");
	}

	let simulatedCpsL = $state(12);
	let simulatedCpsR = $state(16);
	let testKeyW = $state(false);
	let testKeyA = $state(false);
	let testKeyS = $state(false);
	let testKeyD = $state(false);
	let testLmb = $state(false);
	let testRmb = $state(false);
	let testSpace = $state(false);
</script>

<div
	class="fixed inset-0 z-[60] flex items-center justify-center bg-bg-overlay/80 backdrop-blur-md p-4 select-none"
	transition:fade={{ duration: 150 }}
	onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
	onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<div
		class="relative flex flex-col w-full max-w-2xl max-h-[85vh] rounded-3xl bg-bg/98 border border-fg/15 shadow-2xl shadow-black/95 overflow-hidden"
		transition:scale={{ duration: 180, start: 0.95 }}
	>
		<!-- Header -->
		<div class="flex items-center justify-between px-6 py-4 border-b border-fg/10 bg-bg-elevated/70">
			<div class="flex items-center gap-3">
				<div class="flex items-center justify-center w-8 h-8 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
					{#if moduleKey === "armorHud"}
						<Shield class="w-4 h-4" />
					{:else if moduleKey === "keystrokes"}
						<Keyboard class="w-4 h-4" />
					{:else if moduleKey === "cps"}
						<Activity class="w-4 h-4" />
					{:else if moduleKey === "toggleSprint"}
						<Zap class="w-4 h-4" />
					{:else if moduleKey === "directionHud"}
						<Compass class="w-4 h-4" />
					{:else if moduleKey === "potionEffects"}
						<FlaskConical class="w-4 h-4" />
					{:else}
						<Sliders class="w-4 h-4" />
					{/if}
				</div>

				<div>
					<h3 class="text-sm font-bold text-fg">
						{#if moduleKey === "armorHud"}
							Configurações de Armor HUD
						{:else if moduleKey === "keystrokes"}
							Configurações de Keystrokes
						{:else if moduleKey === "cps"}
							Configurações do Contador de CPS
						{:else if moduleKey === "toggleSprint"}
							Configurações de ToggleSprint
						{:else if moduleKey === "directionHud"}
							Configurações de Direction HUD
						{:else if moduleKey === "potionEffects"}
							Configurações de Efeitos de Poção
						{:else}
							Configurar Módulo ({moduleKey})
						{/if}
					</h3>
					<p class="text-[11px] text-fg/50">Personalize o comportamento e a aparência in-game</p>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<button
					onclick={resetCurrentModule}
					class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-xl text-xs text-fg/50 hover:text-fg hover:bg-fg/10 transition-colors"
					title="Restaurar padrão deste módulo"
				>
					<RotateCcw class="w-3.5 h-3.5" />
					<span>Padrão</span>
				</button>

				<button
					onclick={handleClose}
					class="w-8 h-8 rounded-xl flex items-center justify-center text-fg/40 hover:text-fg hover:bg-fg/10 transition-colors"
					aria-label="Fechar"
				>
					<X class="w-4 h-4" />
				</button>
			</div>
		</div>

		<!-- Body -->
		<div class="flex-1 overflow-y-auto custom-scrollbar p-6 space-y-6">
			<!-- ============================================== -->
			<!-- ARMOR HUD CONFIGURATION -->
			<!-- ============================================== -->
			{#if moduleKey === "armorHud"}
				<!-- Visual Preview Card -->
				<div class="p-4 rounded-2xl bg-bg-elevated/40 border border-fg/10 flex flex-col gap-3">
					<div class="flex items-center justify-between text-xs text-fg/60">
						<span class="flex items-center gap-1.5 font-semibold text-emerald-400">
							<Eye class="w-3.5 h-3.5" />
							Pré-visualização do HUD
						</span>
						<span class="text-[10px] uppercase tracking-wider text-fg/40">Minecraft 1.8.9 & Moderno</span>
					</div>

					<!-- HUD Element Rendering Simulator -->
					<div class="flex items-center justify-center p-6 rounded-xl bg-black/60 border border-fg/10 min-h-[140px]">
						<div
							class="flex gap-3 p-2.5 rounded-xl bg-black/40 border border-white/5 shadow-inner {clientMods.moduleSettings.armorHud.orientation === 'horizontal' ? 'flex-row items-center' : 'flex-col items-start'}"
							style="transform: scale({clientMods.moduleSettings.armorHud.scale});"
						>
							<!-- Capacete -->
							<div class="flex items-center gap-2">
								<div class="w-7 h-7 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-blue-300 text-[11px] font-bold shadow-sm">
									🪖
								</div>
								{#if clientMods.moduleSettings.armorHud.durabilityMode === "percent"}
									<span class="text-xs font-mono font-bold text-emerald-400">100%</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "numeric"}
									<span class="text-[11px] font-mono text-fg/80">363/363</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "bar"}
									<div class="w-12 h-1.5 rounded-full bg-fg/20 overflow-hidden">
										<div class="w-full h-full bg-emerald-400"></div>
									</div>
								{/if}
							</div>

							<!-- Peitoral -->
							<div class="flex items-center gap-2">
								<div class="w-7 h-7 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-blue-300 text-[11px] font-bold shadow-sm">
									👕
								</div>
								{#if clientMods.moduleSettings.armorHud.durabilityMode === "percent"}
									<span class="text-xs font-mono font-bold text-emerald-400">78%</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "numeric"}
									<span class="text-[11px] font-mono text-fg/80">412/528</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "bar"}
									<div class="w-12 h-1.5 rounded-full bg-fg/20 overflow-hidden">
										<div class="w-[78%] h-full bg-emerald-400"></div>
									</div>
								{/if}
							</div>

							<!-- Calça -->
							<div class="flex items-center gap-2">
								<div class="w-7 h-7 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-blue-300 text-[11px] font-bold shadow-sm">
									👖
								</div>
								{#if clientMods.moduleSettings.armorHud.durabilityMode === "percent"}
									<span class="text-xs font-mono font-bold text-yellow-400">54%</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "numeric"}
									<span class="text-[11px] font-mono text-yellow-300">267/495</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "bar"}
									<div class="w-12 h-1.5 rounded-full bg-fg/20 overflow-hidden">
										<div class="w-[54%] h-full bg-yellow-400"></div>
									</div>
								{/if}
							</div>

							<!-- Botas (Alerta de durabilidade baixa se ativo) -->
							<div class="flex items-center gap-2">
								<div class="relative w-7 h-7 rounded bg-blue-500/20 border {clientMods.moduleSettings.armorHud.warningLowDurability ? 'border-red-500 text-red-400 animate-pulse' : 'border-blue-400/40 text-blue-300'} flex items-center justify-center text-[11px] font-bold shadow-sm">
									👢
								</div>
								{#if clientMods.moduleSettings.armorHud.durabilityMode === "percent"}
									<span class="text-xs font-mono font-bold {clientMods.moduleSettings.armorHud.warningLowDurability ? 'text-red-400 font-extrabold animate-pulse' : 'text-red-400'}">18%</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "numeric"}
									<span class="text-[11px] font-mono text-red-400 font-bold">77/429</span>
								{:else if clientMods.moduleSettings.armorHud.durabilityMode === "bar"}
									<div class="w-12 h-1.5 rounded-full bg-fg/20 overflow-hidden">
										<div class="w-[18%] h-full bg-red-500"></div>
									</div>
								{/if}
							</div>

							<!-- Mão Principal (Espada) -->
							{#if clientMods.moduleSettings.armorHud.showMainHand}
								<div class="flex items-center gap-2 pt-1 border-t border-white/10">
									<div class="w-7 h-7 rounded bg-amber-500/20 border border-amber-400/40 flex items-center justify-center text-amber-300 text-[11px] font-bold shadow-sm">
										🗡️
									</div>
									{#if clientMods.moduleSettings.armorHud.durabilityMode === "percent"}
										<span class="text-xs font-mono font-bold text-emerald-400">92%</span>
									{:else if clientMods.moduleSettings.armorHud.durabilityMode === "numeric"}
										<span class="text-[11px] font-mono text-fg/80">1436/1561</span>
									{:else if clientMods.moduleSettings.armorHud.durabilityMode === "bar"}
										<div class="w-12 h-1.5 rounded-full bg-fg/20 overflow-hidden">
											<div class="w-[92%] h-full bg-emerald-400"></div>
										</div>
									{/if}
								</div>
							{/if}

							<!-- Mão Secundária (Totem / Escudo) -->
							{#if clientMods.moduleSettings.armorHud.showOffHand}
								<div class="flex items-center gap-2">
									<div class="relative w-7 h-7 rounded bg-purple-500/20 border border-purple-400/40 flex items-center justify-center text-purple-300 text-[11px] font-bold shadow-sm">
										🛡️
										{#if clientMods.moduleSettings.armorHud.showItemCount}
											<span class="absolute -bottom-1 -right-1 px-1 rounded bg-black/90 text-[8px] font-mono text-yellow-300 border border-yellow-500/30">x2</span>
										{/if}
									</div>
									<span class="text-[11px] font-mono text-purple-300">Off-hand</span>
								</div>
							{/if}
						</div>
					</div>
				</div>

				<!-- Options List -->
				<div class="space-y-4">
					<!-- Orientação -->
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Orientação do HUD</div>
							<div class="text-xs text-fg/50">Disposição vertical (coluna) ou horizontal (linha)</div>
						</div>
						<div class="flex items-center gap-1.5 p-1 rounded-xl bg-fg/5 border border-fg/10">
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "orientation", "vertical"); }}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.orientation === 'vertical' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Vertical
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "orientation", "horizontal"); }}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.orientation === 'horizontal' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Horizontal
							</button>
						</div>
					</div>

					<!-- Modo de Durabilidade -->
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Exibição de Durabilidade</div>
							<div class="text-xs text-fg/50">Como a vida restante do equipamento é indicada</div>
						</div>
						<div class="flex items-center gap-1 p-1 rounded-xl bg-fg/5 border border-fg/10">
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "durabilityMode", "percent"); }}
								class="px-2.5 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.durabilityMode === 'percent' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								%
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "durabilityMode", "numeric"); }}
								class="px-2.5 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.durabilityMode === 'numeric' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								123
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "durabilityMode", "bar"); }}
								class="px-2.5 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.durabilityMode === 'bar' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Barra
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "durabilityMode", "none"); }}
								class="px-2.5 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.durabilityMode === 'none' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Ocultar
							</button>
						</div>
					</div>

					<!-- Alerta de Durabilidade Baixa -->
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg flex items-center gap-1.5">
								<span>Alerta de Quebra Crítica</span>
								<span class="px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 text-[10px] font-bold border border-red-500/20">&lt; 20%</span>
							</div>
							<div class="text-xs text-fg/50">Piscar em vermelho quando a armadura estiver prestes a quebrar</div>
						</div>
						<input
							type="checkbox"
							checked={clientMods.moduleSettings.armorHud.warningLowDurability}
							onchange={(e) => { clientMods.updateModuleSetting("armorHud", "warningLowDurability", e.currentTarget.checked); }}
							class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
						/>
					</div>

					<!-- Exibir Mão Principal e Off-hand -->
					<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
						<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
							<div>
								<div class="text-sm font-medium text-fg">Mão Principal</div>
								<div class="text-xs text-fg/50">Arma equipada (Espada/Arco)</div>
							</div>
							<input
								type="checkbox"
								checked={clientMods.moduleSettings.armorHud.showMainHand}
								onchange={(e) => { clientMods.updateModuleSetting("armorHud", "showMainHand", e.currentTarget.checked); }}
								class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
							/>
						</div>

						<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
							<div>
								<div class="text-sm font-medium text-fg">Off-hand</div>
								<div class="text-xs text-fg/50">Escudo ou Totem secundário</div>
							</div>
							<input
								type="checkbox"
								checked={clientMods.moduleSettings.armorHud.showOffHand}
								onchange={(e) => { clientMods.updateModuleSetting("armorHud", "showOffHand", e.currentTarget.checked); }}
								class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
							/>
						</div>
					</div>

					<!-- Escala do HUD -->
					<div class="p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10 space-y-2">
						<div class="flex items-center justify-between text-sm font-medium text-fg">
							<span>Escala do HUD</span>
							<span class="text-xs font-mono text-emerald-400 font-bold">{Math.round(clientMods.moduleSettings.armorHud.scale * 100)}%</span>
						</div>
						<input
							type="range"
							min="0.75"
							max="1.35"
							step="0.05"
							value={clientMods.moduleSettings.armorHud.scale}
							oninput={(e) => { clientMods.updateModuleSetting("armorHud", "scale", parseFloat(e.currentTarget.value)); }}
							class="w-full accent-emerald-400 cursor-pointer"
						/>
					</div>
				</div>

			<!-- ============================================== -->
			<!-- KEYSTROKES CONFIGURATION -->
			<!-- ============================================== -->
			{:else if moduleKey === "keystrokes"}
				<!-- Keystrokes Simulator -->
				<div class="p-4 rounded-2xl bg-bg-elevated/40 border border-fg/10 flex flex-col items-center gap-3">
					<div class="w-full flex items-center justify-between text-xs text-fg/60">
						<span class="flex items-center gap-1.5 font-semibold text-emerald-400">
							<Eye class="w-3.5 h-3.5" />
							Simulador Interativo
						</span>
						<span class="text-[10px] text-fg/40">Clique nas teclas abaixo para testar o feedback</span>
					</div>

					<div class="flex flex-col items-center gap-1.5 p-4 rounded-2xl bg-black/60 border border-fg/10">
						<!-- W -->
						<button
							onmousedown={() => testKeyW = true}
							onmouseup={() => testKeyW = false}
							class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-all {testKeyW ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
						>
							W
						</button>

						<!-- A S D -->
						<div class="flex items-center gap-1.5">
							<button
								onmousedown={() => testKeyA = true}
								onmouseup={() => testKeyA = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-all {testKeyA ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								A
							</button>
							<button
								onmousedown={() => testKeyS = true}
								onmouseup={() => testKeyS = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-all {testKeyS ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								S
							</button>
							<button
								onmousedown={() => testKeyD = true}
								onmouseup={() => testKeyD = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-all {testKeyD ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								D
							</button>
						</div>

						<!-- LMB & RMB -->
						{#if clientMods.moduleSettings.keystrokes.showMouse}
							<div class="flex items-center gap-1.5 w-full">
								<button
									onmousedown={() => testLmb = true}
									onmouseup={() => testLmb = false}
									class="flex-1 h-12 rounded-xl flex flex-col items-center justify-center font-bold font-mono text-xs transition-all {testLmb ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
								>
									<span>LMB</span>
									{#if clientMods.moduleSettings.keystrokes.showCps}
										<span class="text-[9px] opacity-75">{simulatedCpsL} CPS</span>
									{/if}
								</button>
								<button
									onmousedown={() => testRmb = true}
									onmouseup={() => testRmb = false}
									class="flex-1 h-12 rounded-xl flex flex-col items-center justify-center font-bold font-mono text-xs transition-all {testRmb ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
								>
									<span>RMB</span>
									{#if clientMods.moduleSettings.keystrokes.showCps}
										<span class="text-[9px] opacity-75">{simulatedCpsR} CPS</span>
									{/if}
								</button>
							</div>
						{/if}

						<!-- Space -->
						{#if clientMods.moduleSettings.keystrokes.showSpace}
							<button
								type="button"
								aria-label="Espaço"
								onmousedown={() => testSpace = true}
								onmouseup={() => testSpace = false}
								class="w-full h-7 rounded-xl flex items-center justify-center font-bold font-mono text-xs transition-all {testSpace ? 'bg-emerald-400 text-black' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								<span class="w-12 h-1 rounded-full bg-current opacity-70"></span>
							</button>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Exibir CPS nos Botões do Mouse</div>
							<div class="text-xs text-fg/50">Mostra cliques/segundo em tempo real em LMB e RMB</div>
						</div>
						<input
							type="checkbox"
							checked={clientMods.moduleSettings.keystrokes.showCps}
							onchange={(e) => { clientMods.updateModuleSetting("keystrokes", "showCps", e.currentTarget.checked); }}
							class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
						/>
					</div>

					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Exibir Barra de Espaço</div>
							<div class="text-xs text-fg/50">Tecla de pulo na parte inferior</div>
						</div>
						<input
							type="checkbox"
							checked={clientMods.moduleSettings.keystrokes.showSpace}
							onchange={(e) => { clientMods.updateModuleSetting("keystrokes", "showSpace", e.currentTarget.checked); }}
							class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
						/>
					</div>

					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Modo de Cor</div>
							<div class="text-xs text-fg/50">Esquema cromático das teclas pressionadas</div>
						</div>
						<div class="flex items-center gap-1.5">
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "emerald")}
								class="w-6 h-6 rounded-full bg-emerald-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'emerald' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title="Verde Esmeralda"
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "cyan")}
								class="w-6 h-6 rounded-full bg-cyan-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'cyan' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title="Ciano Luxmc"
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "white")}
								class="w-6 h-6 rounded-full bg-white border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'white' ? 'border-emerald-400 scale-110' : 'border-transparent opacity-60'}"
								title="Branco Neve"
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "chroma")}
								class="w-6 h-6 rounded-full bg-gradient-to-tr from-pink-500 via-amber-400 to-cyan-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'chroma' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title="Chroma Arco-íris"
							></button>
						</div>
					</div>
				</div>

			<!-- ============================================== -->
			<!-- CPS COUNTER CONFIGURATION -->
			<!-- ============================================== -->
			{:else if moduleKey === "cps"}
				<div class="p-4 rounded-2xl bg-bg-elevated/40 border border-fg/10 flex flex-col items-center gap-3">
					<div class="flex items-center gap-4 p-4 rounded-xl bg-black/60 border border-fg/10 font-mono font-bold text-lg">
						<span class="text-emerald-400">14 {clientMods.moduleSettings.cps.suffix}</span>
						{#if clientMods.moduleSettings.cps.showRight}
							<span class="text-fg/30">|</span>
							<span class="text-cyan-400">18 RMB {clientMods.moduleSettings.cps.suffix}</span>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Exibir Botão Direito (RMB)</div>
							<div class="text-xs text-fg/50">Mede cliques do botão direito para drag-clicking e blockhit</div>
						</div>
						<input
							type="checkbox"
							checked={clientMods.moduleSettings.cps.showRight}
							onchange={(e) => { clientMods.updateModuleSetting("cps", "showRight", e.currentTarget.checked); }}
							class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
						/>
					</div>

					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Sufixo de Texto</div>
							<div class="text-xs text-fg/50">Ex: "CPS", "Cliques", "c/s"</div>
						</div>
						<input
							type="text"
							value={clientMods.moduleSettings.cps.suffix}
							oninput={(e) => { clientMods.updateModuleSetting("cps", "suffix", e.currentTarget.value); }}
							class="w-24 px-3 py-1.5 rounded-xl bg-fg/5 border border-fg/10 text-sm text-center font-mono text-fg focus:outline-none focus:border-emerald-400"
						/>
					</div>
				</div>

			<!-- ============================================== -->
			<!-- TOGGLE SPRINT CONFIGURATION -->
			<!-- ============================================== -->
			{:else if moduleKey === "toggleSprint"}
				<div class="p-4 rounded-2xl bg-bg-elevated/40 border border-fg/10 flex flex-col items-center gap-3">
					<div class="flex items-center gap-2 p-3 rounded-xl bg-black/60 border border-fg/10 font-mono font-bold text-sm text-emerald-400">
						{#if clientMods.moduleSettings.toggleSprint.style === "icon"}
							<Zap class="w-4 h-4 fill-current" />
							<span>ATIVO</span>
						{:else}
							<span>{clientMods.moduleSettings.toggleSprint.text}</span>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Estilo de Exibição</div>
							<div class="text-xs text-fg/50">Texto tradicional ou ícone minimalista</div>
						</div>
						<div class="flex items-center gap-1.5 p-1 rounded-xl bg-fg/5 border border-fg/10">
							<button
								onclick={() => clientMods.updateModuleSetting("toggleSprint", "style", "text")}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.toggleSprint.style === 'text' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Texto
							</button>
							<button
								onclick={() => clientMods.updateModuleSetting("toggleSprint", "style", "icon")}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.toggleSprint.style === 'icon' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								Ícone
							</button>
						</div>
					</div>

					{#if clientMods.moduleSettings.toggleSprint.style === "text"}
						<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
							<div>
								<div class="text-sm font-medium text-fg">Texto Customizado</div>
								<div class="text-xs text-fg/50">Mensagem exibida quando correndo</div>
							</div>
							<input
								type="text"
								value={clientMods.moduleSettings.toggleSprint.text}
								oninput={(e) => { clientMods.updateModuleSetting("toggleSprint", "text", e.currentTarget.value); }}
								class="w-48 px-3 py-1.5 rounded-xl bg-fg/5 border border-fg/10 text-sm font-mono text-fg focus:outline-none focus:border-emerald-400"
							/>
						</div>
					{/if}
				</div>

			<!-- ============================================== -->
			<!-- DIRECTION HUD CONFIGURATION -->
			<!-- ============================================== -->
			{:else if moduleKey === "directionHud"}
				<div class="p-4 rounded-2xl bg-bg-elevated/40 border border-fg/10 flex flex-col items-center gap-3">
					<div class="flex items-center justify-center p-3 rounded-xl bg-black/60 border border-fg/10 font-mono text-xs w-full">
						<span class="text-fg/40">W · NW · </span>
						<span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/40">[ N ]</span>
						<span class="text-fg/40"> · NE · E</span>
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">Exibir Bioma Atual</div>
							<div class="text-xs text-fg/50">Mostra o nome do bioma abaixo da bússola</div>
						</div>
						<input
							type="checkbox"
							checked={clientMods.moduleSettings.directionHud.showBiome}
							onchange={(e) => { clientMods.updateModuleSetting("directionHud", "showBiome", e.currentTarget.checked); }}
							class="w-5 h-5 rounded-md accent-emerald-400 cursor-pointer"
						/>
					</div>
				</div>

			<!-- ============================================== -->
			<!-- OTHER MODULES (GENERIC TOGGLE) -->
			<!-- ============================================== -->
			{:else}
				<div class="p-6 rounded-2xl bg-bg-elevated/40 border border-fg/10 text-center space-y-2">
					<div class="text-sm font-medium text-fg">Módulo Ativo e Otimizado</div>
					<p class="text-xs text-fg/50">Este módulo utiliza presets de alta performance com zero impacto no FPS.</p>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-between px-6 py-4 border-t border-fg/10 bg-bg">
			<span class="text-xs text-fg/40">Salvo automaticamente</span>
			<button
				onclick={handleClose}
				class="px-5 py-2 rounded-2xl bg-emerald-500 text-black font-semibold text-xs hover:bg-emerald-400 active:scale-95 transition-all shadow-lg shadow-emerald-500/20"
			>
				Concluído
			</button>
		</div>
	</div>
</div>
