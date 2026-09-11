<script lang="ts">
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
	class="fixed inset-0 z-[999999] bg-black/80 backdrop-blur-sm flex items-center justify-center p-6"
	transition:fade={{ duration: 150 }}
	onclick={onClose}
>
	<div
		class="bg-[#18191c] border border-white/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-5"
		transition:scale={{ start: 0.95, duration: 150 }}
		onclick={(e) => e.stopPropagation()}
	>
		<div class="flex items-center justify-between border-b border-white/5 pb-3">
			<div class="flex items-center gap-2.5">
				<div class="w-8 h-8 rounded-xl bg-[#6c5ce7]/10 text-[#a29bfe] flex items-center justify-center">
					<Box class="w-4 h-4" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-white">Escolha a Instância</h3>
					<p class="text-[11px] text-white/40 font-medium">Onde você deseja instalar este {selectedType}?</p>
				</div>
			</div>
			<button
				type="button"
				class="text-white/40 hover:text-white p-1 rounded-lg hover:bg-white/5 transition-colors cursor-pointer"
				onclick={onClose}
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		<div class="flex items-center gap-3 bg-[#131418] p-3 rounded-2xl border border-white/5">
			<div class="w-10 h-10 rounded-xl bg-[#222328] border border-white/10 overflow-hidden shrink-0 flex items-center justify-center">
				{#if item.iconUrl}
					<img src={item.iconUrl} alt={item.title} class="w-full h-full object-cover" />
				{:else}
					<Cpu class="w-5 h-5 text-[#caa97c]" />
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<h4 class="text-xs font-extrabold text-white truncate">{item.title}</h4>
				<span class="text-[10px] text-white/40">{item.author || "Autor"} • {selectedType}</span>
			</div>
		</div>

		<div class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
			{#each profiles.list as p}
				<button
					type="button"
					class="w-full text-left p-3 rounded-2xl border transition-all cursor-pointer flex items-center justify-between gap-3 {chosenInstanceId === p.id ? 'bg-[#6c5ce7]/15 border-[#6c5ce7] shadow-sm' : 'bg-[#131418] border-white/5 hover:border-white/20'}"
					onclick={() => chosenInstanceId = p.id}
				>
					<div class="flex items-center gap-3 min-w-0">
						<div class="w-9 h-9 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center overflow-hidden shrink-0">
							{#if p.icon && p.icon !== "default" && (p.icon.startsWith("http") || p.icon.startsWith("data:"))}
								<img src={p.icon} alt={p.name} class="w-full h-full object-cover" />
							{:else}
								<Box class="w-4 h-4 text-white/60" />
							{/if}
						</div>
						<div class="min-w-0">
							<h5 class="text-xs font-bold text-white truncate">{p.name}</h5>
							<div class="flex items-center gap-2 text-[10px] text-white/40 mt-0.5">
								<span class="font-mono">{p.mcVersion}</span>
								<span>•</span>
								<span class="uppercase font-semibold text-[#caa97c]">{p.loader}</span>
								{#if p.modCount}
									<span>•</span>
									<span>{p.modCount} mods</span>
								{/if}
							</div>
						</div>
					</div>

					<div class="w-5 h-5 rounded-full border flex items-center justify-center shrink-0 {chosenInstanceId === p.id ? 'border-[#6c5ce7] bg-[#6c5ce7] text-white' : 'border-white/20 bg-transparent'}">
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
				class="px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white/70 hover:text-white text-xs font-semibold transition-all cursor-pointer"
				onclick={onClose}
			>
				Cancelar
			</button>
			<button
				type="button"
				class="px-5 py-2.5 rounded-xl bg-[#6c5ce7] hover:bg-[#5b4cdb] text-white font-extrabold text-xs flex items-center gap-2 transition-all cursor-pointer active:scale-95 disabled:opacity-50 shadow-md shadow-[#6c5ce7]/20"
				disabled={!chosenInstanceId}
				onclick={onConfirm}
			>
				<Download class="w-4 h-4" />
				<span>Confirmar e Instalar</span>
			</button>
		</div>
	</div>
</div>
