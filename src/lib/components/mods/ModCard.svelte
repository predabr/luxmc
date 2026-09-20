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

	const curatedBanners: Record<string, string> = {
		rlcraft: "https://media.forgecdn.net/attachments/267/928/rlcraft-banner.png",
		"skyfactory 4": "https://media.forgecdn.net/attachments/258/182/skyfactory4_banner.png",
		"skyfactory-4": "https://media.forgecdn.net/attachments/258/182/skyfactory4_banner.png",
		skyfactory: "https://media.forgecdn.net/attachments/258/182/skyfactory4_banner.png",
		"zombie invade": "https://media.forgecdn.net/attachments/403/328/banner.png",
		"all the mods": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
		"better mc": "/modpack_better_mc.webp",
		cobblemon: "/modpack_cobblemon.webp",
		"fabulously optimized": "/modpack_fo.webp",
		pixelmon: "https://media.forgecdn.net/attachments/305/760/banner.png",
		dawncraft: "https://media.forgecdn.net/attachments/474/883/banner.png",
		"medieval mc": "https://media.forgecdn.net/attachments/418/850/banner.png",
		sevtech: "https://media.forgecdn.net/attachments/231/13/banner.png",
		stoneblock: "https://media.forgecdn.net/attachments/251/72/banner.png"
	};

	let bannerFailed = $state(false);

	const effectiveBanner = $derived.by(() => {
		if (item.bannerUrl) return item.bannerUrl;
		const lowerTitle = item.title.toLowerCase();
		const lowerSlug = item.slug.toLowerCase();
		for (const [key, banner] of Object.entries(curatedBanners)) {
			if (lowerTitle.includes(key) || lowerSlug.includes(key)) {
				return banner;
			}
		}
		return null;
	});
</script>

<div 
	role="button"
	tabindex="0"
	onclick={() => onOpenDetails(item)}
	onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onOpenDetails(item); } }}
	class="group relative flex flex-col overflow-hidden rounded-2xl border border-border bg-bg-elevated shadow-soft transition-all duration-150 hover:-translate-y-0.5 hover:border-brand-500/40 hover:shadow-elevated cursor-pointer"
>
	<div class="relative h-40 bg-bg-subtle rounded-t-2xl overflow-hidden">
		{#if effectiveBanner && !bannerFailed}
			<img 
				src={effectiveBanner} 
				alt="" 
				class="h-full w-full rounded-t-2xl object-cover transition-transform duration-500 group-hover:scale-105" 
				onerror={() => { bannerFailed = true; }}
			/>
		{:else if item.iconUrl}
			<div class="absolute inset-0 overflow-hidden rounded-t-2xl bg-bg-surface">
				<img src={item.iconUrl} alt="" class="h-full w-full object-cover scale-125 blur-md opacity-40 transition-transform duration-500 group-hover:scale-135" />
				<div class="absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-70"></div>
			</div>
		{:else}
			<div class="absolute inset-0 overflow-hidden rounded-t-2xl">
				<div class="absolute inset-0 bg-gradient-to-br {mesh} to-bg-elevated opacity-70"></div>
			</div>
		{/if}
		<div class="absolute inset-0 bg-gradient-to-t from-bg-elevated via-bg-elevated/20 to-transparent"></div>
		<div class="absolute right-3 top-3 z-10"><SourceBadge source={item.source} /></div>
		<div class="absolute -bottom-4 left-5 z-10 flex h-16 w-16 items-center justify-center overflow-hidden rounded-2xl border border-border-strong bg-bg-elevated p-1 shadow-elevated transition-transform duration-150 group-hover:-translate-y-0.5">
			{#if item.iconUrl}<LazyImage src={item.iconUrl} alt="" class="h-full w-full rounded-xl object-cover" />{:else}<Box class="h-7 w-7 text-brand-400" />{/if}
		</div>
	</div>
	<div class="flex flex-1 flex-col gap-3 p-5 pt-7">
		<h3 class="truncate text-base font-bold text-fg group-hover:text-brand-400">
			<span>{item.title}</span>
		</h3>
		<p class="line-clamp-2 min-h-8 text-xs leading-relaxed text-fg-muted">{item.description}</p>
		<div class="flex items-center gap-1.5 text-xs text-fg-subtle"><Users class="h-3 w-3" /><span class="truncate">{item.author || item.slug}</span></div>
		{#if loaders.length}<div class="flex flex-wrap gap-1.5">{#each loaders as loader}<LoaderBadge {loader} />{/each}</div>{/if}
		<div class="mt-auto flex items-center justify-between gap-2 border-t border-border pt-3">
			<div class="space-y-1 text-xs text-fg-muted"><span class="flex items-center gap-1"><Download class="h-3 w-3" />{downloads}</span>{#if latest}<span class="block text-[10px] text-fg-subtle">MC {latest}</span>{/if}</div>
			<button 
				type="button" 
				class={button({ variant: isInstalled ? "secondary" : "primary", size: "sm", class: "relative z-10" })} 
				onclick={(e) => { e.stopPropagation(); onInstall(item); }} 
				disabled={isInstalling || isInstalled} 
				aria-label={`${isInstalled ? "Instalado" : "Instalar"}: ${item.title}`}
			>
				{#if isInstalling}<Loader2 class="h-3.5 w-3.5 animate-spin" />Instalando{:else if isInstalled}<Check class="h-3.5 w-3.5 text-success" />Instalado{:else if isModpack}<PackagePlus class="h-3.5 w-3.5" />Criar{:else}<Download class="h-3.5 w-3.5" />Instalar{/if}
			</button>
		</div>
	</div>
</div>
