<script lang="ts">
	import { Download, Users, Loader2, Check, PackagePlus } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import { button } from "$lib/components/ui/button";
	import SourceBadge from "./SourceBadge.svelte";
	import type { ModSearchResultItem } from "$lib/api";

	let {
		item,
		isInstalling = false,
		isInstalled = false,
		contentType = "Mod",
		onOpenDetails,
		onInstall
	}: {
		item: ModSearchResultItem;
		isInstalling?: boolean;
		isInstalled?: boolean;
		contentType?: string;
		onOpenDetails: (item: ModSearchResultItem) => void;
		onInstall: (item: ModSearchResultItem) => void;
	} = $props();

	function formatDownloads(n: number): string {
		if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
		if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
		return n.toString();
	}

	const isModpack = $derived(contentType === "Modpack");
</script>

<article class="surface-glass group relative flex flex-wrap items-center gap-4 p-4 transition-colors duration-200 hover:border-brand-500/30">
    <div class="flex min-w-48 flex-1 items-center gap-4">
        <div class="flex h-16 w-16 shrink-0 items-center justify-center overflow-hidden rounded-2xl border border-fg/10 bg-brand-500/10 shadow-elevated">
            {#if item.iconUrl}<LazyImage src={item.iconUrl} alt="" class="h-full w-full object-cover" />{:else}<PackagePlus class="h-7 w-7 text-brand-400" />{/if}
        </div>
        <div class="min-w-0 flex-1">
            <h3 class="truncate text-sm font-semibold text-fg group-hover:text-brand-400"><button type="button" class="text-left after:absolute after:inset-0 after:rounded-2xl focus-visible:after:ring-2 focus-visible:after:ring-brand-500" onclick={() => onOpenDetails(item)}>{item.title}</button></h3>
            <p class="mt-1 line-clamp-1 text-xs text-fg-muted">{item.description}</p>
            <div class="mt-2 flex items-center gap-4 text-[10px] text-fg-subtle"><span class="flex items-center gap-1"><Download class="h-3 w-3" />{formatDownloads(item.downloads)}</span><span class="flex items-center gap-1"><Users class="h-3 w-3" />{item.author || item.slug}</span></div>
        </div>
    </div>
    <div class="relative z-10 flex items-center gap-3">
        <SourceBadge source={item.source} />
        <button type="button" class={button({variant: isInstalled ? 'secondary' : 'primary', size:'sm'})} onclick={() => onInstall(item)} disabled={isInstalling || isInstalled} aria-busy={isInstalling}>
            {#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />Instalando{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />Instalado{:else if isModpack}<PackagePlus class="h-3.5 w-3.5" />Criar instância{:else}<Download class="h-3.5 w-3.5" />Instalar{/if}
        </button>
    </div>
</article>
