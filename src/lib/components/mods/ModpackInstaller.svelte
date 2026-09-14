<script lang="ts">
	import { PackagePlus, Box, Loader2, X, Cpu } from "lucide-svelte";
	import { fade, scale } from "svelte/transition";
	import type { ModSearchResultItem } from "$lib/api";

	let {
		modpack,
		instanceName = $bindable(""),
		ramMb = $bindable(4096),
		isInstalling = false,
		progressText = "",
		progressPercent = 0,
		onConfirm,
		onClose
	}: {
		modpack: ModSearchResultItem;
		instanceName?: string;
		ramMb?: number;
		isInstalling?: boolean;
		progressText?: string;
		progressPercent?: number;
		onConfirm: () => void;
		onClose: () => void;
	} = $props();

	function formatDownloads(n: number): string {
		if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
		if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
		return n.toString();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-black/80 backdrop-blur-sm flex items-center justify-center p-6"
	transition:fade={{ duration: 150 }}
	onclick={() => { if (!isInstalling) onClose(); }}
>
	<div
		class="bg-[#18191c] border border-white/10 rounded-3xl p-6 max-w-lg w-full shadow-2xl space-y-5"
		transition:scale={{ start: 0.95, duration: 150 }}
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-white/5 pb-3">
			<div class="flex items-center gap-2.5">
				<div class="w-8 h-8 rounded-xl bg-[#caa97c]/10 text-[#caa97c] flex items-center justify-center">
					<PackagePlus class="w-4 h-4" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-white">Criar Instância a partir do Modpack</h3>
					<p class="text-[11px] text-white/40 font-medium">Configure a nova instância para este modpack</p>
				</div>
			</div>
			{#if !isInstalling}
				<button
					type="button"
					class="text-white/40 hover:text-white p-1 rounded-lg hover:bg-white/5 transition-colors cursor-pointer"
					onclick={onClose}
				>
					<X class="w-4 h-4" />
				</button>
			{/if}
		</div>

		<!-- Modpack Preview Card -->
		<div class="flex items-center gap-3 bg-[#131418] p-3 rounded-2xl border border-white/5">
			<div class="w-12 h-12 rounded-xl bg-[#222328] border border-white/10 overflow-hidden shrink-0 flex items-center justify-center">
				{#if modpack.iconUrl}
					<img src={modpack.iconUrl} alt={modpack.title} class="w-full h-full object-cover" />
				{:else}
					<Box class="w-6 h-6 text-[#caa97c]" />
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<h4 class="text-xs font-extrabold text-white truncate">{modpack.title}</h4>
				<p class="text-[11px] text-white/40 truncate mt-0.5">{modpack.description}</p>
				<div class="flex items-center gap-2 mt-1 text-[10px] text-white/50">
					<span class="capitalize font-semibold text-white/70">{modpack.source}</span>
					<span>•</span>
					<span>{formatDownloads(modpack.downloads)} downloads</span>
				</div>
			</div>
		</div>

		<!-- Form Fields -->
		<div class="space-y-4">
			<div>
				<label for="modpack-inst-name" class="block text-[11px] font-bold text-white/60 uppercase tracking-wider mb-1.5">
					Nome da Instância
				</label>
				<input
					id="modpack-inst-name"
					type="text"
					bind:value={instanceName}
					placeholder="Ex: Better MC, All the Mods..."
					disabled={isInstalling}
					class="w-full bg-[#131418] border border-white/10 focus:border-[#caa97c] rounded-xl px-3.5 py-2.5 text-xs text-white placeholder-white/30 focus:outline-none transition-all disabled:opacity-50"
				/>
			</div>

			<div class="bg-[#18191d] border border-emerald-500/20 rounded-xl p-3 flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<Cpu class="w-4 h-4" />
					</div>
					<div>
						<span class="text-xs font-bold text-white block">Memória RAM Automática</span>
						<span class="text-[10px] text-white/50 block">Otimizada dinamicamente com base no seu hardware e modpack</span>
					</div>
				</div>
				<span class="text-xs font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded-full border border-emerald-500/20">Auto</span>
			</div>
		</div>

		{#if isInstalling}
			<div class="bg-[#131418] border border-[#caa97c]/20 rounded-2xl p-4 space-y-3">
				<div class="flex items-center gap-3">
					<Loader2 class="w-5 h-5 text-[#caa97c] animate-spin shrink-0" />
					<div class="min-w-0 flex-1">
						<p class="text-xs font-bold text-white">Instalando Modpack...</p>
						<p class="text-[11px] text-white/60 truncate mt-0.5">{progressText || "Por favor, aguarde..."}</p>
					</div>
					{#if progressPercent > 0}
						<span class="text-xs font-mono font-bold text-[#caa97c] shrink-0">{progressPercent}%</span>
					{/if}
				</div>
				{#if progressPercent > 0}
					<div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
						<div
							class="h-full bg-gradient-to-r from-[#caa97c] via-[#e4c99c] to-[#caa97c] rounded-full transition-all duration-300 ease-out"
							style="width: {progressPercent}%"
						></div>
					</div>
				{:else if progressPercent === -1}
					<div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
						<div class="h-full bg-gradient-to-r from-[#caa97c] via-[#e4c99c] to-[#caa97c] rounded-full animate-pulse" style="width: 100%"></div>
					</div>
				{/if}
			</div>
		{/if}

		<div class="flex items-center justify-end gap-3 pt-2">
			<button
				type="button"
				class="px-4 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white/70 hover:text-white text-xs font-semibold transition-all cursor-pointer disabled:opacity-30"
				disabled={isInstalling}
				onclick={onClose}
			>
				Cancelar
			</button>
			<button
				type="button"
				class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-[#caa97c] to-[#e4c99c] hover:from-[#d5b588] hover:to-[#edd5ad] text-black font-extrabold text-xs flex items-center gap-2 transition-all cursor-pointer active:scale-95 disabled:opacity-50 shadow-md shadow-[#caa97c]/20"
				disabled={isInstalling || !instanceName.trim()}
				onclick={onConfirm}
			>
				{#if isInstalling}
					<Loader2 class="w-4 h-4 animate-spin text-black" />
					<span>Instalando...</span>
				{:else}
					<PackagePlus class="w-4 h-4" />
					<span>Criar e Baixar Instância</span>
				{/if}
			</button>
		</div>
	</div>
</div>
