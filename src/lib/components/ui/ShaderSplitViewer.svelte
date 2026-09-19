<script lang="ts">
	import { Sparkles, Eye, Sliders, Check } from "lucide-svelte";

	type ShaderPreset = {
		name: string;
		author: string;
		fpsCost: string;
		beforeImg: string;
		afterImg: string;
		features: string[];
	};

	const presets: ShaderPreset[] = [
		{
			name: "Complementary Reimagined",
			author: "EminGT",
			fpsCost: "Leve (~10% FPS)",
			beforeImg: "/vanilla_banner.png",
			afterImg: "https://cdn.modrinth.com/data/HVnmMxH1/images/26327bef581206670288bf7e1b1b5f411291f793.jpeg",
			features: ["Água Wavy PBR", "Godrays Suaves", "Sombras de Alta Resolução", "Auroras Boreais"]
		},
		{
			name: "BSL Shaders Pro",
			author: "Capt Tatsu",
			fpsCost: "Médio (~18% FPS)",
			beforeImg: "/vanilla_banner.png",
			afterImg: "https://cdn.modrinth.com/data/Q1vvjJYV/images/01e67d2bc1cfb34d2b80790eddcc836d00de5e55.jpeg",
			features: ["Oclusão de Ambiente", "Profundidade de Campo", "Iluminação Quente", "Céu Volumétrico"]
		},
		{
			name: "Photon Shaders (PBR & Realismo)",
			author: "SixthSurge",
			fpsCost: "Médio (~15% FPS)",
			beforeImg: "/vanilla_banner.png",
			afterImg: "https://cdn.modrinth.com/data/lLqFfGNs/images/53158735be49a61e603c276c66788af48e5a9503.png",
			features: ["Traçado de Raios Parcial", "Reflexos em Tempo Real", "Vento na Vegetação", "Física de Nuvens"]
		}
	];

	let selectedIdx = $state(0);
	let splitPercent = $state(50);
	let isDragging = $state(false);
	let containerEl: HTMLDivElement | null = $state(null);

	const currentPreset = $derived(presets[selectedIdx]);

	function handleMove(clientX: number) {
		if (!containerEl) return;
		const rect = containerEl.getBoundingClientRect();
		const x = clientX - rect.left;
		const pct = Math.max(0, Math.min(100, (x / rect.width) * 100));
		splitPercent = Math.round(pct);
	}

	function onPointerDown(e: PointerEvent) {
		isDragging = true;
		handleMove(e.clientX);
	}

	function onPointerMove(e: PointerEvent) {
		if (isDragging) {
			handleMove(e.clientX);
		}
	}

	function onPointerUp() {
		isDragging = false;
	}

	function onKeyDown(e: KeyboardEvent) {
		if (e.key === "ArrowLeft") {
			splitPercent = Math.max(0, splitPercent - 5);
		} else if (e.key === "ArrowRight") {
			splitPercent = Math.min(100, splitPercent + 5);
		}
	}
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="bg-bg-elevated border border-fg/5 rounded-3xl p-5 flex flex-col gap-4 shadow-xl">
	<!-- Top info & selector -->
	<div class="flex flex-wrap items-center justify-between gap-3">
		<div class="flex items-center gap-2.5">
			<div class="w-8 h-8 rounded-xl bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-400">
				<Sparkles class="w-4 h-4" />
			</div>
			<div>
				<h4 class="text-xs font-bold text-fg">Comparador Visual de Shaders (Split-View)</h4>
				<p class="text-[10px] text-fg/40">Arraste o controle central para comparar Vanilla vs Shader</p>
			</div>
		</div>

		<!-- Presets Selector -->
		<div class="flex items-center bg-bg-subtle p-1 rounded-xl border border-fg/5 gap-1">
			{#each presets as preset, idx}
				<button 
					type="button" 
					class="px-3 py-1 rounded-lg text-[10px] font-bold transition-all cursor-pointer {selectedIdx === idx ? 'bg-bg-overlay text-fg shadow-sm border border-fg/10' : 'text-fg/40 hover:text-fg'}"
					onclick={() => selectedIdx = idx}
				>
					{preset.name}
				</button>
			{/each}
		</div>
	</div>

	<!-- Interactive Split Viewer Canvas -->
	<div 
		bind:this={containerEl}
		onpointerdown={onPointerDown}
		onkeydown={onKeyDown}
		class="relative w-full h-64 md:h-80 rounded-2xl overflow-hidden select-none cursor-ew-resize border border-fg/10 shadow-inner group"
		role="slider"
		aria-label="Comparação de Shaders"
		aria-valuenow={splitPercent}
		aria-valuemin={0}
		aria-valuemax={100}
		tabindex="0"
	>
		<!-- Right Image: Shader Enhanced -->
		<div class="absolute inset-0 bg-cover bg-center" style="background-image: url('{currentPreset.afterImg}'); filter: saturate(1.2) contrast(1.05);">
			<div class="absolute top-3 right-3 bg-bg-overlay/60 backdrop-blur-md px-2.5 py-1 rounded-lg text-[10px] font-black text-purple-300 border border-purple-500/30">
				COM SHADER
			</div>
		</div>

		<!-- Left Image: Vanilla (Clipped) -->
		<div 
			class="absolute inset-0 bg-cover bg-center overflow-hidden" 
			style="clip-path: polygon(0 0, {splitPercent}% 0, {splitPercent}% 100%, 0 100%); background-image: url('{currentPreset.beforeImg}'); filter: brightness(0.9) contrast(0.95);"
		>
			<div class="absolute top-3 left-3 bg-bg-overlay/60 backdrop-blur-md px-2.5 py-1 rounded-lg text-[10px] font-black text-fg/70 border border-fg/10">
				PADRÃO (VANILLA)
			</div>
		</div>

		<!-- Divider line -->
		<div 
			class="absolute top-0 bottom-0 w-0.5 bg-fg shadow-elevated pointer-events-none"
			style="left: {splitPercent}%;"
		>
			<!-- Draggable thumb handle -->
			<div class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-8 h-8 rounded-full bg-fg text-brand-foreground shadow-xl flex items-center justify-center font-bold text-xs">
				⇄
			</div>
		</div>
	</div>

	<!-- Features footer -->
	<div class="flex flex-wrap items-center justify-between gap-3 pt-1 border-t border-fg/5 text-xs">
		<div class="flex flex-wrap items-center gap-2">
			{#each currentPreset.features as feat}
				<span class="px-2.5 py-0.5 rounded-lg bg-fg/5 border border-fg/5 text-[10px] text-fg/60 flex items-center gap-1 font-medium">
					<Check class="w-3 h-3 text-emerald-400" /> {feat}
				</span>
			{/each}
		</div>
		<div class="text-[11px] font-bold text-purple-400 bg-purple-500/10 px-3 py-1 rounded-xl border border-purple-500/20">
			Impacto: {currentPreset.fpsCost}
		</div>
	</div>
</div>
