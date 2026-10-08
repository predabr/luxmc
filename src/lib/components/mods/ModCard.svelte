<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
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

	import { curatedBanners, matchesCatalogBanner } from "$lib/utils/curatedBanners";

	let bannerAttempt = $state(0);
    let bannerLoaded = $state(false);
    let squareArtwork = $state(false);

	$effect(() => {
		item.slug;
		item.bannerUrl;
		item.iconUrl;
		bannerAttempt = 0;
	});

	const bannerCandidates = $derived.by(() => {
		const candidates: string[] = [];
		if (item.bannerUrl && item.bannerUrl !== item.iconUrl) candidates.push(item.bannerUrl);
		const lowerTitle = item.title.toLowerCase();
		const lowerSlug = item.slug.toLowerCase();
		for (const [key, banner] of Object.entries(curatedBanners)) {
			if (matchesCatalogBanner(lowerTitle, lowerSlug, key) && !candidates.includes(banner)) candidates.push(banner);
		}

        if (item.iconUrl && !candidates.includes(item.iconUrl)) candidates.push(item.iconUrl);

		return candidates;
	});
	const effectiveBanner = $derived(bannerCandidates[bannerAttempt] ?? null);
    const logoArtwork = $derived(effectiveBanner === item.iconUrl || squareArtwork);
    $effect(() => { effectiveBanner; bannerLoaded = false; squareArtwork = false; });
</script>

<div
	role="button"
	tabindex="0"
	onclick={() => onOpenDetails(item)}
	onkeydown={(e) => { if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onOpenDetails(item); } }}
	class="catalog-card group relative h-full flex flex-col overflow-hidden rounded-2xl border border-border bg-bg-elevated cursor-pointer"
>
	<div class="relative h-48 shrink-0 bg-bg-subtle rounded-t-2xl overflow-hidden">
        {#if effectiveBanner}
            <img src={effectiveBanner} alt="" loading="lazy" decoding="async" class="pointer-events-none absolute inset-0 h-full w-full scale-125 object-cover opacity-20 blur-xl" />
        {/if}
		{#if effectiveBanner}
			<img
				src={effectiveBanner}
				alt=""
				loading="lazy"
				decoding="async"
				class="relative h-full w-full rounded-t-2xl {logoArtwork ? 'object-contain p-6 drop-shadow-lg' : 'object-cover'}"
                style:opacity={logoArtwork || bannerLoaded ? 1 : 0}
                onload={event => { bannerLoaded = true; squareArtwork = (event.currentTarget as HTMLImageElement).naturalWidth / Math.max(1,(event.currentTarget as HTMLImageElement).naturalHeight) < 1.3; }}
				onerror={() => { bannerAttempt += 1; }}
			/>
		{:else}
			<div class="absolute inset-0 overflow-hidden rounded-t-2xl bg-bg-elevated">
				<div class="absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-70"></div>
				<div class="card-mesh absolute inset-0 opacity-40"></div>
				<div class="absolute inset-0 flex items-center justify-center p-6">
					{#if item.iconUrl}
						<LazyImage src={item.iconUrl} alt={item.title} class="h-24 w-24 rounded-2xl border border-fg/10 bg-bg-elevated p-2 object-contain shadow-elevated" fallback="/grass_block.png" />
					{:else}
						<Box class="h-12 w-12 text-brand-400" />
					{/if}
				</div>
			</div>
		{/if}
		<div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/20 to-transparent" class:opacity-0={logoArtwork}></div>
		<div class="absolute right-3 top-3 z-10"><SourceBadge source={item.source} /></div>

	</div>
		{#if effectiveBanner && item.iconUrl && !logoArtwork}
			<div data-catalog-logo class="absolute top-40 left-5 z-10 flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl border border-border-strong bg-bg-elevated p-1 shadow-elevated">
				<LazyImage src={item.iconUrl} alt={item.title} class="h-full w-full rounded-xl object-contain p-0.5" fallback="/grass_block.png" />
			</div>
		{/if}
	<div class="flex flex-1 flex-col min-h-0 gap-2.5 p-5 {logoArtwork ? 'pt-5' : 'pt-10'}">
		<h3 title={item.title} class="line-clamp-2 min-h-6 shrink-0 text-base leading-snug font-semibold text-fg group-hover:text-brand-400">
			<span>{item.title}</span>
		</h3>
		<p class="line-clamp-2 min-h-8 shrink-0 text-xs leading-relaxed text-fg-muted">{item.description}</p>
		<div class="flex items-center gap-1.5 text-xs text-fg-subtle"><Users class="h-3 w-3" /><span class="truncate">{item.author || item.slug}</span></div>
		{#if loaders.length}<div class="flex flex-wrap gap-1.5">{#each loaders as loader}<LoaderBadge {loader} />{/each}</div>{/if}
		<div class="mt-auto flex shrink-0 items-center justify-between gap-2 border-t border-border pt-3">
			<div class="space-y-1 text-xs text-fg-muted"><span class="flex items-center gap-1"><Download class="h-3 w-3" />{downloads}</span>{#if latest}<span class="block text-[10px] text-fg-subtle">{uiText("ui.ca34ab5c748c6607")} {latest}</span>{/if}</div>
			<button
				type="button"
				class={button({ variant: isInstalled ? "secondary" : "primary", size: "sm", class: "relative z-10" })}
				onclick={(e) => { e.stopPropagation(); onInstall(item); }}
				disabled={isInstalling || isInstalled}
				aria-label={`${isInstalled ? uiText("mods.installed") : uiText("mods.install")}: ${item.title}`}
			>
				{#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />{uiText("ui.f4200333ccfd750e")}{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />{uiText("mods.installed")}{:else if isModpack}<PackagePlus class="h-3.5 w-3.5" />{uiText("common.create")}{:else}<Download class="h-3.5 w-3.5" />{uiText("mods.install")}{/if}
			</button>
		</div>
	</div>
</div>

<style>
	.catalog-card { contain: layout paint; font-family: var(--font-sans, Inter, sans-serif); }
	.card-mesh {
		background-image:
			linear-gradient(30deg, rgb(var(--fg) / 0.05) 12%, transparent 12.5%, transparent 87%, rgb(var(--fg) / 0.05) 87.5%),
			linear-gradient(150deg, rgb(var(--fg) / 0.05) 12%, transparent 12.5%, transparent 87%, rgb(var(--fg) / 0.05) 87.5%);
		background-size: 36px 62px;
	}
</style>
