<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { Download, Users, Loader2, Check, PackagePlus, Heart, Info } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import { button } from "$lib/components/ui/button";
	import SourceBadge from "./SourceBadge.svelte";
	import type { ModSearchResultItem } from "$lib/api";
	import { favorites } from '$lib/stores/favorites.svelte';
	import { latestMinecraftVersion } from '$lib/utils/catalogGallery';

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
	const isFavorite = $derived(favorites.has(item.source, item.sourceId));
	const latest = $derived(latestMinecraftVersion(item.versions));
</script>

<div
    role="button"
    tabindex="0"
    onclick={() => onOpenDetails(item)}
    onkeydown={(e) => { if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onOpenDetails(item); } }}
    aria-label={item.title}
    class="catalog-card catalog-list-card surface-glass group relative flex flex-wrap items-center rounded-2xl gap-4 p-4 transition-[border-color,box-shadow,translate] duration-200 hover:border-brand-500/30 hover:shadow-elevated cursor-pointer"
>
    <div class="flex min-w-48 flex-1 items-center gap-4">
        <div class="flex h-16 w-16 shrink-0 items-center justify-center overflow-hidden rounded-2xl border border-fg/10 bg-brand-500/10 shadow-elevated">
            {#if item.iconUrl}<LazyImage src={item.iconUrl} alt="" class="h-full w-full object-contain p-0.5" fallback="/grass_block.png" />{:else}<PackagePlus class="h-7 w-7 text-brand-400" />{/if}
        </div>
        <div class="min-w-0 flex-1">
            <h3 class="truncate text-sm font-semibold text-fg group-hover:text-brand-400">
                <span>{item.title}</span>
            </h3>
            <p class="mt-1 line-clamp-1 text-xs text-fg-muted">{item.description}</p>
            <div class="mt-2 flex items-center gap-4 text-[10px] text-fg-subtle">
				<span class="flex items-center gap-1"><Download class="h-3 w-3" />{formatDownloads(item.downloads)}</span>
				<span class="flex min-w-0 items-center gap-1"><Users class="h-3 w-3" /><span class="truncate">{item.author || item.slug}</span></span>
				{#if latest}<span class="font-mono">MC {latest}</span>{/if}
			</div>
        </div>
    </div>
    <div class="relative z-10 flex items-center gap-3">
		<button
			type="button"
			class="favorite-btn"
			aria-pressed={isFavorite}
			aria-label={isFavorite ? `Remover dos favoritos: ${item.title}` : `Favoritar: ${item.title}`}
			onclick={(e) => { e.stopPropagation(); favorites.toggle(item, contentType); }}
		>
			<Heart class="h-4 w-4 {isFavorite ? 'filled-heart' : ''}" />
		</button>
		<button
			type="button"
			class="details-btn"
			onclick={(e) => { e.stopPropagation(); onOpenDetails(item); }}
			aria-label={`Detalhes: ${item.title}`}
		>
			<Info class="h-4 w-4" />
		</button>
        <SourceBadge source={item.source} />
        <button
            type="button"
            class={button({variant: isInstalled ? 'secondary' : 'primary', size:'sm'})}
            onclick={(e) => { e.stopPropagation(); onInstall(item); }}
            disabled={isInstalling || isInstalled}
            aria-busy={isInstalling}
        >
            {#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />{uiText("ui.f4200333ccfd750e")}{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />{uiText("mods.installed")}{:else if isModpack}<PackagePlus class="h-3.5 w-3.5" />{uiText("mods.createInstance")}{:else}<Download class="h-3.5 w-3.5" />{uiText("mods.install")}{/if}
        </button>
    </div>
</div>

<style>
	.favorite-btn {
		display: grid; place-items: center; width: 34px; height: 34px; flex: none;
		border: 1px solid rgb(var(--border)); border-radius: 10px;
		background: rgb(var(--fg) / .03); color: rgb(var(--fg-muted));
		transition: color 150ms ease, border-color 150ms ease, transform 150ms ease;
	}
	.favorite-btn:hover { color: rgb(var(--danger)); border-color: rgb(var(--danger) / .45); transform: scale(1.06); }
	.favorite-btn:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
	.favorite-btn :global(.filled-heart) { fill: rgb(var(--danger)); color: rgb(var(--danger)); }
	.details-btn {
		display: grid; place-items: center; width: 34px; height: 34px; flex: none;
		border: 1px solid rgb(var(--border)); border-radius: 10px;
		background: rgb(var(--fg) / .03); color: rgb(var(--fg-muted));
		transition: color 150ms ease, border-color 150ms ease;
	}
	.details-btn:hover { color: rgb(var(--fg)); border-color: rgb(var(--brand-400) / .4); }
	.details-btn:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
	@media (prefers-reduced-motion: reduce) { .favorite-btn, .details-btn { transition: none; } }
</style>
