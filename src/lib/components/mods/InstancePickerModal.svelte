<script lang="ts">
    import { focusTrap } from "$lib/utils/focusTrap";
	import { Box, Check, Download, X, Cpu } from "lucide-svelte";
	import { fade, scale } from "svelte/transition";
	import type { ModSearchResultItem } from "$lib/api";
	import { profiles } from "$lib/stores/profiles.svelte";

	let {
		item,
		selectedType = "Mod",
		chosenInstanceId = $bindable(""),
		onConfirm,
		onClose
	}: {
		item: ModSearchResultItem;
		selectedType?: string;
		chosenInstanceId?: string;
		onConfirm: () => void;
		onClose: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-bg-overlay/80 backdrop-blur-sm flex items-center justify-center p-6"
	transition:fade={{ duration: 150 }}
	onclick={onClose}
>
	<div
		role="dialog" aria-modal="true" aria-label="Escolha a instância" tabindex="-1" use:focusTrap
        class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-5"
		transition:scale={{ start: 0.95, duration: 150 }}
		onclick={(e) => e.stopPropagation()}
        onkeydown={(event) => { if (event.key === "Escape") { event.stopPropagation(); onClose(); } }}
	>
		<div class="flex items-center justify-between border-b border-fg/5 pb-3">
			<div class="flex items-center gap-2.5">
				<div class="w-8 h-8 rounded-xl bg-brand-500/10 text-brand-400 flex items-center justify-center">
					<Box class="w-4 h-4" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-fg">Escolha a Instância</h3>
					<p class="text-[11px] text-fg/40 font-medium">Onde você deseja instalar este {selectedType}?</p>
				</div>
			</div>
			<button
				type="button"
				class="text-fg/40 hover:text-fg p-1 rounded-lg hover:bg-fg/5 transition-colors cursor-pointer"
				aria-label="Fechar seleção de instância" onclick={onClose}
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		<div class="flex items-center gap-3 bg-bg-elevated p-3 rounded-2xl border border-fg/5">
			<div class="w-10 h-10 rounded-xl bg-bg-subtle border border-fg/10 overflow-hidden shrink-0 flex items-center justify-center">
				{#if item.iconUrl}
					<img src={item.iconUrl} alt={item.title} class="w-full h-full object-cover" />
				{:else}
					<Cpu class="w-5 h-5 text-brand-400" />
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<h4 class="text-xs font-extrabold text-fg truncate">{item.title}</h4>
				<span class="text-[10px] text-fg/40">{item.author || "Autor"} • {selectedType}</span>
			</div>
		</div>

		<div class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
			{#each profiles.list as p}
				<button
					type="button"
					class="w-full text-left p-3 rounded-2xl border transition-all cursor-pointer flex items-center justify-between gap-3 {chosenInstanceId === p.id ? 'bg-brand-500/15 border-brand-500 shadow-sm' : 'bg-bg-elevated border-fg/5 hover:border-fg/20'}"
					onclick={() => chosenInstanceId = p.id}
				>
					<div class="flex items-center gap-3 min-w-0">
						<div class="w-9 h-9 rounded-xl bg-fg/5 border border-fg/10 flex items-center justify-center overflow-hidden shrink-0">
							{#if p.icon && p.icon !== "default" && (p.icon.startsWith("http") || p.icon.startsWith("data:"))}
								<img src={p.icon} alt={p.name} class="w-full h-full object-cover" />
							{:else}
								<Box class="w-4 h-4 text-fg/60" />
							{/if}
						</div>
						<div class="min-w-0">
							<h5 class="text-xs font-bold text-fg truncate">{p.name}</h5>
							<div class="flex items-center gap-2 text-[10px] text-fg/40 mt-0.5">
								<span class="font-mono">{p.mcVersion}</span>
								<span>•</span>
								<span class="uppercase font-semibold text-brand-400">{p.loader}</span>
								{#if p.modCount}
									<span>•</span>
									<span>{p.modCount} mods</span>
								{/if}
							</div>
						</div>
					</div>

					<div class="w-5 h-5 rounded-full border flex items-center justify-center shrink-0 {chosenInstanceId === p.id ? 'border-brand-500 bg-brand-500 text-fg' : 'border-fg/20 bg-transparent'}">
						{#if chosenInstanceId === p.id}
							<Check class="w-3 h-3 stroke-[3]" />
						{/if}
					</div>
				</button>
			{/each}
		</div>

		<div class="flex items-center justify-end gap-3 pt-2">
			<button
				type="button"
				class="px-4 py-2.5 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/70 hover:text-fg text-xs font-semibold transition-all cursor-pointer"
				onclick={onClose}
			>
				Cancelar
			</button>
			<button
				type="button"
				class="px-5 py-2.5 rounded-xl bg-brand-500 hover:bg-brand-500 text-fg font-extrabold text-xs flex items-center gap-2 transition-all cursor-pointer active:scale-[0.98] disabled:opacity-50 shadow-md shadow-elevated"
				disabled={!chosenInstanceId}
				onclick={onConfirm}
			>
				<Download class="w-4 h-4" />
				<span>Confirmar e Instalar</span>
			</button>
		</div>
	</div>
</div>
