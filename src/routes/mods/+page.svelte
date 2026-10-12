<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { button } from "$lib/components/ui/button";
	import { sanitizeHtml } from "$lib/utils/sanitizeHtml";
	import { onMount, untrack } from "svelte";
	import { page } from "$app/state";
	import { deepLinks } from "$lib/stores/deepLinks.svelte";
	import type { DeepLinkAction } from "$lib/utils/deepLink";
	import {
		Search, LayoutGrid, List, Loader2, AlertTriangle,
		Check, ChevronLeft, ChevronRight, ChevronDown,
		Sparkles, Package, ArrowUpRight, SlidersHorizontal
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { catalogCompatibility } from "$lib/utils/catalogCompatibility";
	import { profiles } from "$lib/stores/profiles.svelte";
    import { settings } from '$lib/stores/settings.svelte';
	import { modpackInstallation } from "$lib/stores/modpackInstallation.svelte";
	import {
		modsSearch, modsVersions, modsInstall, modsProjectDetails, modsChangelog,
		modsDownloadToTemp, instanceImportMrpack, instanceImportModpack, instanceCancelImport,
		curseforgeStatus,
		type ModProjectDetails, type ModVersion, type ModSearchResultItem
	} from "$lib/api";
	import { listen } from "$lib/api/client";
	import { marked } from "marked";
	import CatalogResults from "$lib/components/mods/CatalogResults.svelte";
	import ModCard from "$lib/components/mods/ModCard.svelte";
	import ModCardList from "$lib/components/mods/ModCardList.svelte";
	import ModSearchBar from "$lib/components/mods/ModSearchBar.svelte";
	import ModDetailView from "$lib/components/mods/ModDetailView.svelte";
    import ModChangelog from '$lib/components/mods/ModChangelog.svelte';
	import ModVersionPicker from "$lib/components/mods/ModVersionPicker.svelte";
	import ModpackInstaller from "$lib/components/mods/ModpackInstaller.svelte";
	import InstancePickerModal from "$lib/components/mods/InstancePickerModal.svelte";
	import InstallGuideModal from "$lib/components/mods/InstallGuideModal.svelte";
	import LightboxModal from "$lib/components/mods/LightboxModal.svelte";
    import { goto } from '$app/navigation';
    import CatalogNavigation from '$lib/components/mods/CatalogNavigation.svelte';
    import { pageMotion } from '$lib/actions/pageMotion';

	let targetInstanceId = $state<string>(profiles.activeId ?? profiles.list[0]?.id ?? "default");

	$effect(() => {
		if ((!targetInstanceId || targetInstanceId === "default") && profiles.list.length > 0) {
			targetInstanceId = profiles.activeId ?? profiles.list[0].id;
		}
	});

	let showModpackInstallModal = $state(false);
	let modpackToInstall = $state<ModSearchResultItem | null>(null);
	let modpackVersionId = $state<string | undefined>();
	let modpackInstanceName = $state("");
	let modpackRamMb = $state(4096);
	const isInstallingModpack = $derived(modpackInstallation.state.busy);
    const cancelRequested = $derived(modpackInstallation.state.cancelling);
	let modpackProgressText = $state("");
	let modpackProgressPercent = $state(0);

	let showInstancePickerModal = $state(false);
	let itemToInstall = $state<{ item: ModSearchResultItem; versionId?: string } | null>(null);
	let chosenWorldName = $state("");
	let chosenInstanceId = $state<string>(profiles.activeId ?? profiles.list[0]?.id ?? "");

	let curseforgeActive = $state(true);
	let searchQuery = $state("");
	let selectedSource = $state<"all" | "modrinth" | "curseforge">("all");
	let selectedType = $state("Modpack");
    $effect(() => { if (selectedType === "World") untrack(() => { selectedSource = "curseforge"; }); });
	let selectedLoader = $state<string | null>(null);
	let selectedCategory = $state<string | null>(null);
	let selectedVersion = $state("Qualquer Versão");
	let selectedSort = $state<"downloads" | "relevance" | "updated" | "newest">("downloads");
	$effect(() => {
		const query = page.url.search;
		untrack(() => {
			const params = new URLSearchParams(query);
			const contentTypes: Record<string, string> = { mod: "Mod", modpack: "Modpack", resourcepack: "Resource Pack", shader: "Shader", datapack: "Data Pack", world: "World" };
			selectedType = contentTypes[params.get("type") ?? ""] ?? "Modpack";
			searchQuery = params.get("search") ?? "";
			const profile = profiles.list.find(profile => profile.id === params.get("instance"));
			if (profile) {
				targetInstanceId = profile.id;
				chosenInstanceId = profile.id;
				selectedVersion = profile.mcVersion;
				selectedLoader = selectedType === "Mod" && profile.loader !== "vanilla" ? profile.loader : null;
			}
		});
	});
	let sortMenuOpen = $state(false);
	let viewMode = $state<"grid" | "list">("grid");
    let filtersOpen = $state(false);

    let resultsWidth = $state(0);
    const resultColumns = $derived(resultsWidth >= 1050 ? 3 : resultsWidth >= 650 ? 2 : 1);

	const sortOptions = [
		{ id: "downloads", label: uiText("mods.downloads") },
		{ id: "relevance", label: uiText("mods.relevance") },
		{ id: "updated", label: uiText("ui.115b65962f4c2815") },
		{ id: "newest", label: uiText("ui.ade822bf50f985e6") }
	] as const;

	const currentSortLabel = $derived(sortOptions.find(s => s.id === selectedSort)?.label ?? "Downloads");
	const activeFilters = $derived([
        ...(selectedSource !== "all" ? [{ label: selectedSource === "modrinth" ? "Modrinth" : "CurseForge", clear: () => selectedSource = "all" as const }] : []),
		...(selectedVersion !== "Qualquer Versão" ? [{ label: selectedVersion, clear: () => selectedVersion = "Qualquer Versão" }] : []),
		...(selectedLoader ? [{ label: selectedLoader, clear: () => selectedLoader = null }] : []),
		...(selectedCategory ? [{ label: selectedCategory, clear: () => selectedCategory = null }] : [])
	]);

	function resetFilters() {
		searchQuery = "";
		selectedVersion = "Qualquer Versão";
		selectedLoader = null;
		selectedCategory = null;
		selectedSource = "all";
	}

	let results = $state<Array<ModSearchResultItem>>([]);
	let loading = $state(false);
	let searchError = $state<string | null>(null);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let currentPage = $state(1);
	const pageSize = $derived(settings.value.catalogPageSize || 36);
	let installingIds = $state<Set<string>>(new Set());
	let installedIds = $state<Set<string>>(new Set());
	let hasSearched = $state(false);

	const hasMoreResults = $derived(results.length >= pageSize);
	const totalPages = $derived(hasMoreResults ? currentPage + 1 : Math.max(1, currentPage));
	const totalEstimate = $derived(
		hasMoreResults
			? `${(currentPage * pageSize).toLocaleString(currentUiLocale())}+`
			: `${Math.max(results.length, (currentPage - 1) * pageSize + results.length)}`
	);
	const pageNumbers = $derived.by(() => {
		const windowSize = Math.min(4, Math.max(1, totalPages));
		const start = Math.max(1, Math.min(currentPage - 1, totalPages - windowSize + 1));
		const end = Math.min(totalPages, start + windowSize - 1);
		const pages: number[] = [];
		for (let p = start; p <= end; p++) pages.push(p);
		return pages;
	});
	const showPager = $derived(hasSearched && !loading && !searchError && (results.length > 0 || currentPage > 1));

	let selectedItem = $state<ModSearchResultItem | null>(null);
	let modDetails = $state<ModProjectDetails | null>(null);
	let modVersionsList = $state<ModVersion[]>([]);
	let loadingDetails = $state(false);
	let activeDetailTab = $state<"overview" | "gallery" | "changelog" | "versions">("overview");
	let lightboxImage = $state<{ url: string; title?: string | null; description?: string | null } | null>(null);
	let showInstallGuide = $state(false);

	function renderMarkdown(content: string): string {
		if (!content) return "";
		try {
			return marked.parse(content, { breaks: true, gfm: true }) as string;
		} catch {
			return content;
		}
	}

	function createYouTubeVideoCard(vid: string): string {
		return uiText("ui.423dbb1457a25e45", {arg0: (vid), arg1: (vid), arg2: (vid), arg3: (vid)});
	}

	const YOUTUBE_ID_PATTERN = /^[A-Za-z0-9_-]{11}$/;

	function playVideoFromElement(element: EventTarget | null): boolean {
		const target = element as HTMLElement | null;
		const playBtn = target?.closest(".btn-play-video");
		if (!playBtn) return false;
		const container = playBtn.closest(".video-player-container");
		const vid = playBtn.getAttribute("data-video-id");
		if (!container || !vid || !YOUTUBE_ID_PATTERN.test(vid)) return false;

		const iframe = document.createElement("iframe");
		iframe.src = `https://luxmc-r92.pages.dev/api/player?video=${encodeURIComponent(vid)}`;
		iframe.referrerPolicy = "strict-origin-when-cross-origin";
		iframe.title = "YouTube video player";
		iframe.className = "w-full h-full border-0";
		iframe.setAttribute(
			"allow",
			"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
		);
		iframe.setAttribute("sandbox", "allow-scripts allow-same-origin allow-presentation");
		iframe.setAttribute("allowfullscreen", "");
		container.replaceChildren(iframe);
		return true;
	}

	function processDescription(body: string, isHtml: boolean): string {
		if (!body) return "";
		let html = isHtml ? body : renderMarkdown(body);

		html = html.replace(
			/<iframe[^>]*src=["'](?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/embed\/|youtube-nocookie\.com\/embed\/|youtu\.be\/)([\w-]+)[^"']*["'][^>]*>[\s\S]*?<\/iframe>/gi,
			(_m, vid) => createYouTubeVideoCard(vid)
		);

		html = html.replace(
			/<a[^>]*href=["'](?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/(?:watch\?v=|embed\/)|youtu\.be\/)([\w-]{11})[^"']*["'][^>]*>([\s\S]*?)<\/a>/gi,
			(match, vid, text) => {
				const lowerText = String(text || "").toLowerCase();
				if (
					lowerText.includes("youtube") ||
					lowerText.includes("trailer") ||
					lowerText.includes("vídeo") ||
					lowerText.includes("video") ||
					lowerText.includes("watch") ||
					lowerText.includes("showcase") ||
					lowerText.includes("youtu.be") ||
					String(text || "").trim() === match.trim()
				) {
					return createYouTubeVideoCard(vid);
				}
				return `<a href="https://www.youtube.com/watch?v=${vid}" target="_blank" rel="noopener noreferrer" class="text-brand-400 hover:text-brand-300 underline font-bold">${text} ↗</a>`;
			}
		);

		html = html.replace(/<iframe[^>]*src=["']([^"']+)["'][^>]*>[\s\S]*?<\/iframe>/gi, (_m, src) => uiText("ui.9a48920d166a8b5c", {arg0: (src)})
		);

		html = html.replace(/<video[^>]*src=["']([^"']+)["'][^>]*>[\s\S]*?<\/video>/gi, (_m, src) => uiText("ui.02ee3107a0e16bc0", {arg0: (src), arg1: (src)})
		);
		html = html.replace(/<video[^>]*>[\s\S]*?<source[^>]*src=["']([^"']+)["'][^>]*>[\s\S]*?<\/video>/gi, (_m, src) => uiText("ui.02ee3107a0e16bc0", {arg0: (src), arg1: (src)})
		);

		html = html.replace(/<a\s+(?![^>]*\btarget=)([^>]+)>/gi, '<a target="_blank" rel="noopener noreferrer" $1>');

		return sanitizeHtml(html);
	}

	let searchToken = 0;
    let previousQuery: string | undefined;

	async function doSearch(page = 1) {
		const token = ++searchToken;
		const root = document.querySelector<HTMLElement>("[data-scroll-root]");
		if (root) root.scrollTop = 0;
		loading = true; searchError = null; hasSearched = true; currentPage = page;
		try {
			const ver = selectedVersion === "Qualquer Versão" ? "" : selectedVersion;
			const found = await modsSearch(searchQuery.trim(), ver, pageSize, (page - 1) * pageSize,
				selectedType.toLowerCase().replace(/\s+/g, ""), selectedSort,
				selectedLoader ?? undefined, selectedCategory ?? undefined, selectedSource);
			if (token !== searchToken) return;
			results = found;
		} catch (e) {
			if (token !== searchToken) return;
			searchError = e instanceof Error ? e.message : String(e); results = [];
		} finally { if (token === searchToken) loading = false; }
	}

	function goToPage(p: number) {
		if (p < 1 || loading) return;
		const root = document.querySelector<HTMLElement>("[data-scroll-root]");
		if (root) root.scrollTop = 0;
		void doSearch(p);
	}

	$effect(() => {
		searchQuery; selectedVersion; selectedType; selectedSource;
		selectedLoader; selectedCategory; selectedSort; pageSize;
        const typing = previousQuery !== undefined && previousQuery !== searchQuery;
        previousQuery = searchQuery;
		searchToken++;
		loading = true;
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => doSearch(1), typing ? 250 : 0);
		return () => { searchToken++; if (debounceTimer) clearTimeout(debounceTimer); };
	});

	onMount(() => {
		curseforgeStatus().then(s => { curseforgeActive = s; }).catch(() => { curseforgeActive = true; });
	});

    $effect(() => {
        const request = deepLinks.install;
        if (!request || showInstancePickerModal || showModpackInstallModal) return;
        untrack(() => { deepLinks.install = null; void openLinkedProject(request); });
    });

    async function openLinkedProject(request: Extract<DeepLinkAction, { kind: "install" }>): Promise<void> {
        try {
            const details = await modsProjectDetails(request.id, request.source);
            const item: ModSearchResultItem = {
                sourceId: details.id, source: request.source, slug: details.slug,
                title: details.title, description: details.description, downloads: details.downloads,
                iconUrl: details.iconUrl, bannerUrl: details.gallery[0]?.url ?? null,
                author: details.author?.name ?? null, categories: details.categories, versions: details.gameVersions
            };
            selectedSource = request.source;
            selectedType = request.content === "modpack" ? "Modpack" : "Mod";
            if (request.content === "modpack") { openModpackInstall(item); return; }
            const compatible = profiles.list.filter(profile =>
                (!details.gameVersions.length || details.gameVersions.includes(profile.mcVersion)) &&
                (!details.loaders.length || details.loaders.includes(profile.loader)));
            targetInstanceId = compatible.find(profile => profile.id === profiles.activeId)?.id ?? compatible[0]?.id ?? "";
            if (!targetInstanceId) throw new Error(uiText("ui.844ab9e35683af6e"));
            itemToInstall = { item };
            chosenInstanceId = targetInstanceId;
            showInstancePickerModal = true;
        } catch (error) { toast(uiText("ui.67ea146a2d807fae", {arg0: (String(error))}), "error"); }
    }

	let detailsToken = 0;
	const detailCache = new Map<string, { detail: Awaited<ReturnType<typeof modsProjectDetails>>; versions: Awaited<ReturnType<typeof modsVersions>>; time: number }>();
	async function openDetails(item: ModSearchResultItem) {
		const token = ++detailsToken;
		const key = `${item.source}:${item.sourceId}`;
		const cached = detailCache.get(key);
		if (cached && Date.now() - cached.time < 300000) {
			selectedItem = item; modDetails = cached.detail; modVersionsList = cached.versions;
			loadingDetails = false; activeDetailTab = "overview";
		} else {
			selectedItem = item; modDetails = null; modVersionsList = [];
			loadingDetails = true; activeDetailTab = "overview";
		}
		try {
			const detailRequest = modsProjectDetails(item.sourceId, item.source).then(detail => {
				if (token === detailsToken) { modDetails = detail; loadingDetails = false; }
				return detail;
			});
			const [d, v] = await Promise.all([detailRequest, modsVersions(item.sourceId, "", item.source).catch(() => [])]);
			if (detailCache.size >= 30) detailCache.delete(detailCache.keys().next().value!);
			detailCache.set(key, { detail: d, versions: v, time: Date.now() });
			if (token !== detailsToken) return;
			modDetails = d; modVersionsList = v;
		} catch (e) {
			if (token !== detailsToken) return;
			toast(uiText("ui.64e3285c73f9cf91") + String(e), "error");
		} finally { if (token === detailsToken) loadingDetails = false; }
	}

	function closeDetails() { detailsToken++; selectedItem = null; modDetails = null; modVersionsList = []; loadingDetails = false; }

	function isItemModpack(_item: ModSearchResultItem): boolean {
		return selectedType === "Modpack";
	}

	function promptInstall(item: ModSearchResultItem, versionId?: string) {
		if (isItemModpack(item)) { openModpackInstall(item, versionId); return; }
		if (profiles.list.length === 0) { toast(uiText("ui.f00ac39128b9de9d"), "error"); return; }
		const compatible = profiles.list.filter(profile => !catalogCompatibility(item, profile, selectedType, false));
		
		itemToInstall = { item, versionId };
		chosenInstanceId = compatible.find(profile => profile.id === targetInstanceId)?.id ?? compatible.find(profile => profile.id === profiles.activeId)?.id ?? compatible[0]?.id ?? "";
		showInstancePickerModal = true;
	}

	function openModpackInstall(item: ModSearchResultItem, versionId?: string) {
		modpackVersionId = versionId;
		modpackToInstall = item; modpackInstanceName = item.title; modpackRamMb = 4096;
        modpackProgressText = ""; showModpackInstallModal = true;
	}

    function confirmModpackInstall() {
        if (!modpackToInstall) return;
        void modpackInstallation.start(modpackToInstall, modpackInstanceName.trim() || modpackToInstall.title, modpackRamMb, selectedVersion, modpackVersionId);
        showModpackInstallModal = false;
    }
    function cancelModpackInstall() { return modpackInstallation.cancel(); }

	async function confirmInstanceInstall() {
		if (!itemToInstall) return;
		const { item, versionId } = itemToInstall;
		showInstancePickerModal = false;
		await executeInstallItem(item, chosenInstanceId, versionId);
		itemToInstall = null;
	}

	async function executeInstallItem(item: ModSearchResultItem, profileId: string, versionId?: string) {
		const id = `${item.source}:${item.sourceId}`;
		if (installingIds.has(id)) return;
		installingIds = new Set([...installingIds, id]);
		try {
			const pid = profileId || targetInstanceId || profiles.activeId || profiles.list[0]?.id || "default";
			const prof = profiles.list.find(p => p.id === pid);
			if (!prof) throw new Error(uiText("ui.57d70370f60875d9"));
			const incompatibility = catalogCompatibility(item, prof, selectedType, false);
			if (incompatibility) throw new Error(incompatibility);
            const tv = prof.mcVersion;
            const vs = await modsVersions(item.sourceId, tv, item.source);
            const candidates = vs.filter(version => selectedType !== "Mod" || !version.loaders?.length || version.loaders.includes(prof.loader));
            const chosenVersion = versionId ? candidates.find(version => version.id === versionId) : candidates[0];
            if (!chosenVersion) throw new Error(uiText("ui.6c519ffd1be4eabf", {arg0: (item.title), arg1: (tv), arg2: (selectedType === "Mod" ? ` / ${prof.loader}` : "")}));
            const vid = chosenVersion.id;

			const ct = selectedType.toLowerCase().replace(/\s+/g, "");
			await modsInstall({ profileId: pid, projectId: item.sourceId, versionId: vid, source: item.source, contentType: ct, worldName: ct === "datapack" ? chosenWorldName : undefined, iconUrl: item.iconUrl });
			if (pid && pid !== "default") {
				const p = profiles.list.find(p => p.id === pid);
				if (p && ct === "mod") profiles.update(pid, { modCount: (p.modCount ?? 0) + 1 });
			}
			targetInstanceId = pid;
			installedIds = new Set([...installedIds, `${pid}:${id}`]);
			const pn = profiles.list.find(p => p.id === pid)?.name;
			toast(pn ? `"${item.title}" instalado em "${pn}"!` : uiText("ui.8cb332040d3078ab", {arg0: (item.title)}), "success");
		} catch (e) {
			toast(uiText("ui.523804d745b0d9e4", {arg0: (item.title), arg1: (e instanceof Error ? e.message : String(e))}), "error");
		} finally { const n = new Set(installingIds); n.delete(id); installingIds = n; }
	}
</script>

<div class="catalog-page min-h-full flex flex-col select-none">

	{#if selectedItem}
		{@const id = `${selectedItem.source}:${selectedItem.sourceId}`}
		<ModDetailView
			item={selectedItem}
			details={modDetails}
			versions={modVersionsList}
			{loadingDetails}
			bind:activeTab={activeDetailTab}
			isInstalling={installingIds.has(id)}
			isInstalled={installedIds.has(`${targetInstanceId}:${id}`)}
			contentType={selectedType}
			onBack={closeDetails}
			onInstall={() => (selectedItem && isItemModpack(selectedItem)) ? openModpackInstall(selectedItem) : promptInstall(selectedItem!)}
			onOpenGuide={() => showInstallGuide = true}
			onInstallVersion={(verId) => promptInstall(selectedItem!, verId)}
		>
			{#if loadingDetails}
				<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-12 flex flex-col items-center justify-center gap-3">
					<Loader2 class="w-8 h-8 text-brand-500 animate-spin" />
					<p class="text-xs text-fg/35 font-medium">{uiText("ui.271713677d349e0f")}</p>
				</div>
			{:else if modDetails}
				{#if activeDetailTab === 'overview'}
					<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-6 shadow-sm overflow-hidden">
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="prose prose-invert max-w-none text-xs text-fg/80 leading-relaxed font-sans [&_a]:text-brand-400 [&_a]:underline [&_a]:underline-offset-2 hover:[&_a]:text-brand-300 [&_h1]:text-fg [&_h1]:font-black [&_h2]:text-fg [&_h2]:font-bold [&_h3]:text-fg [&_h3]:font-bold [&_img]:rounded-2xl [&_img]:max-w-full [&_img]:shadow-md [&_table]:w-full [&_table]:border-collapse [&_th]:border [&_th]:border-fg/10 [&_td]:border [&_td]:border-fg/10 [&_th]:p-2.5 [&_td]:p-2.5 [&_th]:bg-fg/5 [&_code]:bg-fg/10 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:rounded-md [&_code]:font-mono [&_pre]:bg-bg-subtle [&_pre]:p-4 [&_pre]:rounded-2xl [&_pre]:overflow-x-auto overflow-x-auto break-words select-text"
							onclick={(e) => {
								if (playVideoFromElement(e.target)) {
									e.preventDefault();
									e.stopPropagation();
									return;
								}
								const target = e.target as HTMLElement | null;
								const a = target?.closest('a');
								if (a && a.href && (a.href.startsWith('http://') || a.href.startsWith('https://') || a.href.startsWith('mailto:'))) {
									e.preventDefault();
									e.stopPropagation();
									openUrl(a.href).catch(() => {});
								}
							}}
							onkeydown={(e) => {
								if (e.key === "Enter" || e.key === " ") {
									if (playVideoFromElement(e.target)) {
										e.preventDefault();
										e.stopPropagation();
									}
								}
							}}
						>
							{@html processDescription(modDetails.body, modDetails.bodyType === 'html')}
						</div>
					</div>
				{:else if activeDetailTab === 'gallery'}
					{#if modDetails.gallery.length === 0}
						<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-12 text-center text-fg/35 text-xs">
							<p>{uiText("ui.eff657a1305cfb79")}</p>
						</div>
					{:else}
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
							{#each modDetails.gallery as img}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div class="group relative bg-bg-elevated border border-fg/[0.06] rounded-2xl overflow-hidden cursor-pointer hover:border-brand-500/30 transition-[color,background-color,border-color,box-shadow,transform,opacity] shadow-sm" onclick={() => lightboxImage = img}>
									<div class="h-44 w-full bg-bg-subtle overflow-hidden">
										<img decoding="async"
											src={img.url}
											alt={img.title || "Screenshot"}
											loading="lazy"
											class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
											onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
										/>
									</div>
									{#if img.title}
										<div class="p-3 bg-bg-elevated">
											<h4 class="text-xs font-bold text-fg truncate">{img.title}</h4>
											{#if img.description}
												<p class="text-[10px] text-fg/35 line-clamp-1 mt-0.5">{img.description}</p>
											{/if}
										</div>
									{/if}
								</div>
							{/each}
						</div>
					{/if}
                {:else if activeDetailTab === 'changelog'}
                    <ModChangelog projectId={selectedItem.sourceId} source={selectedItem.source} versions={modVersionsList} renderBody={body => processDescription(body, selectedItem!.source === 'curseforge')} />
				{:else if activeDetailTab === 'versions'}
					<ModVersionPicker
						versions={modVersionsList}
						contentType={selectedType}
						onInstall={(verId) => promptInstall(selectedItem!, verId)}
					/>
				{/if}
			{/if}
		</ModDetailView>

	{:else}
        <header use:pageMotion={'catalog-header'} class="catalog-hero" data-motion-surface>
            <div><p class="page-eyebrow">Luxmc / Discover</p><h1 class="page-title mt-2">{uiText('catalogDesign.title')}</h1></div>
            <button type="button" class="catalog-library" aria-label={uiText('catalogDesign.openLibrary')} onclick={() => { void goto('/instances'); }}><Package class="h-4 w-4" /><span>{uiText('catalogDesign.library')} · {profiles.list.length}</span><ArrowUpRight class="h-4 w-4" /></button>
        </header>
        <div class="catalog-layout">
        <div class="catalog-main flex flex-col min-w-0" class:catalog-compact={settings.value.catalogCompactCards === true}>
            <CatalogNavigation bind:selectedType />
            <div class="catalog-search-row">
            <div class="catalog-search relative w-full mb-4">
				<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/30" />
				<input type="search" aria-label={uiText("ui.659ee868fc163d0e")} bind:value={searchQuery} placeholder={uiText("ui.859c2b032a83e5c7", {arg0: (selectedType.toLowerCase())})}
					class="w-full bg-bg-elevated border border-border focus:border-brand-400/60 rounded-2xl py-4 pl-11 pr-4 text-sm text-fg placeholder-fg/30 focus:outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity] shadow-inner" />
			</div>
                <button type="button" class="catalog-filter-toggle" aria-expanded={filtersOpen} aria-controls="catalog-filter-panel" onclick={() => filtersOpen = !filtersOpen}><SlidersHorizontal class="h-4 w-4" />{uiText('ui.57ac13ce3d5bedff')}{#if activeFilters.length}<span>{activeFilters.length}</span>{/if}</button>
            </div>
			{#if activeFilters.length > 0 || searchQuery}
				<div class="mb-4 flex flex-wrap items-center gap-2" aria-label={uiText("ui.505d23a2b73a2c49")}>
					{#each activeFilters as filter}
						<button type="button" class={button({ variant: "secondary", size: "sm" })} aria-label={uiText("ui.1333e427c3efc526", {arg0: (filter.label)})} onclick={filter.clear}>{filter.label} ×</button>
					{/each}
					<button type="button" class={button({ variant: "ghost", size: "sm" })} onclick={resetFilters}>{uiText("ui.937f8ee080a9f405")}</button>
				</div>
			{/if}

			<div class="catalog-toolbar flex flex-wrap items-center justify-between gap-3 mb-5">
				<div class="flex items-center gap-2">
					<span class="text-xs font-extrabold text-fg">{selectedType}</span>
					<span class="text-xs text-fg/35 font-medium">({results.length} {uiText("ui.a30d8d35304a5476")}</span>
					{#if loading}
						<span class="text-[10px] text-brand-500 font-mono font-medium flex items-center gap-1 ml-2">
							<Loader2 class="w-3 h-3 animate-spin text-brand-500" /> {uiText("ui.5f0db2a236d0014f")}
						</span>
					{/if}
				</div>
				<div class="flex items-center gap-2.5">
					<div class="relative">
						<button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => sortMenuOpen = !sortMenuOpen}>
							<span>{currentSortLabel}</span>
							<ChevronDown class="w-3.5 h-3.5 text-fg/35 transition-transform duration-300 {sortMenuOpen ? 'rotate-180' : ''}" />
						</button>
						{#if sortMenuOpen}
							<button type="button" aria-label={uiText("ui.7ff31408d36adf67")} class="fixed inset-0 z-20 border-0 bg-transparent outline-none" onclick={() => sortMenuOpen = false}></button>
							<div class="absolute right-0 mt-1.5 w-52 bg-bg-elevated border border-fg/[0.06] rounded-xl shadow-2xl py-1 z-30 divide-y divide-white/5">
								{#each sortOptions as opt}
									<button type="button" class="w-full text-left px-3 py-2 text-xs transition-colors flex items-center justify-between cursor-pointer {selectedSort === opt.id ? 'bg-brand-500/20 text-brand-500 font-bold' : 'text-fg/70 hover:bg-fg/5 hover:text-fg'}" onclick={() => { selectedSort = opt.id; sortMenuOpen = false; }}>
										<span>{opt.label}</span>
										{#if selectedSort === opt.id}<Check class="w-3.5 h-3.5 text-brand-500" />{/if}
									</button>
								{/each}
							</div>
						{/if}
					</div>
					<div class="flex bg-bg-elevated border border-fg/[0.06] rounded-xl p-0.5 gap-0.5">
						<button type="button" class={button({ variant: viewMode === "grid" ? "primary" : "ghost", size: "icon" })} aria-pressed={viewMode === "grid"} onclick={() => viewMode = 'grid'} title={uiText("ui.839e20fb3599fc74")}><LayoutGrid class="w-3.5 h-3.5" /></button>
						<button type="button" class={button({ variant: viewMode === "list" ? "primary" : "ghost", size: "icon" })} aria-pressed={viewMode === "list"} onclick={() => viewMode = 'list'} title={uiText("ui.5e77e1785d574f9b")}><List class="w-3.5 h-3.5" /></button>
					</div>
				</div>
			</div>


			{#if searchError}
				<div class="flex items-center gap-3 bg-red-500/10 border border-red-500/20 rounded-2xl p-4 mb-4">
					<AlertTriangle class="w-5 h-5 text-red-400 shrink-0" />
					<div>
						<p class="text-xs font-bold text-red-400">{uiText("ui.d134bad3ef637359")}</p>
						<p class="text-[10px] text-red-300/60 mt-0.5">{searchError}</p>
					</div>
				</div>
			{/if}

			{#if loading && results.length === 0}
                <div role="status" aria-label={uiText('ui.268872b3efb06a70')} class="catalog-skeleton-grid">
                    {#each Array(6) as _}<div class="catalog-skeleton" aria-hidden="true"><div class="catalog-skeleton-art"></div><div class="p-5 space-y-3"><div class="h-5 w-3/4 rounded-lg bg-fg/10"></div><div class="h-3 w-full rounded bg-fg/5"></div><div class="h-3 w-2/3 rounded bg-fg/5"></div></div></div>{/each}
                </div>
            {:else if results.length === 0 && hasSearched}
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Search class="w-8 h-8 text-fg/20" />
						<p class="text-xs text-fg/35 font-medium">{uiText("ui.61c53e10c67742dc")}</p>
						<p class="text-[10px] text-fg/30">{uiText("ui.08061693739daef1")}</p>
						<button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={resetFilters}>{uiText("ui.937f8ee080a9f405")}</button>
					</div>
				</div>
			{:else}
                <div bind:clientWidth={resultsWidth} inert={loading} aria-busy={loading}>
                    <CatalogResults items={results} columns={viewMode === 'grid' ? resultColumns : 1} rowHeight={viewMode === 'grid' ? settings.value.catalogCompactCards ? 400 : 480 : resultsWidth < 650 ? 144 : 112}>
                        {#snippet children(item)}
                            {@const id = `${item.source}:${item.sourceId}`}
                            {#if viewMode === 'grid'}
                                <ModCard {item} isInstalling={installingIds.has(id)} isInstalled={installedIds.has(`${targetInstanceId}:${id}`)} contentType={selectedType} onOpenDetails={openDetails} onInstall={promptInstall} />
                            {:else}
                                <ModCardList {item} isInstalling={installingIds.has(id)} isInstalled={installedIds.has(`${targetInstanceId}:${id}`)} contentType={selectedType} onOpenDetails={openDetails} onInstall={promptInstall} />
                            {/if}
                        {/snippet}
                    </CatalogResults>
                </div>
			{/if}

			{#if showPager}
				<div class="flex items-center justify-between pt-3 pb-8 border-t border-fg/[0.06]">
					<div class="text-xs text-fg/35 font-medium">
						{#if results.length > 0}
							{uiText("ui.7c1679025ff67cf0")} <span class="text-fg font-bold">{(currentPage - 1) * pageSize + 1} - {(currentPage - 1) * pageSize + results.length}</span> {uiText("ui.959a45d44e6fcf58")} <span class="text-fg font-bold">{totalEstimate}</span>
						{:else}
							{uiText("ui.8f63f4d7594a823e")} <span class="text-fg font-bold">{currentPage}</span> {uiText("ui.9bddc2d54d195ce8")}
						{/if}
					</div>
					<div class="flex items-center gap-1 text-xs">
						<button type="button" aria-label={uiText("ui.9bcad734827a849a")} class={button({ variant: "secondary", size: "icon" })} onclick={() => goToPage(currentPage - 1)} disabled={currentPage <= 1 || loading}><ChevronLeft class="w-4 h-4" /></button>
						{#if pageNumbers[0] > 1}
							<button type="button" class={launcherButton({ variant: "secondary", size: "sm", class: "" })} onclick={() => goToPage(1)}>1</button>
							{#if pageNumbers[0] > 2}
								<span class="px-1 text-fg/35">...</span>
							{/if}
						{/if}
						{#each pageNumbers as p}
							<button type="button" class="h-8 w-8 rounded-xl font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {currentPage === p ? 'bg-bg-subtle text-fg border border-fg/[0.06] shadow-sm' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}" onclick={() => goToPage(p)}>{p}</button>
						{/each}
						{#if pageNumbers[pageNumbers.length - 1] < totalPages}
							{#if pageNumbers[pageNumbers.length - 1] < totalPages - 1}
								<span class="px-1 text-fg/35">...</span>
							{/if}
							<button type="button" class="h-8 w-8 rounded-xl font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {currentPage === totalPages ? 'bg-bg-subtle text-fg border border-fg/[0.06] shadow-sm' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}" onclick={() => goToPage(totalPages)}>{totalPages}</button>
						{/if}
						<button type="button" aria-label={uiText("ui.d537f9b442ea6fa2")} class={button({ variant: "secondary", size: "icon" })} onclick={() => goToPage(currentPage + 1)} disabled={currentPage >= totalPages || loading}><ChevronRight class="w-4 h-4" /></button>
					</div>
				</div>
			{/if}
		</div>
        <ModSearchBar bind:selectedSource bind:selectedVersion bind:selectedLoader bind:selectedCategory bind:expanded={filtersOpen} />
        </div>
	{/if}
</div>

{#if lightboxImage}
	<LightboxModal imageUrl={lightboxImage.url} title={lightboxImage.title} description={lightboxImage.description} onClose={() => lightboxImage = null} />
{/if}

{#if showInstallGuide}
	<InstallGuideModal onClose={() => showInstallGuide = false} />
{/if}

{#if showModpackInstallModal && modpackToInstall}
	<ModpackInstaller modpack={modpackToInstall} bind:instanceName={modpackInstanceName} bind:ramMb={modpackRamMb} isInstalling={false} queueMode={isInstallingModpack} progressText={modpackProgressText} progressPercent={modpackProgressPercent} onConfirm={confirmModpackInstall} onCancel={cancelModpackInstall} cancelling={cancelRequested} onClose={() => { showModpackInstallModal = false; }} />
{/if}

{#if showInstancePickerModal && itemToInstall}
	<InstancePickerModal item={itemToInstall.item} {selectedType} bind:chosenInstanceId bind:chosenWorldName onConfirm={confirmInstanceInstall} onClose={() => showInstancePickerModal = false} />
{/if}

<style>
    .catalog-layout { display:grid; grid-template-columns:minmax(0,1fr) 260px; align-items:start; gap:24px; }
    .catalog-compact :global(.catalog-cover) { height:156px; }
    .catalog-compact :global(.catalog-logo) { top:128px; height:48px; width:48px; }
    @media(max-width:1199px) { .catalog-layout { grid-template-columns:minmax(0,1fr); gap:16px; } }
    .catalog-hero { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 16px; margin-bottom: 22px; padding: 4px 0; }
    .catalog-hero :global(.page-title) { font-size: clamp(24px, 2.2vw, 32px); letter-spacing: -.045em; }
    .catalog-library { display: flex; align-items: center; gap: 9px; padding: 12px 14px; border: 1px solid rgb(var(--border)); border-radius: 14px; color: rgb(var(--fg-muted)); font-size: 11px; background: rgb(var(--bg-elevated) / .9); }
    .catalog-library:hover { color: rgb(var(--brand-action)); border-color: rgb(var(--brand-400) / .4); }
    .catalog-search-row { display: flex; align-items: stretch; gap: 10px; margin-bottom: 18px; }
    .catalog-search { flex: 1; min-width: 0; margin: 0; border-radius: 18px; }
    .catalog-search:focus-within { box-shadow: 0 0 0 4px rgb(var(--brand-500) / .07); }
    .catalog-filter-toggle { display: none; align-items: center; justify-content: center; gap: 9px; padding: 12px 18px; border: 1px solid rgb(var(--border)); border-radius: 16px; font-size: 12px; font-weight: 600; background: rgb(var(--bg-elevated) / .94); color: rgb(var(--fg-muted)); }
    .catalog-filter-toggle[aria-expanded='true'] { border-color: rgb(var(--brand-400) / .4); color: rgb(var(--brand-action)); background: rgb(var(--brand-500) / .08); }
    .catalog-filter-toggle span { padding: 1px 5px; border-radius: 6px; background: rgb(var(--brand-500) / .15); }
    .catalog-filter-toggle:focus-visible, .catalog-library:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 3px; }
    @media(max-width:1199px) { .catalog-filter-toggle { display:flex; } }
    .catalog-toolbar { padding: 0 2px; }
    .catalog-skeleton-grid { display: grid; grid-template-columns: repeat(2, minmax(0,1fr)); gap: 16px; }
    .catalog-skeleton { height: 464px; border: 1px solid rgb(var(--border)); border-radius: 24px; overflow: hidden; background: rgb(var(--bg-elevated)); }
    .catalog-skeleton-art { height: 224px; background: linear-gradient(120deg, rgb(var(--bg-subtle)), rgb(var(--brand-500) / .06), rgb(var(--bg-subtle))); }
    @media (prefers-reduced-motion: no-preference) { :global(html:not(.no-anim):not(.no-ui-motion)) .catalog-skeleton-art { animation: catalog-pulse 1.8s ease-in-out infinite alternate; } @keyframes catalog-pulse { from { opacity: .45; } to { opacity: 1; } } }
    @media (min-width: 1550px) { .catalog-skeleton-grid { grid-template-columns: repeat(3, minmax(0,1fr)); } }
    @media (max-width: 650px) { .catalog-skeleton-grid { grid-template-columns: minmax(0,1fr); } .catalog-filter-toggle { padding-inline: 12px; } }
</style>
