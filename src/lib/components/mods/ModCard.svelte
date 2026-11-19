<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { Download, Users, Loader2, Check, Box, ArrowUpRight, Heart, Info } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import SourceBadge from "./SourceBadge.svelte";
    import LoaderBadge from "$lib/components/instances/LoaderBadge.svelte";
	import { button } from "$lib/components/ui/button";
	import type { ModSearchResultItem } from "$lib/api";
    import { catalogReveal } from '$lib/actions/catalogReveal';
	import { untrack } from 'svelte';
	import { catalogGallery, latestMinecraftVersion } from '$lib/utils/catalogGallery';
	import { favorites } from '$lib/stores/favorites.svelte';

	let {
		item,
		isInstalling = false,
		isInstalled = false,
		contentType = "Mod",
		presentation = "square",
		cardSize = "grid",
		onOpenDetails,
		onInstall,
		interactive = false,
		isDragging = false,
		onCatalogDragStart,
		onCatalogDragOver,
		onCatalogDrop,
		onCatalogDragEnd,
		onCatalogReorderKeydown
	}: {
		item: ModSearchResultItem;
		isInstalling?: boolean;
		isInstalled?: boolean;
		contentType?: string;
		presentation?: "square" | "row";
		cardSize?: "grid" | "compact" | "large" | "interactive";
		onOpenDetails: (item: ModSearchResultItem) => void;
		onInstall: (item: ModSearchResultItem) => void;
		interactive?: boolean;
		isDragging?: boolean;
		onCatalogDragStart?: (event: DragEvent) => void;
		onCatalogDragOver?: (event: DragEvent) => void;
		onCatalogDrop?: (event: DragEvent) => void;
		onCatalogDragEnd?: () => void;
		onCatalogReorderKeydown?: (event: KeyboardEvent) => void;
	} = $props();
	const loaders = $derived(item.categories.filter(category => ["fabric", "forge", "neoforge", "quilt"].includes(category.toLowerCase())));
	const downloads = $derived(new Intl.NumberFormat("en", { notation: "compact", maximumFractionDigits: 1 }).format(item.downloads));
	const isModpack = $derived(contentType === "Modpack");
    const latest = $derived(latestMinecraftVersion(item.versions));
	const isFavorite = $derived(favorites.has(item.source, item.sourceId));

	import { catalogBannerCandidates, projectLandscape } from "$lib/utils/curatedBanners";

	let bannerAttempt = $state(0);
    let bannerLoaded = $state(false);
    let bannerAspect = $state(16 / 9);
	let galleryUrls = $state<string[]>([]);
	let galleryRequested = '';
	let nearViewport = $state(false);
	const artworkKey = $derived(`${item.source}:${item.sourceId}:${item.bannerUrl}:${item.iconUrl}`);

	type CoverHandlers = { onLoad: (image: HTMLImageElement) => void; onError: () => void };

	function coverEvents(node: HTMLImageElement, handlers: CoverHandlers) {
		let current = handlers;
		const handleLoad = () => current.onLoad(node);
		const handleError = () => current.onError();
		node.addEventListener('load', handleLoad);
		node.addEventListener('error', handleError);
		return {
			update(next: CoverHandlers) { current = next; },
			destroy() {
				node.removeEventListener('load', handleLoad);
				node.removeEventListener('error', handleError);
			}
		};
	}

	function handleCoverLoad(image: HTMLImageElement) {
		bannerAspect = image.naturalWidth / Math.max(1, image.naturalHeight);
		const isIcon = image.getAttribute('src') === item.iconUrl;
		const needsBetterBanner = isModpack && !isIcon && (image.naturalWidth < 480 || bannerAspect <= 1.2 || bannerAspect >= 4);
		if (needsBetterBanner) {
			requestGallery(true);
			if (bannerAttempt + 1 < bannerCandidates.length) {
				bannerLoaded = false;
				bannerAttempt += 1;
				return;
			}
		}
		bannerLoaded = true;
		if (!isIcon && image.naturalWidth < image.clientWidth) requestGallery(true);
	}

	function handleCoverError() {
		bannerAttempt += 1;
		requestGallery(true);
	}


	$effect(() => {
		artworkKey;
		bannerAttempt = 0;
		galleryUrls = [];
		galleryRequested = '';
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
	const candidateBanner = $derived(bannerCandidates[bannerAttempt] ?? null);
	const effectiveBanner = $derived(candidateBanner);
	const iconCover = $derived(Boolean(effectiveBanner && effectiveBanner === item.iconUrl));
	const logoArtwork = $derived(iconCover || bannerAspect <= 1.2);
	const showCatalogLogo = $derived(Boolean(item.iconUrl && effectiveBanner !== item.iconUrl));
    $effect(() => { effectiveBanner; bannerLoaded = false; bannerAspect = 16 / 9; });
	$effect(() => {
		artworkKey;
		if (nearViewport && isModpack && !projectLandscape(item) && (!candidateBanner || candidateBanner === item.iconUrl)) {
			untrack(() => requestGallery());
		}
	});
	function requestGallery(force = false) {
		if (!nearViewport || !isModpack || !force && projectLandscape(item) || galleryRequested === artworkKey) return;
		const key = artworkKey;
		galleryRequested = key;
		void catalogGallery(item.source, item.sourceId).then(urls => {
			if (artworkKey !== key || !urls.length) return;
			galleryUrls = [...new Set(urls.filter(url => url !== item.iconUrl))];
			bannerAttempt = 0;
		});
	}
	function toggleFavorite(event: Event) {
		event.stopPropagation();
		favorites.toggle(item, contentType);
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
	onkeydown={(e) => {
		if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); onOpenDetails(item); }
		onCatalogReorderKeydown?.(e);
	}}
	draggable={interactive}
	ondragstart={onCatalogDragStart}
	ondragover={onCatalogDragOver}
	ondrop={onCatalogDrop}
	ondragend={onCatalogDragEnd}
	aria-label={item.title}
	class="catalog-card catalog-card--{presentation} catalog-size--{cardSize} group relative h-full flex flex-col overflow-hidden rounded-3xl border border-border bg-bg-elevated cursor-pointer transition-[border-color,box-shadow,translate] duration-300 hover:border-brand-400/40 hover:shadow-elevated {interactive ? 'cursor-grab active:cursor-grabbing' : ''}"
	class:catalog-item-dragging={isDragging}
>
	<div class="catalog-cover relative h-56 shrink-0 bg-bg-subtle overflow-hidden">
		{#if effectiveBanner}
			{#key effectiveBanner}
			{@const artworkUrl = effectiveBanner}
			<img
				use:coverEvents={{ onLoad: handleCoverLoad, onError: handleCoverError }}
				src={artworkUrl}
				alt=""
				loading={nearViewport ? "eager" : "lazy"}
				decoding="async"
                referrerpolicy="no-referrer"
				class="catalog-cover-image relative h-full w-full object-cover transition-opacity duration-150"
				class:catalog-cover-icon={iconCover}
				class:catalog-cover-contained={bannerAspect <= 1.2}
                style:object-position="50% 50%"
                style:opacity={bannerLoaded ? 1 : 0}
			/>
			{/key}
		{:else}
			<div class="catalog-cover-fallback absolute inset-0 flex items-center justify-center overflow-hidden" aria-hidden="true">
				{#if item.iconUrl}
					<LazyImage src={item.iconUrl} alt="" loading="eager" class="catalog-cover-fallback-icon absolute inset-0 object-cover" fallback="/grass_block.png" />
				{:else}
					<div class="catalog-cover-mark flex h-20 w-20 items-center justify-center rounded-3xl border border-brand-400/15">
						<Box class="h-9 w-9 text-brand-400/55" />
					</div>
				{/if}
			</div>
		{/if}
		{#if !bannerLoaded && effectiveBanner}<div aria-hidden="true" class="catalog-cover-loading pointer-events-none absolute inset-0"></div>{/if}
		<div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-bg-overlay/40 via-transparent to-transparent" ></div>
		<div class="absolute right-3 top-3 z-10 flex flex-col items-end gap-1.5">
			<SourceBadge source={item.source} />
			<button
				type="button"
				class="favorite-btn"
				aria-pressed={isFavorite}
				aria-label={isFavorite ? `Remover dos favoritos: ${item.title}` : `Favoritar: ${item.title}`}
				onclick={toggleFavorite}
			>
				<Heart class="h-3.5 w-3.5 {isFavorite ? 'filled-heart' : ''}" />
			</button>
		</div>
        {#if !isModpack && loaders.length}<div class="catalog-loaders absolute left-3 top-3 z-10 flex gap-1">{#each loaders.slice(0,2) as loader}<LoaderBadge {loader} />{/each}</div>{/if}
        {#if latest}<span class="catalog-version absolute bottom-3 right-3">MC {latest}</span>{/if}

	</div>
		{#if showCatalogLogo}
			<div data-catalog-logo class="catalog-logo absolute left-5 z-10 flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl border border-border-strong bg-bg-elevated p-1 shadow-elevated">
				<LazyImage src={item.iconUrl ?? ""} alt={item.title} loading="eager" class="h-full w-full rounded-xl object-contain p-0.5" fallback="/grass_block.png" />
			</div>
		{/if}
	<div class="flex flex-1 flex-col min-h-0 catalog-card-body gap-2 p-5" class:pt-9={Boolean(item.iconUrl)}>
		<h3 title={item.title} class="catalog-card-title line-clamp-2 min-h-11 shrink-0 text-lg leading-tight font-semibold text-fg group-hover:text-brand-400">
			<span>{item.title}</span>
		</h3>
		<p class="line-clamp-2 min-h-8 shrink-0 text-xs leading-relaxed text-fg-muted">{item.description}</p>
		<div class="flex items-center gap-3 text-[11px] text-fg-muted">
			<span class="flex min-w-0 items-center gap-1.5"><Users class="h-3 w-3 shrink-0" /><span class="truncate">{item.author || item.slug}</span></span>
			<span class="flex shrink-0 items-center gap-1"><Download class="h-3 w-3" />{downloads}</span>
		</div>

		<div class="mt-auto flex shrink-0 items-center justify-between gap-2 border-t border-border pt-3">
			<button
				type="button"
				class={button({ variant: isInstalled ? "secondary" : "ghostBrand", size: "sm", class: "catalog-install relative z-10" })}
				onclick={(e) => { e.stopPropagation(); installItem(); }}
				disabled={isInstalling || isInstalled}
				aria-label={`${isInstalled ? uiText("mods.installed") : uiText("mods.install")}: ${item.title}`}
			>
				{#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />{uiText("ui.f4200333ccfd750e")}{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />{uiText("mods.installed")}{:else if isModpack}{uiText("catalogDesign.installPack")}<ArrowUpRight class="h-4 w-4" />{:else}<Download class="h-3.5 w-3.5" />{uiText("mods.install")}{/if}
			</button>
			<button
				type="button"
				class="details-btn"
				tabindex="-1"
				aria-hidden="true"
				onclick={(e) => { e.stopPropagation(); onOpenDetails(item); }}
			>
				<Info class="h-3.5 w-3.5" />
				<span>Detalhes</span>
			</button>
		</div>
	</div>
</div>

<style>
	.catalog-card { contain: layout; font-family: var(--font-sans, Inter, sans-serif); border-radius: 24px; }
    .catalog-cover { border-bottom: 1px solid rgb(var(--border)); }
    .catalog-cover-image { display: block; object-fit:cover; }
    .catalog-cover-image.catalog-cover-icon { position: absolute; inset: 0; width: 100%; height: 100%; padding: 0; object-fit: cover; }
    .catalog-cover-fallback-icon :global(img) { width: 100%; height: 100%; border-radius: 0; object-fit: cover; opacity: .8; }
    .catalog-cover-loading { background: linear-gradient(110deg, transparent 20%, rgb(var(--fg) / .04) 42%, transparent 64%); }
    .catalog-loaders { padding: 3px; border-radius: 12px; background: rgb(var(--bg-elevated) / .94); }
    .catalog-logo { top: 188px; }
    .catalog-cover { background: rgb(var(--bg-subtle)); }
	.catalog-card--square { border-radius:24px; }
	.catalog-cover-image.catalog-cover-portrait { object-fit:contain; }
	.catalog-size--compact .catalog-cover { height:156px; }
	.catalog-size--compact .catalog-card-body { gap:6px; padding:16px; }
	.catalog-size--compact .catalog-card-body h3 { font-size:15px; }
	.catalog-size--grid .catalog-cover { height:224px; }
	.catalog-size--large .catalog-cover { height:260px; }
	.catalog-size--large .catalog-card-body { gap:12px; padding:24px; }
	.catalog-size--large .catalog-card-body h3 { font-size:21px; }
	.catalog-size--interactive .catalog-cover { height:232px; }
	.catalog-size--interactive .catalog-card-body { padding:20px; }
	.catalog-card--row { display:grid; grid-template-columns:minmax(168px,30%) minmax(0,1fr); height:auto; min-height:208px; border-radius:20px; }
	.catalog-card--row .catalog-cover { width:auto; height:auto; min-height:0; min-width:0; max-height:none; flex:none; align-self:stretch; border-right:1px solid rgb(var(--border)); border-bottom:1px solid rgb(var(--border)); border-radius:19px 0 0 19px; }
	.catalog-card--row .catalog-cover-image { position:absolute; inset:0; width:100%; height:100%; object-fit:cover; background:transparent; }
	.catalog-card--row .catalog-cover-fallback-icon :global(img) { object-fit:cover; }
	.catalog-card--row .catalog-cover-fallback { border-radius:inherit; }
	.catalog-card--row .catalog-logo { top:auto; bottom:12px; left:12px; width:44px; height:44px; }
	.catalog-card--row .catalog-card-body { min-width:0; justify-content:center; padding:18px 22px; }
	.catalog-card--row .catalog-card-title { display:block; min-height:0; overflow:visible; overflow-wrap:anywhere; -webkit-line-clamp:unset; line-clamp:unset; }
	.catalog-card--row .catalog-card-body > p { min-height:0; }
	.catalog-card--row .catalog-card-body > div:nth-of-type(1) { flex-wrap:wrap; }
	.catalog-card--row .catalog-card-body > div:last-child { margin-top:8px; }
	.catalog-card--row.catalog-size--compact { min-height:176px; grid-template-columns:minmax(144px,24%) minmax(0,1fr); }
	.catalog-card--row.catalog-size--compact .catalog-card-body { gap:6px; padding:14px 18px; }
	.catalog-card--row.catalog-size--compact .catalog-card-title { font-size:15px; line-height:1.35; }
	.catalog-card--row.catalog-size--compact .catalog-card-body > p { display:none; }
	.catalog-card--row.catalog-size--large { min-height:292px; grid-template-columns:minmax(220px,34%) minmax(0,1fr); }
	.catalog-card--row.catalog-size--large .catalog-card-body { gap:12px; padding:24px 28px; }
	.catalog-card--row.catalog-size--large .catalog-card-title { font-size:22px; line-height:1.25; }
	.catalog-card--row.catalog-size--large .catalog-card-body > p { -webkit-line-clamp:3; line-clamp:3; }
	.catalog-card--row.catalog-size--interactive { min-height:244px; }
	@media (max-width:560px) {
		.catalog-card--row { grid-template-columns:minmax(104px,32%) minmax(0,1fr); min-height:192px; }
		.catalog-card--row.catalog-size--compact { grid-template-columns:minmax(104px,32%) minmax(0,1fr); }
		.catalog-card--row.catalog-size--large { grid-template-columns:minmax(120px,34%) minmax(0,1fr); min-height:264px; }
		.catalog-card--row .catalog-card-body { padding:14px; }
		.catalog-card--row .catalog-logo { bottom:8px; left:8px; width:36px; height:36px; }
		.catalog-card--row .catalog-card-title { font-size:16px; line-height:1.3; }
		.catalog-card--row .catalog-card-body > div:last-child { flex-wrap:wrap; }
		.catalog-card--row.catalog-size--compact .catalog-card-body > p { display:-webkit-box; }
	}
	.catalog-cover-fallback { background: radial-gradient(ellipse at 50% 38%, rgb(var(--brand-500) / .16), transparent 58%), linear-gradient(145deg, rgb(var(--bg-subtle)), rgb(var(--bg-elevated))); }
	.catalog-cover-fallback::before, .catalog-cover-fallback::after { content: ''; position: absolute; width: 70%; aspect-ratio: 1; border: 1px solid rgb(var(--brand-400) / .08); border-radius: 35%; transform: rotate(30deg); pointer-events: none; }
	.catalog-cover-fallback::after { width: 48%; border-color: rgb(var(--brand-400) / .06); transform: rotate(60deg); }
	.catalog-cover-mark { position: relative; z-index: 1; background: rgb(var(--bg-elevated) / .42); box-shadow: 0 12px 32px rgb(var(--shadow-color) / .14); }
    .catalog-version { padding: 5px 8px; border: 1px solid rgb(var(--surface-shine) / .15); border-radius: 10px; background: rgb(var(--shadow-color) / .75); color: rgb(var(--surface-shine)); font-size: 9px; font-weight: 600; backdrop-filter: blur(8px); }
    .catalog-card-body { background: linear-gradient(165deg, rgb(var(--brand-500) / .03), transparent 55%); }
    .catalog-card :global(.catalog-install) {
        height: 38px; padding-inline: 15px; border-radius: 12px;
        border: 1px solid rgb(var(--brand-300) / .55);
        color: #fff; font-weight: 750; letter-spacing: .01em;
        background-image: linear-gradient(180deg, rgb(var(--brand-400) / .98), rgb(var(--brand-600) / .98));
        box-shadow: inset 0 1px 0 rgb(255 255 255 / .3), 0 10px 22px -10px rgb(var(--brand-500) / .85);
        transition: box-shadow 180ms ease, filter 180ms ease, translate 180ms ease, background-image 180ms ease;
    }
    .catalog-card :global(.catalog-install:hover:not(:disabled)) {
        filter: brightness(1.06) saturate(1.05);
        box-shadow: inset 0 1px 0 rgb(255 255 255 / .38), 0 16px 30px -12px rgb(var(--brand-500) / .95);
        translate: 0 -1px;
    }
    .catalog-card :global(.catalog-install:active:not(:disabled)) { translate: 0 0; filter: brightness(.98); box-shadow: inset 0 2px 6px rgb(2 6 23 / .35); }
    .catalog-card :global(.catalog-install:disabled) { color: rgb(var(--fg-muted)); background-image: none; background-color: rgb(var(--fg) / .06); border-color: rgb(var(--fg) / .12); box-shadow: none; font-weight: 650; }
    .catalog-card :global(.catalog-install:focus-visible) { outline: 2px solid rgb(var(--brand-300)); outline-offset: 2px; }
    :global(html.has-custom-wallpaper) .catalog-card { background-color: rgb(var(--bg-elevated) / var(--interface-opacity, .34)) !important; backdrop-filter: blur(12px); }
    :global(html.has-custom-wallpaper) .catalog-card-body { background-color: transparent !important; }
    :global(html.has-custom-wallpaper) .catalog-card :global(.catalog-install:disabled) { background-color: rgb(var(--fg) / .08) !important; background-image: none !important; }
    .catalog-card:hover { translate: 0 -4px; box-shadow: 0 18px 44px -16px rgb(var(--brand-500) / .3), var(--shadow-elevated); }
    .catalog-card:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 3px; }
	.favorite-btn {
		display: grid; place-items: center; width: 28px; height: 28px;
		border: 1px solid rgb(var(--surface-shine) / .18); border-radius: 12px;
		background: rgb(var(--shadow-color) / .72); color: rgb(var(--surface-shine));
		backdrop-filter: blur(8px);
		transition: color 150ms ease, border-color 150ms ease, transform 150ms ease;
	}
	.favorite-btn:hover { color: rgb(var(--danger)); border-color: rgb(var(--danger) / .45); transform: scale(1.06); }
	.favorite-btn:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
	.favorite-btn :global(.filled-heart) { fill: rgb(var(--danger)); color: rgb(var(--danger)); }
	.details-btn {
		display: inline-flex; align-items: center; gap: 7px;
		padding: 7px 11px; border-radius: 12px; border: 1px solid rgb(var(--border));
		background: rgb(var(--fg) / .03); color: rgb(var(--fg-muted));
		font-size: 11.5px; font-weight: 600;
		opacity: 0; translate: 4px 0;
		transition: opacity 200ms ease, translate 200ms ease, color 150ms ease, border-color 150ms ease;
	}
	.catalog-card:hover .details-btn, .catalog-card:focus-within .details-btn { opacity: 1; translate: 0 0; }
	.details-btn:hover { color: rgb(var(--fg)); border-color: rgb(var(--brand-400) / .4); }
    :global(.no-ui-motion) .catalog-card, :global(.no-anim) .catalog-card { translate: none; transition: none; }
	:global(.no-ui-motion) .details-btn, :global(.no-anim) .details-btn { opacity: 1; translate: none; transition: none; }
    @media (prefers-reduced-motion: reduce) {
		:global(html:not(.force-ui-motion)) .catalog-card, :global(html:not(.force-ui-motion)) .catalog-card:hover { translate: none; transition: none; }
		:global(html:not(.force-ui-motion)) .details-btn { opacity: 1; translate: none; transition: none; }
	}
</style>
