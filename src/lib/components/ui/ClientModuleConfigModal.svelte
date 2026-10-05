<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
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
	transition:fade={{ easing: quintOut, duration: 220 }}
	onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
	onkeydown={(e) => { if (e.key === "Escape") handleClose(); }}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<div
		class="relative flex flex-col w-full max-w-2xl max-h-[85vh] rounded-3xl bg-bg/98 border border-fg/15 shadow-2xl shadow-black/95 overflow-hidden"
		transition:scale={{ easing: backOut, duration: 240, start: 0.95 }}
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
							{uiText("ui.837c573831c9e244")}
						{:else if moduleKey === "keystrokes"}
							{uiText("ui.bcd217f17dfe36ba")}
						{:else if moduleKey === "cps"}
							{uiText("ui.ad3b27e86d77f534")}
						{:else if moduleKey === "toggleSprint"}
							{uiText("ui.feb67671ed628f02")}
						{:else if moduleKey === "directionHud"}
							{uiText("ui.827406144dc938b1")}
						{:else if moduleKey === "potionEffects"}
							{uiText("ui.c8c5e130fbf2ccfe")}
						{:else}
							{uiText("ui.df0765b76b9d3f01")}{moduleKey})
						{/if}
					</h3>
					<p class="text-[11px] text-fg/50">{uiText("ui.e3e5c61ab1665c39")}</p>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<button
					onclick={resetCurrentModule}
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
					title={uiText("ui.6a79dae77021221c")}
				>
					<RotateCcw class="w-3.5 h-3.5" />
					<span>{uiText("settings.resolutionDefault")}</span>
				</button>

				<button
					onclick={handleClose}
					class={launcherButton({ variant: "secondary", size: "icon", class: "flex items-center justify-center" })}
					aria-label={uiText("statusBanner.dismiss")}
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
							{uiText("ui.07a69721a705912e")}
						</span>
						<span class="text-[10px] uppercase tracking-wider text-fg/40">{uiText("ui.b4dcf2a137e6e3f2")}</span>
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
											<span class="absolute -bottom-1 -right-1 px-1 rounded bg-black/90 text-[8px] font-mono text-yellow-300 border border-yellow-500/30">{uiText("ui.844ecc08164e2eab")}</span>
										{/if}
									</div>
									<span class="text-[11px] font-mono text-purple-300">{uiText("ui.6dcb0b43ee5c55dd")}</span>
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
							<div class="text-sm font-medium text-fg">{uiText("ui.17f96b8a1bafbead")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.cef2420dfd241190")}</div>
						</div>
						<div class="flex items-center gap-1.5 p-1 rounded-xl bg-fg/5 border border-fg/10">
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "orientation", "vertical"); }}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.orientation === 'vertical' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								{uiText("ui.727cd3a64d792ad4")}
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "orientation", "horizontal"); }}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.orientation === 'horizontal' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								{uiText("ui.0abba441f16eff90")}
							</button>
						</div>
					</div>

					<!-- Modo de Durabilidade -->
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">{uiText("ui.4c563c308b017b25")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.aa7655926b9105b2")}</div>
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
								{uiText("ui.797ff8f0ba41da55")}
							</button>
							<button
								onclick={() => { clientMods.updateModuleSetting("armorHud", "durabilityMode", "none"); }}
								class="px-2.5 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.armorHud.durabilityMode === 'none' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								{uiText("ui.dd680772877e1917")}
							</button>
						</div>
					</div>

					<!-- Alerta de Durabilidade Baixa -->
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg flex items-center gap-1.5">
								<span>{uiText("ui.b2e60441d437ba99")}</span>
								<span class="px-1.5 py-0.5 rounded bg-red-500/10 text-red-400 text-[10px] font-bold border border-red-500/20">&lt; 20%</span>
							</div>
							<div class="text-xs text-fg/50">{uiText("ui.671a4d0be0716ddd")}</div>
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
								<div class="text-sm font-medium text-fg">{uiText("ui.b4400a61ec69dbec")}</div>
								<div class="text-xs text-fg/50">{uiText("ui.b672b21a7ce07fff")}</div>
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
								<div class="text-sm font-medium text-fg">{uiText("ui.6dcb0b43ee5c55dd")}</div>
								<div class="text-xs text-fg/50">{uiText("ui.596b4cb5305766ea")}</div>
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
							<span>{uiText("ui.c53ed157cef28922")}</span>
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
							{uiText("ui.15b062385fce6395")}
						</span>
						<span class="text-[10px] text-fg/40">{uiText("ui.b8cb66e7441da09a")}</span>
					</div>

					<div class="flex flex-col items-center gap-1.5 p-4 rounded-2xl bg-black/60 border border-fg/10">
						<!-- W -->
						<button
							onmousedown={() => testKeyW = true}
							onmouseup={() => testKeyW = false}
							class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-[color,background-color,border-color,box-shadow,transform,opacity] {testKeyW ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
						>
							W
						</button>

						<!-- A S D -->
						<div class="flex items-center gap-1.5">
							<button
								onmousedown={() => testKeyA = true}
								onmouseup={() => testKeyA = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-[color,background-color,border-color,box-shadow,transform,opacity] {testKeyA ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								A
							</button>
							<button
								onmousedown={() => testKeyS = true}
								onmouseup={() => testKeyS = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-[color,background-color,border-color,box-shadow,transform,opacity] {testKeyS ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								S
							</button>
							<button
								onmousedown={() => testKeyD = true}
								onmouseup={() => testKeyD = false}
								class="w-12 h-12 rounded-xl flex items-center justify-center font-bold font-mono text-sm transition-[color,background-color,border-color,box-shadow,transform,opacity] {testKeyD ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
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
									class="flex-1 h-12 rounded-xl flex flex-col items-center justify-center font-bold font-mono text-xs transition-[color,background-color,border-color,box-shadow,transform,opacity] {testLmb ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
								>
									<span>{uiText("ui.a0b0cf417d0b4c7d")}</span>
									{#if clientMods.moduleSettings.keystrokes.showCps}
										<span class="text-[9px] opacity-75">{simulatedCpsL} {uiText("ui.f0c5fdda4d5fe35b")}</span>
									{/if}
								</button>
								<button
									onmousedown={() => testRmb = true}
									onmouseup={() => testRmb = false}
									class="flex-1 h-12 rounded-xl flex flex-col items-center justify-center font-bold font-mono text-xs transition-[color,background-color,border-color,box-shadow,transform,opacity] {testRmb ? 'bg-emerald-400 text-black shadow-lg shadow-emerald-500/40' : 'bg-white/10 text-white hover:bg-white/15'}"
								>
									<span>{uiText("ui.02cb6fa562753c7e")}</span>
									{#if clientMods.moduleSettings.keystrokes.showCps}
										<span class="text-[9px] opacity-75">{simulatedCpsR} {uiText("ui.f0c5fdda4d5fe35b")}</span>
									{/if}
								</button>
							</div>
						{/if}

						<!-- Space -->
						{#if clientMods.moduleSettings.keystrokes.showSpace}
							<button
								type="button"
								aria-label={uiText("ui.f53807aa6cf5d027")}
								onmousedown={() => testSpace = true}
								onmouseup={() => testSpace = false}
								class="w-full h-7 rounded-xl flex items-center justify-center font-bold font-mono text-xs transition-[color,background-color,border-color,box-shadow,transform,opacity] {testSpace ? 'bg-emerald-400 text-black' : 'bg-white/10 text-white hover:bg-white/15'}"
							>
								<span class="w-12 h-1 rounded-full bg-current opacity-70"></span>
							</button>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">{uiText("ui.94baaaaae71fcba2")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.099b320ce565cad2")}</div>
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
							<div class="text-sm font-medium text-fg">{uiText("ui.d88d389a8cad5423")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.8aaba0162a203e73")}</div>
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
							<div class="text-sm font-medium text-fg">{uiText("ui.54fb011ca72a6881")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.d5ece5a1bfe5177b")}</div>
						</div>
						<div class="flex items-center gap-1.5">
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "emerald")}
								class="w-6 h-6 rounded-full bg-emerald-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'emerald' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title={uiText("ui.dc19d29e20082151")}
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "cyan")}
								class="w-6 h-6 rounded-full bg-cyan-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'cyan' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title={uiText("ui.23e30f4e242d53d0")}
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "white")}
								class="w-6 h-6 rounded-full bg-white border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'white' ? 'border-emerald-400 scale-110' : 'border-transparent opacity-60'}"
								title={uiText("ui.feab5ef2c4df4ea5")}
							></button>
							<button
								onclick={() => clientMods.updateModuleSetting("keystrokes", "colorMode", "chroma")}
								class="w-6 h-6 rounded-full bg-gradient-to-tr from-pink-500 via-amber-400 to-cyan-400 border-2 {clientMods.moduleSettings.keystrokes.colorMode === 'chroma' ? 'border-white scale-110' : 'border-transparent opacity-60'}"
								title={uiText("ui.a8e3e0394e17b20e")}
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
							<span class="text-cyan-400">{uiText("ui.29d3cbd4a3e384b1")} {clientMods.moduleSettings.cps.suffix}</span>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">{uiText("ui.8307f129f015a7e1")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.7648d19e288933ee")}</div>
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
							<div class="text-sm font-medium text-fg">{uiText("ui.5e501de43d6b2045")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.f0e3b9f131cd6466")}</div>
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
							<span>{uiText("ui.ba2d8673aef57cb0")}</span>
						{:else}
							<span>{clientMods.moduleSettings.toggleSprint.text}</span>
						{/if}
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">{uiText("ui.75195ba3203af76b")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.8289d9ffc738a437")}</div>
						</div>
						<div class="flex items-center gap-1.5 p-1 rounded-xl bg-fg/5 border border-fg/10">
							<button
								onclick={() => clientMods.updateModuleSetting("toggleSprint", "style", "text")}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.toggleSprint.style === 'text' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								{uiText("ui.6df3d6661c095ce7")}
							</button>
							<button
								onclick={() => clientMods.updateModuleSetting("toggleSprint", "style", "icon")}
								class="px-3 py-1 rounded-lg text-xs font-medium transition-colors {clientMods.moduleSettings.toggleSprint.style === 'icon' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-fg/60 hover:text-fg'}"
							>
								{uiText("ui.665ba1908fbc76bf")}
							</button>
						</div>
					</div>

					{#if clientMods.moduleSettings.toggleSprint.style === "text"}
						<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
							<div>
								<div class="text-sm font-medium text-fg">{uiText("ui.f3fd3f578dd90709")}</div>
								<div class="text-xs text-fg/50">{uiText("ui.69d8c06e2135a860")}</div>
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
						<span class="text-fg/40">{uiText("ui.4aec8dbb389c7792")} </span>
						<span class="px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-400 font-bold border border-emerald-500/40">{uiText("ui.6c598b8e463df413")}</span>
						<span class="text-fg/40"> {uiText("ui.a54e695390ed9d59")}</span>
					</div>
				</div>

				<div class="space-y-4">
					<div class="flex items-center justify-between p-3.5 rounded-2xl bg-bg-elevated/50 border border-fg/10">
						<div>
							<div class="text-sm font-medium text-fg">{uiText("ui.d5b0d7845a9040aa")}</div>
							<div class="text-xs text-fg/50">{uiText("ui.61d0fe64b1e840e2")}</div>
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
					<div class="text-sm font-medium text-fg">{uiText("ui.8e8328d13809fa1c")}</div>
					<p class="text-xs text-fg/50">{uiText("ui.4103a52427b679f6")}</p>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-between px-6 py-4 border-t border-fg/10 bg-bg">
			<span class="text-xs text-fg/40">{uiText("ui.bfde85ea50c5a998")}</span>
			<button
				onclick={handleClose}
				class={launcherButton({ variant: "primary", size: "sm", class: "" })}
			>
				{uiText("ui.2801cb53f1fa6f08")}
			</button>
		</div>
	</div>
</div>
