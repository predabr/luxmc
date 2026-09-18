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

	function getModGradient(title: string): string {
		let hash = 0;
		for (let i = 0; i < title.length; i++) {
			hash = (hash << 5) - hash + title.charCodeAt(i);
			hash |= 0;
		}
		const gradients = [
			"from-blue-600/30 via-indigo-950/60 to-[#0f1013]",
			"from-purple-600/30 via-violet-950/60 to-[#0f1013]",
			"from-emerald-600/30 via-teal-950/60 to-[#0f1013]",
			"from-amber-600/30 via-orange-950/60 to-[#0f1013]",
			"from-rose-600/30 via-pink-950/60 to-[#0f1013]",
			"from-cyan-600/30 via-sky-950/60 to-[#0f1013]",
		];
		const idx = Math.abs(hash) % gradients.length;
		return gradients[idx];
	}

	const isModpack = $derived(contentType === "Modpack");
	const btnClass = $derived(
		isModpack
			? "bg-amber-500 hover:bg-amber-400 text-black shadow-amber-500/20 font-bold"
			: "bg-blue-600 hover:bg-blue-500 text-white shadow-blue-500/20 font-bold"
	);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="group bg-[#18191c] hover:bg-[#1e1f25] border border-white/5 hover:border-white/20 rounded-2xl overflow-hidden transition-all duration-300 flex flex-col justify-between shadow-lg hover:shadow-xl hover:shadow-blue-500/10 hover:scale-[1.02] cursor-pointer active:scale-[0.98] [content-visibility:auto] [contain-intrinsic-size:300px_280px]"
	onclick={() => onOpenDetails(item)}
>
	<!-- Top Banner Image / Ambient Mesh -->
	<div class="relative w-full h-32 bg-[#0d0e12] overflow-hidden">
		{#if item.bannerUrl}
			<LazyImage
				src={item.bannerUrl}
				alt={item.title}
				class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
			/>
			<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-black/20 to-transparent"></div>
		{:else}
			<!-- Dynamic Ambient Glow Backdrop with Icon Reflection -->
			<div class="w-full h-full bg-gradient-to-br {getModGradient(item.title)} relative flex items-center justify-center overflow-hidden">
				{#if item.iconUrl}
					<img 
						src={item.iconUrl} 
						alt="" 
						class="absolute inset-0 w-full h-full object-cover blur-2xl opacity-40 scale-150 transform group-hover:scale-175 transition-transform duration-700" 
						aria-hidden="true"
					/>
				{/if}
				<div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-white/10 via-transparent to-black/60 pointer-events-none"></div>
				<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-black/30"></div>
			</div>
		{/if}

		<div class="absolute top-2.5 right-2.5 z-10">
			<SourceBadge source={item.source} />
		</div>

		<!-- Square Icon Thumbnail Box (Bottom Left) -->
		<div class="absolute bottom-2.5 left-3 h-12 w-12 rounded-2xl bg-[#14151a]/90 backdrop-blur-md border border-white/15 p-1 shadow-2xl flex items-center justify-center overflow-hidden shrink-0 z-10 group-hover:border-white/30 transition-all">
			{#if item.iconUrl}
				<LazyImage
					src={item.iconUrl}
					alt={item.title}
					class="w-full h-full object-cover rounded-xl"
				/>
			{:else}
				<div class="w-full h-full rounded-xl bg-gradient-to-br from-white/10 to-white/5 flex items-center justify-center text-white/70 text-xs font-black">
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
