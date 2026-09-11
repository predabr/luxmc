<script lang="ts">
	import {
		ArrowLeft, CheckCircle2, Download, Loader2, Check, PackagePlus,
		HelpCircle, MessageSquare, Globe, Heart, FileText, ImageIcon, Layers
	} from "lucide-svelte";
	import LazyImage from "$lib/components/ui/LazyImage.svelte";
	import SourceBadge from "./SourceBadge.svelte";
	import type { ModSearchResultItem, ModProjectDetails, ModVersion } from "$lib/api";

	let {
		item,
		details = null,
		versions = [],
		loadingDetails = false,
		activeTab = $bindable("overview"),
		isInstalling = false,
		isInstalled = false,
		contentType = "Mod",
		onBack,
		onInstall,
		onOpenGuide,
		onInstallVersion,
		children
	}: {
		item: ModSearchResultItem;
		details: ModProjectDetails | null;
		versions: ModVersion[];
		loadingDetails?: boolean;
		activeTab?: "overview" | "gallery" | "versions";
		isInstalling?: boolean;
		isInstalled?: boolean;
		contentType?: string;
		onBack: () => void;
		onInstall: () => void;
		onOpenGuide: () => void;
		onInstallVersion: (versionId: string) => void;
		children: any;
	} = $props();

	function formatDownloads(n: number): string {
		if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
		if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
		return n.toString();
	}

	function formatDate(d?: string | null): string {
		if (!d) return "Recente";
		try {
			const date = new Date(d);
			return date.toLocaleDateString("pt-BR", { day: "2-digit", month: "short", year: "numeric" });
		} catch {
			return d;
		}
	}

	const isModpack = $derived(contentType === "Modpack");
</script>

<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">

	<div class="flex items-center justify-between shrink-0">
		<button
			type="button"
			class="flex items-center gap-2 px-3.5 py-2 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/10 text-xs font-bold transition-all shadow-sm cursor-pointer active:scale-95 shrink-0"
			onclick={onBack}
		>
			<ArrowLeft class="w-4 h-4" />
			<span>Voltar ao Catálogo</span>
		</button>
		<div class="flex items-center gap-2 text-xs text-white/40 font-mono shrink-0">
			<span>Fonte: <strong class="text-white uppercase">{item.source}</strong></span>
			<span>•</span>
			<span>ID: {item.sourceId}</span>
		</div>
	</div>

	<!-- Hero Card -->
	<div class="bg-[#18191c] border border-white/10 rounded-3xl p-6 shadow-xl relative overflow-hidden shrink-0">
		<div class="absolute -right-16 -top-16 w-64 h-64 bg-[#caa97c]/5 rounded-full blur-3xl pointer-events-none"></div>
		<div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6 relative z-10 w-full">
			<div class="flex items-start sm:items-center gap-5 min-w-0 flex-1">
				<div class="w-20 h-20 shrink-0 rounded-2xl bg-[#222328] border border-white/15 p-1 overflow-hidden shadow-2xl flex items-center justify-center">
					{#if item.iconUrl}
						<img src={item.iconUrl} alt={item.title} class="w-full h-full object-cover rounded-xl" />
					{:else}
						<Layers class="w-8 h-8 text-[#caa97c]" />
					{/if}
				</div>
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2 flex-wrap">
						<h1 class="text-xl font-black text-white tracking-tight truncate">{item.title}</h1>
						<div class="flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 shrink-0">
							<CheckCircle2 class="w-3 h-3" /> Verificado
						</div>
						<SourceBadge source={item.source} />
					</div>
					<p class="text-xs text-white/60 mt-1.5 line-clamp-2 max-w-2xl leading-relaxed">
						{item.description}
					</p>
					<div class="flex items-center gap-1.5 flex-wrap mt-3">
						{#each item.categories.slice(0, 4) as cat}
							<span class="text-[10px] font-semibold px-2.5 py-0.5 rounded-lg bg-white/5 border border-white/10 text-white/70 shrink-0">
								{cat}
							</span>
						{/each}
					</div>
				</div>
			</div>
			<div class="shrink-0 flex items-center gap-3 w-full lg:w-auto justify-end mt-2 lg:mt-0">
				<button
					type="button"
					class="w-full lg:w-auto bg-gradient-to-r from-[#caa97c] to-[#e4c99c] hover:from-[#d5b588] hover:to-[#edd5ad] text-black font-extrabold text-xs px-7 py-3.5 rounded-2xl flex items-center justify-center gap-2.5 shadow-[0_4px_24px_rgba(202,169,124,0.35)] transition-all cursor-pointer active:scale-95 disabled:opacity-50 shrink-0 hover:scale-[1.02]"
					onclick={onInstall}
					disabled={isInstalling || isInstalled}
				>
					{#if isInstalling}
						<Loader2 class="w-4 h-4 animate-spin text-black" />
						<span>Instalando...</span>
					{:else if isInstalled}
						<Check class="w-4 h-4 text-emerald-950 font-black" />
						<span>Instalado na Instância</span>
					{:else if isModpack}
						<span>Criar Instância do Modpack</span>
					{:else}
						<span>Instalar na Instância</span>
					{/if}
				</button>
			</div>
		</div>
	</div>

	<!-- Tabs + Actions -->
	<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/10 pb-3 shrink-0">
		<div class="flex items-center gap-1 bg-[#18191c] p-1 rounded-2xl border border-white/5">
			{#each [
				{ key: "overview", label: "Visão Geral", icon: FileText },
				{ key: "gallery", label: "Galeria", icon: ImageIcon, count: details?.gallery.length },
				{ key: "versions", label: "Versões", icon: Layers, count: versions.length }
			] as tab}
				<button
					type="button"
					class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-2 {activeTab === tab.key ? 'bg-[#caa97c] text-black font-black shadow-md' : 'text-white/60 hover:text-white'}"
					onclick={() => activeTab = tab.key as any}
				>
					<tab.icon class="w-3.5 h-3.5" />
					<span>{tab.label}{tab.count !== undefined ? ` (${tab.count})` : ''}</span>
				</button>
			{/each}
		</div>
		<div class="flex items-center gap-2 flex-wrap text-xs font-bold">
			<button
				type="button"
				class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all cursor-pointer"
				onclick={onOpenGuide}
			>
				<HelpCircle class="w-3.5 h-3.5 text-[#caa97c]" />
				<span>Como Instalar</span>
			</button>
			{#if details?.discordUrl}
				<a href={details.discordUrl} target="_blank" class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all">
					<MessageSquare class="w-3.5 h-3.5 text-indigo-400" />
					<span>Discord</span>
				</a>
			{/if}
			{#if details?.sourceUrl}
				<a href={details.sourceUrl} target="_blank" class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all">
					<Globe class="w-3.5 h-3.5 text-emerald-400" />
					<span>Código Fonte</span>
				</a>
			{/if}
			{#if details?.donationUrl}
				<a href={details.donationUrl} target="_blank" class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all">
					<Heart class="w-3.5 h-3.5 text-rose-400" />
					<span>Apoiar</span>
				</a>
			{/if}
		</div>
	</div>

	<!-- Main Split Content Area -->
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start pb-12">
		<div class="lg:col-span-2 space-y-6">
			{@render children()}
		</div>

		<!-- Right Sidebar: Info Panel -->
		{#if details}
			<aside class="bg-[#18191c] border border-white/5 rounded-3xl p-5 shadow-xl space-y-5">
				<div class="flex items-center gap-2 text-xs font-bold text-white uppercase tracking-wider border-b border-white/5 pb-3">
					<span class="w-4 h-4 text-[#caa97c] flex items-center justify-center">✓</span>
					<span>Informações</span>
				</div>
				{#if details.author}
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">AUTOR</span>
						<div class="flex items-center gap-3 bg-[#131417] p-3 rounded-2xl border border-white/5">
							{#if details.author.avatarUrl}
								<img src={details.author.avatarUrl} alt={details.author.name} class="w-10 h-10 rounded-full object-cover border border-white/10 shrink-0" />
							{:else}
								<div class="w-10 h-10 rounded-full bg-[#2a2b33] flex items-center justify-center text-white/60 font-black text-xs shrink-0">
									{details.author.name.slice(0, 2).toUpperCase()}
								</div>
							{/if}
							<div>
								<h4 class="text-xs font-extrabold text-white">{details.author.name}</h4>
								<span class="text-[10px] text-white/40">{details.author.role || "Criador do Projeto"}</span>
							</div>
						</div>
					</div>
				{/if}
				<div>
					<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">DETALHES</span>
					<div class="bg-[#131417] rounded-2xl border border-white/5 divide-y divide-white/5 text-xs">
						<div class="p-3 flex items-center justify-between">
							<span class="text-white/40">Fonte</span>
							<span class="font-bold text-white capitalize">{item.source}</span>
						</div>
						<div class="p-3 flex items-center justify-between">
							<span class="text-white/40">Downloads</span>
							<span class="font-bold text-white font-mono">{formatDownloads(item.downloads)}</span>
						</div>
						{#if details.createdAt}
							<div class="p-3 flex items-center justify-between">
								<span class="text-white/40">Criado em</span>
								<span class="font-medium text-white/80">{formatDate(details.createdAt)}</span>
							</div>
						{/if}
						{#if details.updatedAt}
							<div class="p-3 flex items-center justify-between">
								<span class="text-white/40">Atualizado</span>
								<span class="font-medium text-white/80">{formatDate(details.updatedAt)}</span>
							</div>
						{/if}
					</div>
				</div>
				{#if details.loaders.length > 0}
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">LOADERS COMPATÍVEIS</span>
						<div class="flex flex-wrap gap-1.5">
							{#each details.loaders as l}
								<span class="px-2.5 py-1 rounded-xl bg-white/5 border border-white/10 text-xs font-bold text-white/80">{l}</span>
							{/each}
						</div>
					</div>
				{/if}
				{#if details.gameVersions.length > 0}
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">VERSÕES SUPORTADAS</span>
						<div class="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto custom-scrollbar pr-1">
							{#each details.gameVersions as v}
								<span class="px-2 py-0.5 rounded-lg bg-[#131417] border border-white/5 text-[10px] font-mono text-white/60">{v}</span>
							{/each}
						</div>
					</div>
				{/if}
				{#if details.categories.length > 0}
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">CATEGORIAS</span>
						<div class="flex flex-wrap gap-1.5">
							{#each details.categories as cat}
								<span class="px-2.5 py-0.5 rounded-full bg-[#caa97c]/10 text-[#caa97c] border border-[#caa97c]/20 text-[10px] font-bold">{cat}</span>
							{/each}
						</div>
					</div>
				{/if}
			</aside>
		{/if}
	</div>
</div>
