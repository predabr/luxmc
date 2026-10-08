<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { openUrl } from "@tauri-apps/plugin-opener";
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
		activeTab?: "overview" | "gallery" | "changelog" | "versions";
		isInstalling?: boolean;
		isInstalled?: boolean;
		contentType?: string;
		onBack: () => void;
		onInstall: () => void;
		onOpenGuide: () => void;
		onInstallVersion: (versionId: string) => void;
		children: import("svelte").Snippet;
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
			return date.toLocaleDateString(currentUiLocale(), { day: "2-digit", month: "short", year: "numeric" });
		} catch {
			return d;
		}
	}

	const isModpack = $derived(contentType === "Modpack");
	const effectiveLoaders = $derived(details?.loaders && details.loaders.length > 0 ? details.loaders : item.categories.filter(c => ['fabric', 'forge', 'neoforge', 'quilt'].includes(c.toLowerCase())));
	const effectiveVersions = $derived(details?.gameVersions && details.gameVersions.length > 0 ? details.gameVersions : item.versions);
	const effectiveCategories = $derived(details?.categories && details.categories.length > 0 ? details.categories : item.categories.filter(c => !['fabric', 'forge', 'neoforge', 'quilt'].includes(c.toLowerCase())));
</script>

<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">

	<div class="flex items-center justify-between shrink-0">
		<button
			type="button"
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2 shrink-0" })}
			onclick={onBack}
		>
			<ArrowLeft class="w-4 h-4" />
			<span>{uiText("ui.6e6e6985a71399bd")}</span>
		</button>
		<div class="flex items-center gap-2 text-xs text-fg/40 font-mono shrink-0">
			<span>{uiText("ui.0dd6da8d793f9da3")} <strong class="text-fg uppercase">{item.source}</strong></span>
			<span>•</span>
			<span>{uiText("ui.3ea36adcd1e02c94")} {item.sourceId}</span>
		</div>
	</div>

	<!-- Hero Card -->
	<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-xl relative overflow-hidden shrink-0">
		<div class="absolute -right-16 -top-16 w-64 h-64 bg-[radial-gradient(circle_at_center,rgb(var(--brand-400)/0.1),transparent_70%)] rounded-full pointer-events-none"></div>
		<div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-6 relative z-10 w-full">
			<div class="flex items-start sm:items-center gap-5 min-w-0 flex-1">
				<div class="w-20 h-20 shrink-0 rounded-2xl bg-bg-subtle border border-fg/15 p-1 overflow-hidden shadow-2xl flex items-center justify-center">
					{#if details?.iconUrl || item.iconUrl || item.bannerUrl}
						<img loading="lazy" decoding="async"
							src={details?.iconUrl || item.iconUrl || item.bannerUrl || ""}
							alt={item.title}
							class="w-full h-full object-contain p-0.5 rounded-xl"
							onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
						/>
					{:else}
						<Layers class="w-8 h-8 text-brand-400" />
					{/if}
				</div>
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2 flex-wrap">
						<h1 class="text-xl font-black text-fg tracking-tight truncate">{item.title}</h1>
						<SourceBadge source={item.source} />
					</div>
					<p class="text-xs text-fg/60 mt-1.5 line-clamp-2 max-w-2xl leading-relaxed">
						{item.description || details?.description || uiText("ui.4d8752e14fe79c1f")}
					</p>
					<div class="flex items-center gap-1.5 flex-wrap mt-3">
						{#each item.categories.slice(0, 4) as cat}
							<span class="text-[10px] font-semibold px-2.5 py-0.5 rounded-lg bg-fg/5 border border-fg/10 text-fg/70 shrink-0">
								{cat}
							</span>
						{/each}
					</div>
				</div>
			</div>
			<div class="shrink-0 flex items-center gap-3 w-full lg:w-auto justify-end mt-2 lg:mt-0">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "lg", class: "w-full lg:w-auto from-brand-400 to-brand-400 hover:from-brand-400 hover:to-brand-400 flex items-center justify-center gap-2.5 disabled:opacity-50 shrink-0" })}
					onclick={onInstall}
					disabled={isInstalling || isInstalled}
				>
					{#if isInstalling}
						<Loader2 class="w-4 h-4 animate-spin text-brand-foreground" />
						<span>{uiText("ui.b1e8e68efcbda240")}</span>
					{:else if isInstalled}
						<Check class="w-4 h-4 text-emerald-950 font-black" />
						<span>{uiText("ui.c885592faeb86f3b")}</span>
					{:else if isModpack}
						<span>{uiText("ui.5c9690515c95c811")}</span>
					{:else}
						<span>{uiText("ui.9951cba901a50134")}</span>
					{/if}
				</button>
			</div>
		</div>
	</div>

	<!-- Tabs + Actions -->
	<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-fg/10 pb-3 shrink-0">
		<div class="flex items-center gap-1 bg-bg-elevated p-1 rounded-2xl border border-fg/5">
			{#each [
				{ key: "overview", label: uiText("ui.f7e888eb3ae0d191"), icon: FileText, count: undefined },
				{ key: "gallery", label: uiText("ui.6660507dd6461e01"), icon: ImageIcon, count: details?.gallery.length },
				{ key: "changelog", label: "Changelog", icon: FileText, count: undefined },
				{ key: "versions", label: uiText("settings.catVersions"), icon: Layers, count: versions.length }
			] as const as tab}
				<button
					type="button"
					class="px-4 py-2 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center gap-2 {activeTab === tab.key ? 'bg-brand-400 text-brand-foreground font-black shadow-md' : 'text-fg/60 hover:text-fg'}"
					onclick={() => activeTab = tab.key}
				>
					<tab.icon class="w-3.5 h-3.5" />
					<span>{tab.label}{tab.count !== undefined ? ` (${tab.count})` : ''}</span>
				</button>
			{/each}
		</div>
		<div class="flex items-center gap-2 flex-wrap text-xs font-bold">
			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
				onclick={onOpenGuide}
			>
				<HelpCircle class="w-3.5 h-3.5 text-brand-400" />
				<span>{uiText("ui.0d989c8815c710b0")}</span>
			</button>
			{#if details?.discordUrl}
				<a
					href={details.discordUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="px-3 py-1.5 rounded-xl bg-bg-subtle hover:bg-bg-subtle text-fg/70 hover:text-fg border border-fg/5 flex items-center gap-1.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer"
					onclick={(e) => { e.preventDefault(); if (details?.discordUrl) void openUrl(details.discordUrl); }}
				>
					<MessageSquare class="w-3.5 h-3.5 text-indigo-400" />
					<span>Discord</span>
				</a>
			{/if}
			{#if details?.sourceUrl}
				<a
					href={details.sourceUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="px-3 py-1.5 rounded-xl bg-bg-subtle hover:bg-bg-subtle text-fg/70 hover:text-fg border border-fg/5 flex items-center gap-1.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer"
					onclick={(e) => { e.preventDefault(); if (details?.sourceUrl) void openUrl(details.sourceUrl); }}
				>
					<Globe class="w-3.5 h-3.5 text-emerald-400" />
					<span>{uiText("ui.283dd78f89984eba")}</span>
				</a>
			{/if}
			{#if details?.donationUrl}
				<a
					href={details.donationUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="px-3 py-1.5 rounded-xl bg-bg-subtle hover:bg-bg-subtle text-fg/70 hover:text-fg border border-fg/5 flex items-center gap-1.5 transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer"
					onclick={(e) => { e.preventDefault(); if (details?.donationUrl) void openUrl(details.donationUrl); }}
				>
					<Heart class="w-3.5 h-3.5 text-rose-400" />
					<span>{uiText("ui.0539d1c42300ff8d")}</span>
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
			<aside class="bg-bg-elevated border border-fg/5 rounded-3xl p-5 shadow-xl space-y-5">
				<div class="flex items-center gap-2 text-xs font-bold text-fg uppercase tracking-wider border-b border-fg/5 pb-3">
					<span class="w-4 h-4 text-brand-400 flex items-center justify-center">✓</span>
					<span>{uiText("ui.07156da5279b23ce")}</span>
				</div>
				{#if details.author}
					<div>
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.b08f360172abfa86")}</span>
						<div class="flex items-center gap-3 bg-bg-elevated p-3 rounded-2xl border border-fg/5">
							{#if details.author.avatarUrl}
								<img loading="lazy" decoding="async"
									src={details.author.avatarUrl}
									alt={details.author.name}
									class="w-10 h-10 rounded-full object-cover border border-fg/10 shrink-0"
									onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
								/>
							{:else}
								<div class="w-10 h-10 rounded-full bg-bg-subtle flex items-center justify-center text-fg/60 font-black text-xs shrink-0">
									{details.author.name ? details.author.name.slice(0, 2).toUpperCase() : "AU"}
								</div>
							{/if}
							<div class="min-w-0">
								<h4 class="text-xs font-extrabold text-fg truncate">{details.author.name || "Autor Desconhecido"}</h4>
								<span class="text-[10px] text-fg/40">{details.author.role || "Criador do Projeto"}</span>
							</div>
						</div>
					</div>
				{:else if item.author}
					<div>
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.b08f360172abfa86")}</span>
						<div class="flex items-center gap-3 bg-bg-elevated p-3 rounded-2xl border border-fg/5">
							<div class="w-10 h-10 rounded-full bg-bg-subtle flex items-center justify-center text-fg/60 font-black text-xs shrink-0">
								{item.author.slice(0, 2).toUpperCase()}
							</div>
							<div class="min-w-0">
								<h4 class="text-xs font-extrabold text-fg truncate">{item.author}</h4>
								<span class="text-[10px] text-fg/40">{uiText("ui.9927daba52e701c4")}</span>
							</div>
						</div>
					</div>
				{/if}
				<div>
					<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.21e93777a47faf9a")}</span>
					<div class="bg-bg-elevated rounded-2xl border border-fg/5 divide-y divide-white/5 text-xs">
						<div class="p-3 flex items-center justify-between">
							<span class="text-fg/40">{uiText("ui.1bdb4add5da2415f")}</span>
							<span class="font-bold text-fg capitalize">{item.source}</span>
						</div>
						<div class="p-3 flex items-center justify-between">
							<span class="text-fg/40">{uiText("mods.downloads")}</span>
							<span class="font-bold text-fg font-mono">{formatDownloads(item.downloads)}</span>
						</div>
						{#if details.createdAt}
							<div class="p-3 flex items-center justify-between">
								<span class="text-fg/40">{uiText("ui.f6d7d1b50677930a")}</span>
								<span class="font-medium text-fg/80">{formatDate(details.createdAt)}</span>
							</div>
						{/if}
						{#if details.updatedAt}
							<div class="p-3 flex items-center justify-between">
								<span class="text-fg/40">{uiText("mods.updated")}</span>
								<span class="font-medium text-fg/80">{formatDate(details.updatedAt)}</span>
							</div>
						{/if}
					</div>
				</div>
				{#if effectiveLoaders.length > 0}
					<div>
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.321385b0db5420d2")}</span>
						<div class="flex flex-wrap gap-1.5">
							{#each effectiveLoaders as l}
								<span class="px-2.5 py-1 rounded-xl bg-fg/5 border border-fg/10 text-xs font-bold text-fg/80 capitalize">{l}</span>
							{/each}
						</div>
					</div>
				{/if}
				{#if effectiveVersions.length > 0}
					<div>
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.ca1f6b638d8a298d")}</span>
						<div class="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto custom-scrollbar pr-1">
							{#each effectiveVersions as v}
								<span class="px-2 py-0.5 rounded-lg bg-bg-elevated border border-fg/5 text-[10px] font-mono text-fg/60">{v}</span>
							{/each}
						</div>
					</div>
				{/if}
				{#if effectiveCategories.length > 0}
					<div>
						<span class="text-[10px] font-bold text-fg/40 uppercase tracking-widest block mb-2">{uiText("ui.3adb5537c59ac03f")}</span>
						<div class="flex flex-wrap gap-1.5">
							{#each effectiveCategories as cat}
								<span class="px-2.5 py-0.5 rounded-full bg-brand-400/10 text-brand-400 border border-brand-400/20 text-[10px] font-bold">{cat}</span>
							{/each}
						</div>
					</div>
				{/if}
			</aside>
		{/if}
	</div>
</div>
