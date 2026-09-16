<script lang="ts">
	import { 
		GripVertical, Eye, EyeOff, ArrowUp, ArrowDown, RotateCcw, 
		Check, Sparkles, LayoutGrid, Package, Boxes, Clock, 
		Newspaper, Play, MoveVertical, Smartphone, Monitor,
		Gamepad2, AlignJustify, Grid3x3, Signal
	} from "lucide-svelte";
	import { dndzone } from "svelte-dnd-action";
	import confetti from "canvas-confetti";
	import { layoutStore, type SectionId, type LayoutSectionItem, type LayoutPreset } from "$lib/stores/layout.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import Heading from "$lib/components/ui/Heading.svelte";
	import Card from "$lib/components/ui/Card.svelte";
	import Badge from "$lib/components/ui/Badge.svelte";

	let items = $state<LayoutSectionItem[]>(layoutStore.sections.map(s => ({ ...s })));
	let selectedPreview = $state<SectionId | null>(null);

	$effect(() => {
		items = layoutStore.sections.map(s => ({ ...s }));
	});

	const presets: Array<{ id: LayoutPreset; label: string; desc: string; icon: typeof Gamepad2 }> = [
		{ id: "gamer",   label: "Modo Gamer",    desc: "Foco em play rápido",  icon: Gamepad2 },
		{ id: "compact", label: "Compacto",       desc: "Minimalista",          icon: Smartphone },
		{ id: "full",    label: "Completo",        desc: "Tudo visível",         icon: Grid3x3 },
	];

	function applyPreset(id: LayoutPreset) {
		layoutStore.applyPreset(id);
		items = layoutStore.sections.map(s => ({ ...s }));
		try {
			confetti({ particleCount: 40, spread: 50, origin: { y: 0.5 } });
		} catch {}
		toast(`Preset "${presets.find(p => p.id === id)?.label}" aplicado!`, "success");
	}

	function handleDndConsider(e: CustomEvent<{ items: LayoutSectionItem[] }>) {
		items = e.detail.items;
	}

	function handleDndFinalize(e: CustomEvent<{ items: LayoutSectionItem[] }>) {
		items = e.detail.items;
		layoutStore.reorderSections(items);
		toast("Seções reordenadas!", "info");
	}

	function resetLayout() {
		layoutStore.resetToDefaults();
		items = layoutStore.sections.map(s => ({ ...s }));
		try {
			confetti({
				particleCount: 50,
				spread: 60,
				origin: { y: 0.6 }
			});
		} catch {}
		toast("Layout restaurado para a ordem padrão!", "success");
	}

	function getSectionIcon(id: SectionId) {
		switch (id) {
			case "hero": return Play;
			case "quickInstances": return Boxes;
			case "favoriteServer": return Signal;
			case "curatedPacks": return Package;
			case "gamingStats": return Clock;
			case "newsFeed": return Newspaper;
			default: return LayoutGrid;
		}
	}
</script>

<div class="h-full flex flex-col gap-6 select-none overflow-y-auto custom-scrollbar pr-1 pb-10">
	
	<!-- Header Bar -->
	<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-[#141518]/80 backdrop-blur-xl border border-white/5 p-6 rounded-3xl shadow-xl">
		<div class="space-y-1">
			<div class="flex items-center gap-3">
				<div class="p-2.5 rounded-2xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
					<LayoutGrid class="w-5 h-5" />
				</div>
				<div>
					<Heading class="text-xl font-bold">Organizador de Layout</Heading>
					<p class="text-xs text-white/50">Personalize, ordene e arraste as seções da tela inicial para ter um painel exclusivo.</p>
				</div>
			</div>
		</div>

		<div class="flex items-center gap-3 self-end sm:self-center">
			<!-- Preset Buttons -->
			<div class="flex items-center gap-1.5 bg-bg-subtle border border-white/5 p-1 rounded-2xl">
				{#each presets as preset}
					{@const Icon = preset.icon}
					<button
						type="button"
						onclick={() => applyPreset(preset.id)}
						title={preset.desc}
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-[11px] font-bold transition-all cursor-pointer text-white/50 hover:text-white hover:bg-white/10"
					>
						<Icon class="w-3.5 h-3.5" />
						{preset.label}
					</button>
				{/each}
			</div>
			<Button variant="secondary" onclick={resetLayout} class="gap-2 text-xs">
				<RotateCcw class="w-3.5 h-3.5" />
				<span>Redefinir</span>
			</Button>
			<a href="/" class="inline-flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-emerald-500 hover:bg-emerald-400 text-black text-xs font-black transition-all shadow-lg hover:shadow-emerald-500/20">
				<span>Ver no Início</span>
				<Check class="w-3.5 h-3.5" />
			</a>
		</div>
	</div>

	<!-- Main Workspace Grid -->
	<div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
		
		<!-- Left: Draggable Reorder Section List (7 cols) -->
		<div class="lg:col-span-7 space-y-4">
			<div class="flex items-center justify-between px-2">
				<div class="flex items-center gap-2 text-xs font-bold text-white/70 uppercase tracking-wider">
					<MoveVertical class="w-4 h-4 text-emerald-400" />
					<span>Seções do Dashboard ({layoutStore.sections.filter(s => s.enabled).length} ativas)</span>
				</div>
				<span class="text-[11px] text-white/40">Arraste ou use as setas</span>
			</div>

			<!-- DnD Reorder Zone -->
			<div 
				use:dndzone={{ items, dragDisabled: false, flipDurationMs: 200 }}
				onconsider={handleDndConsider}
				onfinalize={handleDndFinalize}
				class="space-y-2.5 min-h-[300px]"
			>
				{#each items as section, index (section.id)}
					{@const Icon = getSectionIcon(section.id)}
					{@const isSelected = selectedPreview === section.id}
					<div
						onclick={() => selectedPreview = section.id}
						onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectedPreview = section.id; }}
						role="button"
						tabindex="0"
						class="group relative flex items-center justify-between gap-4 p-4 rounded-2xl transition-all duration-200 cursor-grab active:cursor-grabbing border
							border-white/5 hover:border-white/15 bg-[#16171b]
							{isSelected ? 'ring-2 ring-emerald-500/50 bg-[#1a1b20]' : ''}
							{!section.enabled ? 'opacity-60 bg-[#121316]' : 'shadow-md'}
						"
					>
						<!-- Left Drag Handle + Info -->
						<div class="flex items-center gap-3.5 min-w-0">
							<div class="p-2 text-white/30 group-hover:text-white/70 transition-colors">
								<GripVertical class="w-4 h-4" />
							</div>

							<div class="w-8 h-8 rounded-xl flex items-center justify-center shrink-0 {section.enabled ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-white/5 text-white/30 border border-white/5'}">
								<Icon class="w-4 h-4" />
							</div>

							<div class="min-w-0">
								<div class="flex items-center gap-2">
									<span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-white/50 border border-white/5">#{index + 1}</span>
									<h4 class="text-xs font-bold text-white truncate">{section.title}</h4>
									{#if !section.enabled}
										<span class="text-[9px] font-semibold px-2 py-0.5 rounded-full bg-red-500/10 text-red-400 border border-red-500/20">Oculto</span>
									{:else if section.width === 'half'}
										<span class="text-[9px] font-semibold px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">Meia Largura</span>
									{/if}
								</div>
								<p class="text-[11px] text-white/40 truncate mt-0.5 max-w-sm">{section.description}</p>
							</div>
						</div>

						<!-- Right Controls -->
						<div class="flex items-center gap-2 shrink-0">
							
							<!-- Width Toggle (for cards that support half-width) -->
							{#if section.id === "gamingStats" || section.id === "favoriteServer" || section.id === "newsFeed"}
								<button
									type="button"
									title={section.width === 'full' ? 'Mudar para meia largura (lado a lado)' : 'Mudar para largura total'}
									onclick={(e) => {
										e.stopPropagation();
										layoutStore.setSectionWidth(section.id, section.width === 'full' ? 'half' : 'full');
									}}
									class="px-2.5 py-1.5 rounded-xl border border-white/5 hover:border-white/20 bg-white/5 text-[10px] font-medium text-white/60 hover:text-white transition-all cursor-pointer"
								>
									{section.width === 'full' ? '100%' : '50%'}
								</button>
							{/if}

							<!-- Move Up / Down Buttons -->
							<div class="flex items-center bg-white/5 rounded-xl border border-white/5 p-0.5">
								<button
									type="button"
									disabled={index === 0}
									onclick={(e) => { e.stopPropagation(); layoutStore.moveUp(section.id); }}
									title="Mover para cima"
									class="p-1.5 text-white/50 hover:text-white disabled:opacity-20 disabled:cursor-not-allowed rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
								>
									<ArrowUp class="w-3.5 h-3.5" />
								</button>
								<button
									type="button"
									disabled={index === items.length - 1}
									onclick={(e) => { e.stopPropagation(); layoutStore.moveDown(section.id); }}
									title="Mover para baixo"
									class="p-1.5 text-white/50 hover:text-white disabled:opacity-20 disabled:cursor-not-allowed rounded-lg hover:bg-white/10 transition-colors cursor-pointer"
								>
									<ArrowDown class="w-3.5 h-3.5" />
								</button>
							</div>

							<!-- Visibility Toggle Switch -->
							<button
								type="button"
								onclick={(e) => {
									e.stopPropagation();
									layoutStore.toggleSection(section.id);
								}}
								title={section.enabled ? "Ocultar seção" : "Exibir seção"}
								class="p-2 rounded-xl transition-all cursor-pointer {section.enabled ? 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 border border-emerald-500/30' : 'bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/30'}"
							>
								{#if section.enabled}
									<Eye class="w-4 h-4" />
								{:else}
									<EyeOff class="w-4 h-4" />
								{/if}
							</button>
						</div>
					</div>
				{/each}
			</div>
		</div>

		<!-- Right: Live Interactive Dashboard Layout Mockup (5 cols) -->
		<div class="lg:col-span-5 space-y-4">
			<div class="flex items-center justify-between px-2">
				<div class="flex items-center gap-2 text-xs font-bold text-white/70 uppercase tracking-wider">
					<Monitor class="w-4 h-4 text-emerald-400" />
					<span>Prévia em Tempo Real da Tela Inicial</span>
				</div>
				<span class="text-[10px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20 font-mono">Sincronizado</span>
			</div>

			<!-- Mockup Container (looks like miniature Luxmc Home) -->
			<div class="bg-[#111215] border border-white/10 rounded-3xl p-4 shadow-2xl space-y-3 relative overflow-hidden">
				
				<!-- Mockup Window Header -->
				<div class="flex items-center justify-between pb-2 border-b border-white/5 px-1">
					<div class="flex items-center gap-1.5">
						<span class="w-2.5 h-2.5 rounded-full bg-red-500/70"></span>
						<span class="w-2.5 h-2.5 rounded-full bg-yellow-500/70"></span>
						<span class="w-2.5 h-2.5 rounded-full bg-emerald-500/70"></span>
					</div>
					<span class="text-[10px] text-white/30 font-medium">Dashboard Preview</span>
				</div>

				<!-- Dynamic Rendered Mini Cards -->
				<div class="space-y-2.5 min-h-[420px]">
					{#each layoutStore.sections as section (section.id)}
						{#if section.enabled}
							{@const Icon = getSectionIcon(section.id)}
							{@const isSelected = selectedPreview === section.id}
							
							<div 
								onclick={() => selectedPreview = section.id}
								role="button"
								tabindex="0"
								onkeydown={(e) => { if (e.key === 'Enter') selectedPreview = section.id; }}
								class="p-3 rounded-xl border transition-all cursor-pointer relative overflow-hidden group
									{isSelected ? 'border-emerald-500/60 bg-emerald-500/10 ring-1 ring-emerald-500/50' : 'border-white/5 bg-[#18191c] hover:border-white/20'}
									{section.id === 'hero' ? 'min-h-[70px] bg-gradient-to-r from-[#18191c] via-[#212228] to-[#18191c]' : ''}
								"
							>
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Icon class="w-3.5 h-3.5 {isSelected ? 'text-emerald-400' : 'text-white/60'}" />
										<span class="text-[11px] font-bold text-white group-hover:text-emerald-400 transition-colors">{section.title}</span>
									</div>
									<span class="text-[9px] font-mono text-white/30">{section.width === 'half' ? '50%' : '100%'}</span>
								</div>

								{#if section.id === 'hero'}
									<div class="mt-2 flex items-center justify-between text-[10px] text-white/40">
										<span>Instância Ativa: <strong class="text-white/70">Vanilla Perfected</strong></span>
										<span class="bg-emerald-400 text-black font-black px-2 py-0.5 rounded text-[9px]">JOGAR</span>
									</div>
								{:else if section.id === 'quickInstances'}
									<div class="mt-2 grid grid-cols-3 gap-1.5">
										<div class="h-6 rounded bg-white/5 border border-white/5"></div>
										<div class="h-6 rounded bg-white/5 border border-white/5"></div>
										<div class="h-6 rounded bg-white/5 border border-white/5"></div>
									</div>
								{:else if section.id === 'curatedPacks'}
									<div class="mt-2 grid grid-cols-2 gap-1.5">
										<div class="h-7 rounded bg-white/5 border border-white/5"></div>
										<div class="h-7 rounded bg-white/5 border border-white/5"></div>
									</div>
								{:else}
									<div class="mt-2 h-4 rounded bg-white/5 w-3/4"></div>
								{/if}
							</div>
						{/if}
					{/each}

					{#if layoutStore.sections.filter(s => s.enabled).length === 0}
						<div class="h-48 flex flex-col items-center justify-center text-center p-6 text-white/30 gap-2">
							<EyeOff class="w-8 h-8 opacity-40" />
							<p class="text-xs">Todas as seções estão ocultas.</p>
							<button onclick={resetLayout} class="text-[11px] text-emerald-400 hover:underline font-bold mt-1 cursor-pointer">
								Restaurar Seções
							</button>
						</div>
					{/if}
				</div>

				<div class="pt-3 border-t border-white/5 flex items-center justify-between text-[10px] text-white/40">
					<span>Mudanças salvas automaticamente</span>
					<span class="text-emerald-400 font-medium">Luxmc v1.7.1</span>
				</div>
			</div>
		</div>

	</div>

</div>
