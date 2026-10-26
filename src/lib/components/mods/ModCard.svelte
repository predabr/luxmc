<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { Download, Users, Loader2, Check, Box, ArrowUpRight } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import SourceBadge from "./SourceBadge.svelte";
    import LoaderBadge from "$lib/components/instances/LoaderBadge.svelte";
	import { button } from "$lib/components/ui/button";
	import type { ModSearchResultItem } from "$lib/api";
    import { catalogReveal } from '$lib/actions/catalogReveal';
	import { untrack } from 'svelte';
	import { catalogGallery, firstLandscape, latestMinecraftVersion } from '$lib/utils/catalogGallery';

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
    const latest = $derived(latestMinecraftVersion(item.versions));
	const mesh = $derived.by(() => {
		let hash = 0;
		for (const character of item.title) hash = (Math.imul(hash, 31) + character.charCodeAt(0)) | 0;
		return ["from-brand-500/30 via-info/10", "from-purple-500/30 via-brand-500/10", "from-success/30 via-info/10", "from-warning/30 via-brand-500/10"][Math.abs(hash) % 4];
	});

	import { catalogBannerCandidates, projectLandscape } from "$lib/utils/curatedBanners";

	let bannerAttempt = $state(0);
    let bannerLoaded = $state(false);
    let bannerAspect = $state(16 / 9);
	let galleryUrls = $state<string[]>([]);
	let galleryRequested = '';
	let nearViewport = $state(false);
	const artworkKey = $derived(`${item.source}:${item.sourceId}:${item.bannerUrl}:${item.iconUrl}`);


	$effect(() => {
		artworkKey;
		bannerAttempt = 0;
		galleryUrls = [];
		galleryRequested = '';
	});
	$effect(() => {
		artworkKey;
		if (nearViewport) untrack(() => { if (!bannerCandidates.includes(item.bannerUrl || '') || item.bannerUrl === item.iconUrl || bannerAspect <= 1.2) requestGallery(); });
	});
	function observeArtwork(node: HTMLElement) {
		if (typeof IntersectionObserver === 'undefined') { nearViewport = true; return; }
		const observer = new IntersectionObserver(entries => {
			if (!entries.some(entry => entry.isIntersecting)) return;
			nearViewport = true;
			observer.disconnect();
		}, { rootMargin: '200px' });
		observer.observe(node);
		return { destroy() { observer.disconnect(); } };
	}

	const bannerCandidates = $derived(catalogBannerCandidates({ ...item, galleryUrls }));
	const effectiveBanner = $derived(bannerCandidates[bannerAttempt] ?? null);
    const logoArtwork = $derived(effectiveBanner === item.iconUrl || bannerAspect <= 1.2);
    $effect(() => { effectiveBanner; bannerLoaded = false; bannerAspect = 16 / 9; });
	function requestGallery() {
		if (!nearViewport || !isModpack || projectLandscape(item) || galleryRequested === artworkKey) return;
		const key = artworkKey;
		galleryRequested = key;
		void catalogGallery(item.source, item.sourceId).then(async urls => {
			const landscape = await firstLandscape(urls.filter(url => url !== item.iconUrl));
			if (artworkKey !== key || !urls.length) return;
			galleryUrls = [...new Set([...(landscape ? [landscape] : []), ...urls.filter(url => url !== item.iconUrl)])];
			bannerAttempt = 0;
		});
	}
	function installItem() {
		onInstall({ ...item, bannerUrl: bannerLoaded && !logoArtwork && effectiveBanner ? effectiveBanner : item.bannerUrl });
	}
</script>

<div
    use:observeArtwork
    use:catalogReveal={`${item.source}:${item.sourceId}`}
	role="button"
	tabindex="0"
	onclick={() => onOpenDetails(item)}
	onkeydown={(e) => { if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onOpenDetails(item); } }}
	aria-label={item.title}
	class="catalog-card group relative h-full flex flex-col overflow-hidden rounded-3xl border border-border bg-bg-elevated cursor-pointer transition-[border-color,box-shadow,translate] duration-300 hover:border-brand-400/40 hover:shadow-elevated"
>
	<div class="catalog-cover relative h-56 shrink-0 bg-bg-subtle rounded-t-2xl overflow-hidden">
        <div class="pointer-events-none absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-40"></div>
		{#if effectiveBanner}
			{#key effectiveBanner}
			{@const artworkUrl = effectiveBanner}
			<img
				src={artworkUrl}
				alt=""
				loading="lazy"
				decoding="async"
                referrerpolicy="no-referrer"
				class="catalog-cover-image relative h-full w-full rounded-t-2xl object-cover transition-opacity duration-150"
                style:object-position="50% 50%"
                style:opacity={bannerLoaded ? logoArtwork ? .4 : 1 : 0}
                onload={(event) => {
					if (artworkUrl !== effectiveBanner) return;
					const image = event.currentTarget as HTMLImageElement;
					bannerAspect = image.naturalWidth / Math.max(1, image.naturalHeight);
					bannerLoaded = true;
					if (bannerAspect <= 1.2) {
						requestGallery();
					}
                    if (image.naturalWidth < image.clientWidth) requestGallery();
				}}
				onerror={() => { if (artworkUrl === effectiveBanner) { bannerAttempt += 1; requestGallery(); } }}
			/>
			{/key}
		{:else}
			<div class="absolute inset-0 overflow-hidden rounded-t-2xl bg-bg-elevated">
				<div class="absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-70"></div>
				<div class="card-mesh absolute inset-0 opacity-40"></div>
				<div class="absolute inset-0 flex items-center justify-center p-6">
					{#if item.iconUrl}
						<LazyImage src={item.iconUrl} alt={item.title} loading="eager" class="h-24 w-24 rounded-2xl border border-fg/10 bg-bg-elevated p-2 object-contain shadow-elevated" fallback="/grass_block.png" />
					{:else}
						<Box class="h-12 w-12 text-brand-400" />
					{/if}
				</div>
			</div>
		{/if}
		{#if !bannerLoaded && effectiveBanner}<div aria-hidden="true" class="pointer-events-none absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-50"></div>{/if}
		<div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-bg-overlay/50 via-transparent to-transparent" ></div>
		<div class="absolute right-3 top-3 z-10"><SourceBadge source={item.source} /></div>
        {#if loaders.length}<div class="catalog-loaders absolute left-3 top-3 z-10 flex gap-1">{#each loaders.slice(0,2) as loader}<LoaderBadge {loader} />{/each}</div>{/if}
        {#if latest}<span class="catalog-version absolute bottom-3 right-3">MC {latest}</span>{/if}

	</div>
		{#if item.iconUrl}
			<div data-catalog-logo class="catalog-logo absolute left-5 z-10 flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl border border-border-strong bg-bg-elevated p-1 shadow-elevated">
				<LazyImage src={item.iconUrl} alt={item.title} loading="eager" class="h-full w-full rounded-xl object-contain p-0.5" fallback="/grass_block.png" />
			</div>
		{/if}
	<div class="flex flex-1 flex-col min-h-0 catalog-card-body gap-2 p-5" class:pt-9={Boolean(item.iconUrl)}>
		<h3 title={item.title} class="line-clamp-2 min-h-11 shrink-0 text-lg leading-tight font-semibold text-fg group-hover:text-brand-400">
			<span>{item.title}</span>
		</h3>
		<p class="line-clamp-2 min-h-8 shrink-0 text-xs leading-relaxed text-fg-muted">{item.description}</p>
		<div class="flex items-center gap-1.5 text-[11px] text-fg-muted"><Users class="h-3 w-3" /><span class="truncate">{item.author || item.slug}</span></div>

		<div class="mt-auto flex shrink-0 items-center justify-between gap-2 border-t border-border pt-3">
			<div class="space-y-1 text-xs text-fg-muted"><span class="flex items-center gap-1"><Download class="h-3 w-3" />{downloads}</span><span class="block text-[10px] text-fg-subtle">{uiText("catalogDesign.downloads")}</span></div>
			<button
				type="button"
				class={button({ variant: isInstalled ? "secondary" : "ghostBrand", size: "sm", class: "catalog-install relative z-10" })}
				onclick={(e) => { e.stopPropagation(); installItem(); }}
				disabled={isInstalling || isInstalled}
				aria-label={`${isInstalled ? uiText("mods.installed") : uiText("mods.install")}: ${item.title}`}
			>
				{#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />{uiText("ui.f4200333ccfd750e")}{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />{uiText("mods.installed")}{:else if isModpack}{uiText("catalogDesign.installPack")}<ArrowUpRight class="h-4 w-4" />{:else}<Download class="h-3.5 w-3.5" />{uiText("mods.install")}{/if}
			</button>
		</div>
	</div>
</div>

<style>
	.catalog-card { contain: layout paint; font-family: var(--font-sans, Inter, sans-serif); border-radius: 22px; }
    .catalog-cover { border-bottom: 1px solid rgb(var(--border)); }
    .catalog-loaders { padding: 3px; border-radius: 10px; background: rgb(var(--bg-elevated) / .94); }
    .catalog-logo { top: 188px; }
    .catalog-cover { background:rgb(var(--bg-subtle)/.97); }
    .catalog-version { padding: 5px 8px; border: 1px solid rgb(var(--surface-shine) / .15); border-radius: 8px; background: rgb(var(--shadow-color) / .75); color: rgb(var(--surface-shine)); font-size: 9px; font-weight: 600; backdrop-filter: blur(8px); }
    .catalog-card-body { background: linear-gradient(150deg, rgb(var(--brand-500) / .035), transparent 70%); }
    .catalog-card :global(.catalog-install) { border: 1px solid rgb(var(--brand-400) / .2); border-radius: 12px; padding-inline: 12px; color: rgb(var(--brand-action)); background: rgb(var(--brand-500) / .07); }
    .catalog-card:hover { translate: 0 -4px; }
    .catalog-card:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 3px; }
    :global(.no-ui-motion) .catalog-card, :global(.no-anim) .catalog-card { translate: none; transition: none; }
    @media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .catalog-card, :global(html:not(.force-ui-motion)) .catalog-card:hover { translate: none; transition: none; } }
	.card-mesh {
		background-image:
			linear-gradient(30deg, rgb(var(--fg) / 0.05) 12%, transparent 12.5%, transparent 87%, rgb(var(--fg) / 0.05) 87.5%),
			linear-gradient(150deg, rgb(var(--fg) / 0.05) 12%, transparent 12.5%, transparent 87%, rgb(var(--fg) / 0.05) 87.5%);
		background-size: 36px 62px;
	}
</style>
