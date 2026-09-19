<script lang="ts">
	import { Download, Users, Loader2, Check, PackagePlus, Box } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import SourceBadge from "./SourceBadge.svelte";
    import LoaderBadge from "$lib/components/instances/LoaderBadge.svelte";
	import { button } from "$lib/components/ui/button";
	import type { ModSearchResultItem } from "$lib/api";

	let { item, isInstalling = false, isInstalled = false, contentType = "Mod", onOpenDetails, onInstall }: {
		item: ModSearchResultItem;
		isInstalling?: boolean;
		isInstalled?: boolean;
		contentType?: string;
		onOpenDetails: (item: ModSearchResultItem) => void;
		onInstall: (item: ModSearchResultItem) => void;
	} = $props();
	const loaders = $derived(item.categories.filter(category => ["fabric", "forge", "neoforge", "quilt"].includes(category.toLowerCase())));
	const downloads = $derived(new Intl.NumberFormat("en", { notation: "compact", maximumFractionDigits: 1 }).format(item.downloads));
	const isModpack = $derived(contentType === "Modpack");
    const latest = $derived([...item.versions].sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))[0]);
	const mesh = $derived.by(() => {
		let hash = 0;
		for (const character of item.title) hash = (Math.imul(hash, 31) + character.charCodeAt(0)) | 0;
		return ["from-brand-500/30 via-info/10", "from-purple-500/30 via-brand-500/10", "from-success/30 via-info/10", "from-warning/30 via-brand-500/10"][Math.abs(hash) % 4];
	});
</script>

<article class="group relative flex flex-col overflow-hidden rounded-2xl border border-border bg-bg-elevated shadow-soft transition-all duration-150 hover:-translate-y-0.5 hover:border-brand-500/40 hover:shadow-elevated">
	<div class="relative h-40 bg-bg-subtle rounded-t-2xl">
		{#if item.bannerUrl}
			<LazyImage src={item.bannerUrl} alt="" class="h-full w-full rounded-t-2xl object-cover transition-transform duration-300 group-hover:scale-105" />
		{:else}
			<div class="absolute inset-0 overflow-hidden rounded-t-2xl">
				<div class="absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-60"></div>
			</div>
		{/if}
		<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/10 to-transparent"></div>
		<div class="absolute right-3 top-3"><SourceBadge source={item.source} /></div>
		<div class="absolute -bottom-4 left-5 z-10 flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl border border-border-strong bg-bg-elevated p-1 shadow-elevated transition-transform duration-150 group-hover:-translate-y-0.5">
			{#if item.iconUrl}<LazyImage src={item.iconUrl} alt="" class="h-full w-full rounded-xl object-cover" />{:else}<Box class="h-7 w-7 text-brand-400" />{/if}
		</div>
	</div>
	<div class="flex flex-1 flex-col gap-3 p-5 pt-7">
		<h3 class="truncate text-base font-bold text-fg group-hover:text-brand-400">
			<button type="button" class="text-left after:absolute after:inset-0 focus-visible:outline-none focus-visible:after:ring-2 focus-visible:after:ring-inset focus-visible:after:ring-brand-500" onclick={() => onOpenDetails(item)}>{item.title}</button>
		</h3>
		<p class="line-clamp-2 min-h-8 text-xs leading-relaxed text-fg-muted">{item.description}</p>
		<div class="flex items-center gap-1.5 text-xs text-fg-subtle"><Users class="h-3 w-3" /><span class="truncate">{item.author || item.slug}</span></div>
		{#if loaders.length}<div class="flex flex-wrap gap-1.5">{#each loaders as loader}<LoaderBadge {loader} />{/each}</div>{/if}
		<div class="mt-auto flex items-center justify-between gap-2 border-t border-border pt-3">
			<div class="space-y-1 text-xs text-fg-muted"><span class="flex items-center gap-1"><Download class="h-3 w-3" />{downloads}</span>{#if latest}<span class="block text-[10px] text-fg-subtle">MC {latest}</span>{/if}</div>
			<button type="button" class={button({ variant: isInstalled ? "secondary" : "primary", size: "sm", class: "relative z-10" })} onclick={() => onInstall(item)} disabled={isInstalling || isInstalled} aria-label={`${isInstalled ? "Instalado" : "Instalar"}: ${item.title}`}>
				{#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />Instalando{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />Instalado{:else if isModpack}<PackagePlus class="h-3.5 w-3.5" />Criar{:else}<Download class="h-3.5 w-3.5" />Instalar{/if}
			</button>
		</div>
	</div>
</article>
