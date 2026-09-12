<script lang="ts">
	import { Download, Users, Loader2, Check, PackagePlus } from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
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
	const btnClass = $derived(
		isModpack
			? "bg-[#caa97c] hover:bg-[#b89565] text-black shadow-[#caa97c]/20"
			: "bg-[#6c5ce7] hover:bg-[#5b4cdb] text-white shadow-[#6c5ce7]/20"
	);
	const bannerSrc = $derived(item.bannerUrl || item.iconUrl || "/vanilla_banner.png");
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="group bg-[#18191c] hover:bg-[#1e1f25] border border-white/5 hover:border-white/20 rounded-2xl overflow-hidden transition-all duration-300 flex flex-col justify-between shadow-lg hover:shadow-xl hover:shadow-[#6c5ce7]/10 hover:scale-[1.02] cursor-pointer active:scale-[0.98] [content-visibility:auto] [contain-intrinsic-size:300px_280px]"
	onclick={() => onOpenDetails(item)}
>
	<!-- Top Banner Image -->
	<div class="relative w-full h-36 bg-[#0f1013] overflow-hidden">
		<LazyImage
			src={bannerSrc}
			alt={item.title}
			class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300 {item.bannerUrl ? '' : 'blur-[1px] opacity-80'}"
			fallback="/vanilla_banner.png"
		/>

		<div class="absolute top-2.5 right-2.5 z-10">
			<SourceBadge source={item.source} />
		</div>

		<!-- Square Icon Thumbnail Box (Bottom Left) -->
		<div class="absolute bottom-2.5 left-3 h-12 w-12 rounded-xl bg-[#14151a] border-2 border-[#202129] p-0.5 shadow-xl flex items-center justify-center overflow-hidden shrink-0">
			{#if item.iconUrl}
				<LazyImage
					src={item.iconUrl}
					alt={item.title}
					class="w-full h-full object-cover rounded-lg"
				/>
			{:else}
				<div class="w-full h-full rounded-lg bg-white/5 flex items-center justify-center text-white/40 text-[10px] font-black">
					{item.title.slice(0, 2).toUpperCase()}
				</div>
			{/if}
		</div>
	</div>

	<!-- Card Bottom: Details -->
	<div class="p-3.5 flex-1 flex flex-col justify-between">
		<div>
			<div class="flex items-center justify-between gap-2">
				<h3 class="font-bold text-white text-xs truncate flex-1 group-hover:text-[#a29bfe] transition-colors" title={item.title}>
					{item.title}
				</h3>
				<span class="text-[11px] text-white/50 flex items-center gap-1 shrink-0 font-medium">
					<Download class="w-2.5 h-2.5 text-white/40" />
					{formatDownloads(item.downloads)}
				</span>
			</div>
			<p class="text-[11px] text-white/40 mt-1.5 line-clamp-2 leading-relaxed h-8">
				{item.description}
			</p>
		</div>

		<div class="flex items-center justify-between mt-3 pt-2.5 border-t border-white/[0.06]">
			<div class="flex items-center gap-1 text-[11px] text-white/40 truncate max-w-[55%]">
				<Users class="w-3 h-3 text-white/30 shrink-0" />
				<span class="truncate font-medium">{item.author || item.slug}</span>
			</div>

			<button
				type="button"
				class="{btnClass} active:scale-95 text-xs font-semibold px-3.5 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-md disabled:opacity-50 cursor-pointer shrink-0"
				onclick={(e) => { e.stopPropagation(); onInstall(item); }}
				disabled={isInstalling || isInstalled}
			>
				{#if isInstalling}
					<Loader2 class="w-3 h-3 animate-spin" />
					<span>Instalando</span>
				{:else if isInstalled}
					<Check class="w-3 h-3 text-emerald-300" />
					<span>Instalado</span>
				{:else if isModpack}
					<PackagePlus class="w-3 h-3" />
					<span>Criar</span>
				{:else}
					<Download class="w-3 h-3" />
					<span>Instalar</span>
				{/if}
			</button>
		</div>
	</div>
</div>
