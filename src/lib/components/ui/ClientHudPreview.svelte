<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { quintOut } from "svelte/easing";
	import { fade } from "svelte/transition";
	import { X, Sliders, Check } from "lucide-svelte";
	import { clientMods } from "$lib/stores/clientMods.svelte";

	let { onClose }: { onClose: () => void } = $props();
</script>

<div
	class="fixed inset-0 z-[70] flex flex-col justify-between bg-black/85 backdrop-blur-sm p-6 select-none"
	transition:fade={{ easing: quintOut, duration: 220 }}
>
	<!-- Top Bar -->
	<div class="flex items-center justify-between px-6 py-3 rounded-2xl bg-bg/90 border border-fg/15 shadow-xl max-w-4xl mx-auto w-full">
		<div class="flex items-center gap-3">
			<span class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse"></span>
			<span class="text-xs font-bold uppercase tracking-wider text-emerald-400">{uiText("ui.791a942539ede460")}</span>
			<span class="text-xs text-fg/50">{uiText("ui.274f43dca07aa137")}</span>
		</div>

		<div class="flex items-center gap-3">
			<span class="text-xs text-fg/50">{uiText("ui.0e8560c03349cb6d")} <kbd class="px-1.5 py-0.5 rounded bg-fg/10 text-fg font-mono text-[10px]">{uiText("ui.52f878edb34fa14f")}</kbd> {uiText("ui.8bf86e3a9ce66fb5")}</span>
			<button
				onclick={onClose}
				class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-1.5" })}
			>
				<Check class="w-3.5 h-3.5" />
				<span>{uiText("ui.6e91752edf51a9a4")}</span>
			</button>
		</div>
	</div>

	<!-- In-Game Elements Simulation Area -->
	<div class="relative flex-1 w-full my-4 border border-dashed border-white/20 rounded-3xl p-6 pointer-events-none">
		<!-- Direction HUD (Top Center) -->
		{#if clientMods.config.directionHud}
			<div class="absolute top-4 left-1/2 -translate-x-1/2 flex flex-col items-center p-2 rounded-xl bg-black/60 border border-white/10 font-mono text-xs">
				<div class="flex items-center gap-2">
					<span class="text-white/40">{uiText("ui.4aec8dbb389c7792")} </span>
					<span class="px-2 py-0.5 rounded bg-emerald-500/30 text-emerald-400 font-bold border border-emerald-500/40">{uiText("ui.6c598b8e463df413")}</span>
					<span class="text-white/40"> {uiText("ui.a54e695390ed9d59")}</span>
				</div>
				{#if clientMods.moduleSettings.directionHud.showBiome}
					<span class="text-[10px] text-fg/50 mt-0.5">{uiText("ui.3df4c85812881d88")}</span>
				{/if}
			</div>
		{/if}

		<!-- FPS & CPS (Top Left) -->
		<div class="absolute top-4 left-6 flex flex-col gap-1.5 font-mono text-xs">
			{#if clientMods.config.fps}
				<div class="px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-emerald-400 font-bold">
					{uiText("ui.3b2100547335b31f")}
				</div>
			{/if}

			{#if clientMods.config.cps}
				<div class="px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-cyan-400 font-bold">
					14 {clientMods.moduleSettings.cps.suffix} {#if clientMods.moduleSettings.cps.showRight}{uiText("ui.07c8ed80af2d144b")}{/if}
				</div>
			{/if}

			{#if clientMods.config.pingDisplay}
				<div class="px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-yellow-300 font-bold">
					{uiText("ui.8bb24d26fb844b85")}
				</div>
			{/if}

			{#if clientMods.config.coordinates}
				<div class="px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-white/80 text-[11px]">
					{uiText("ui.4e68edaf5a15bde7")}
				</div>
			{/if}
		</div>

		<!-- Potion Effects (Top Right) -->
		{#if clientMods.config.potionEffects}
			<div class="absolute top-4 right-6 flex flex-col items-end gap-1.5 font-mono text-xs">
				<div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-red-400">
					<span>{uiText("ui.ce8f8be4fe7635b9")}</span>
					<span class="font-bold text-fg">2:45</span>
				</div>
				<div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 text-cyan-400">
					<span>{uiText("ui.63a60e720925c308")}</span>
					<span class="font-bold text-fg">1:12</span>
				</div>
			</div>
		{/if}

		<!-- ToggleSprint (Bottom Left) -->
		{#if clientMods.config.toggleSprint}
			<div class="absolute bottom-6 left-6 px-3 py-1.5 rounded-lg bg-black/60 border border-white/10 font-mono text-xs font-bold text-emerald-400">
				{clientMods.moduleSettings.toggleSprint.text}
			</div>
		{/if}

		<!-- Keystrokes (Bottom Right, Left of Armor HUD) -->
		{#if clientMods.config.keystrokes}
			<div class="absolute bottom-6 right-36 flex flex-col items-center gap-1 p-3 rounded-2xl bg-black/60 border border-white/10">
				<div class="w-8 h-8 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-xs text-white">W</div>
				<div class="flex items-center gap-1">
					<div class="w-8 h-8 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-xs text-white">A</div>
					<div class="w-8 h-8 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-xs text-white">S</div>
					<div class="w-8 h-8 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-xs text-white">D</div>
				</div>
				<div class="flex items-center gap-1 w-full">
					<div class="flex-1 h-7 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-[10px] text-white">{uiText("ui.a0b0cf417d0b4c7d")}</div>
					<div class="flex-1 h-7 rounded-lg bg-white/15 flex items-center justify-center font-bold font-mono text-[10px] text-white">{uiText("ui.02cb6fa562753c7e")}</div>
				</div>
				<div class="w-full h-4 rounded-lg bg-white/15 flex items-center justify-center">
					<span class="w-8 h-0.5 rounded-full bg-white/50"></span>
				</div>
			</div>
		{/if}

		<!-- Armor HUD (Bottom Right) -->
		{#if clientMods.config.armorHud}
			<div
				class="absolute bottom-6 right-6 flex flex-col gap-1.5 p-2 rounded-2xl bg-black/60 border border-white/10"
				style="transform: scale({clientMods.moduleSettings.armorHud.scale});"
			>
				<div class="flex items-center gap-2">
					<div class="w-6 h-6 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-xs">🪖</div>
					<span class="text-[11px] font-mono font-bold text-emerald-400">100%</span>
				</div>
				<div class="flex items-center gap-2">
					<div class="w-6 h-6 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-xs">👕</div>
					<span class="text-[11px] font-mono font-bold text-emerald-400">78%</span>
				</div>
				<div class="flex items-center gap-2">
					<div class="w-6 h-6 rounded bg-blue-500/20 border border-blue-400/40 flex items-center justify-center text-xs">👖</div>
					<span class="text-[11px] font-mono font-bold text-yellow-400">54%</span>
				</div>
				<div class="flex items-center gap-2">
					<div class="w-6 h-6 rounded bg-blue-500/20 border border-red-500 text-red-400 animate-pulse flex items-center justify-center text-xs">👢</div>
					<span class="text-[11px] font-mono font-extrabold text-red-400 animate-pulse">18%</span>
				</div>
				<div class="flex items-center gap-2 pt-1 border-t border-white/10">
					<div class="w-6 h-6 rounded bg-amber-500/20 border border-amber-400/40 flex items-center justify-center text-xs">🗡️</div>
					<span class="text-[11px] font-mono font-bold text-emerald-400">92%</span>
				</div>
			</div>
		{/if}
	</div>

	<!-- Bottom Hint -->
	<div class="text-center text-xs text-fg/40">
		{uiText("ui.2c906f9d68d836ce")}
	</div>
</div>
