<script lang="ts">
    import { focusTrap } from "$lib/utils/focusTrap";
    import { button } from "$lib/components/ui/button";
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
		onClose, onCancel, cancelling = false
	}: {
		modpack: ModSearchResultItem;
		instanceName?: string;
		ramMb?: number;
		isInstalling?: boolean;
		progressText?: string;
		progressPercent?: number;
		onConfirm: () => void;
		onClose: () => void;
        onCancel: () => void;
        cancelling?: boolean;
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
	class="fixed inset-0 z-[999999] bg-bg-overlay/80 backdrop-blur-sm flex items-center justify-center p-6"
	transition:fade={{ duration: 150 }}
	onclick={() => { if (!isInstalling) onClose(); }}
>
	<div
		role="dialog" aria-modal="true" aria-label="Instalar modpack" tabindex="-1" use:focusTrap
        class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 max-w-lg w-full shadow-2xl space-y-5"
		transition:scale={{ start: 0.95, duration: 150 }}
		onclick={(e) => e.stopPropagation()}
        onkeydown={(event) => { if (event.key === "Escape" && !isInstalling) { event.stopPropagation(); onClose(); } }}
	>
		<!-- Header -->
		<div class="flex items-center justify-between border-b border-fg/5 pb-3">
			<div class="flex items-center gap-2.5">
				<div class="w-8 h-8 rounded-xl bg-brand-400/10 text-brand-400 flex items-center justify-center">
					<PackagePlus class="w-4 h-4" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-fg">Criar Instância a partir do Modpack</h3>
					<p class="text-[11px] text-fg/40 font-medium">Configure a nova instância para este modpack</p>
				</div>
			</div>
			{#if !isInstalling}
				<button
					type="button"
					class="text-fg/40 hover:text-fg p-1 rounded-lg hover:bg-fg/5 transition-colors cursor-pointer"
					aria-label="Fechar instalação" onclick={onClose}
				>
					<X class="w-4 h-4" />
				</button>
			{/if}
		</div>

		<!-- Modpack Preview Card -->
		<div class="flex items-center gap-3 bg-bg-elevated p-3 rounded-2xl border border-fg/5">
			<div class="w-12 h-12 rounded-xl bg-bg-subtle border border-fg/10 overflow-hidden shrink-0 flex items-center justify-center">
				{#if modpack.iconUrl}
					<img src={modpack.iconUrl} alt={modpack.title} class="w-full h-full object-cover" />
				{:else}
					<Box class="w-6 h-6 text-brand-400" />
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<h4 class="text-xs font-extrabold text-fg truncate">{modpack.title}</h4>
				<p class="text-[11px] text-fg/40 truncate mt-0.5">{modpack.description}</p>
				<div class="flex items-center gap-2 mt-1 text-[10px] text-fg/50">
					<span class="capitalize font-semibold text-fg/70">{modpack.source}</span>
					<span>•</span>
					<span>{formatDownloads(modpack.downloads)} downloads</span>
				</div>
			</div>
		</div>

		<!-- Form Fields -->
		<div class="space-y-4">
			<div>
				<label for="modpack-inst-name" class="block text-[11px] font-bold text-fg/60 uppercase tracking-wider mb-1.5">
					Nome da Instância
				</label>
				<input
					id="modpack-inst-name"
					type="text"
					bind:value={instanceName}
					placeholder="Ex: Better MC, All the Mods..."
					disabled={isInstalling}
					class="w-full bg-bg-elevated border border-fg/10 focus:border-brand-400 rounded-xl px-3.5 py-2.5 text-xs text-fg placeholder-fg/30 focus:outline-none transition-all disabled:opacity-50"
				/>
			</div>

			<div class="bg-bg-elevated border border-emerald-500/20 rounded-xl p-3 flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
						<Cpu class="w-4 h-4" />
					</div>
					<div>
						<span class="text-xs font-bold text-fg block">Memória da Instância</span>
						<span class="text-[10px] text-fg/50 block">Ajuste conforme os requisitos do modpack</span>
					</div>
				</div>
				<span class="text-xs font-mono font-bold text-emerald-400 bg-emerald-500/10 px-2.5 py-1 rounded-full border border-emerald-500/20">{ramMb / 1024} GB</span>
			</div>
            <label class="block text-xs text-fg-muted" for="modpack-ram">RAM: {ramMb / 1024} GB</label>
            <input id="modpack-ram" type="range" min="1024" max="16384" step="512" bind:value={ramMb} disabled={isInstalling} class="w-full accent-brand-500" />
		</div>

		{#if isInstalling}
			<div class="bg-bg-elevated border border-brand-400/20 rounded-2xl p-4 space-y-3">
				<div class="flex items-center gap-3">
					<Loader2 class="w-5 h-5 text-brand-400 animate-spin shrink-0" />
					<div class="min-w-0 flex-1">
						<p class="text-xs font-bold text-fg">Instalando Modpack...</p>
						<p class="text-[11px] text-fg/60 truncate mt-0.5">{progressText || "Por favor, aguarde..."}</p>
					</div>
					{#if progressPercent > 0}
						<span class="text-xs font-mono font-bold text-brand-400 shrink-0">{progressPercent}%</span>
					{/if}
				</div>
				{#if progressPercent > 0}
					<div class="w-full h-1.5 bg-fg/10 rounded-full overflow-hidden">
						<div
							class="h-full bg-gradient-to-r from-brand-400 via-brand-400 to-brand-400 rounded-full transition-all duration-300 ease-out"
							style="width: {progressPercent}%"
						></div>
					</div>
				{:else if progressPercent === -1}
					<div class="w-full h-1.5 bg-fg/10 rounded-full overflow-hidden">
						<div class="h-full bg-gradient-to-r from-brand-400 via-brand-400 to-brand-400 rounded-full animate-pulse" style="width: 100%"></div>
					</div>
				{/if}
			</div>
		{/if}

		<div class="flex items-center justify-end gap-3 pt-2">
			<button
				type="button"
				class={button({ variant: "secondary", size: "sm" })}
                disabled={isInstalling && cancelling}
                onclick={isInstalling ? onCancel : onClose}
			>
                {isInstalling && cancelling ? "Cancelando…" : "Cancelar"}
			</button>
			<button
				type="button"
				class={button({ variant: "primary", size: "sm" })}
				disabled={isInstalling || !instanceName.trim()}
				onclick={onConfirm}
			>
				{#if isInstalling}
					<Loader2 class="w-4 h-4 animate-spin text-brand-foreground" />
					<span>Instalando...</span>
				{:else}
					<PackagePlus class="w-4 h-4" />
					<span>Criar e Baixar Instância</span>
				{/if}
			</button>
		</div>
	</div>
</div>
