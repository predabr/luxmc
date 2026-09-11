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
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="group bg-[#18191c] hover:bg-[#1e1f25] border border-white/5 hover:border-white/20 rounded-2xl p-3 flex items-center justify-between gap-4 transition-all duration-200 shadow-md cursor-pointer active:scale-[0.99]"
	onclick={() => onOpenDetails(item)}
>
	<!-- Left Icon & Info -->
	<div class="flex items-center gap-3.5 min-w-0 flex-1">
		<div class="h-12 w-12 rounded-xl bg-[#101115] border border-white/10 p-0.5 shrink-0 overflow-hidden flex items-center justify-center">
			{#if item.iconUrl}
				<LazyImage
					src={item.iconUrl}
					alt={item.title}
					class="w-full h-full object-cover rounded-lg"
				/>
			{:else}
				<div class="w-full h-full rounded-lg bg-white/5 flex items-center justify-center text-white/40 text-xs font-black">
					{item.title.slice(0, 2).toUpperCase()}
				</div>
			{/if}
		</div>

		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2.5">
				<h3 class="font-bold text-white text-xs truncate group-hover:text-[#a29bfe] transition-colors">{item.title}</h3>
				<span class="text-[10px] text-white/40 flex items-center gap-1 font-mono">
					<Download class="w-2.5 h-2.5" /> {formatDownloads(item.downloads)}
				</span>
				<span class="text-[10px] text-white/30">•</span>
				<span class="text-[10px] text-white/40 flex items-center gap-1">
					<Users class="w-2.5 h-2.5" /> {item.author || item.slug}
				</span>
			</div>
			<p class="text-[11px] text-white/40 truncate mt-0.5">{item.description}</p>
		</div>
	</div>

	<!-- Right: Source Badge & Install Button -->
	<div class="flex items-center gap-3 shrink-0">
		<SourceBadge source={item.source} />

		<button
			type="button"
			class="{btnClass} active:scale-95 text-xs font-semibold px-4 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-md disabled:opacity-50 cursor-pointer"
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
				<span>Criar Instância</span>
			{:else}
				<Download class="w-3 h-3" />
				<span>Instalar</span>
			{/if}
		</button>
	</div>
</div>
