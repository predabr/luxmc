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
		Flame, Globe
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { catalogCompatibility } from "$lib/utils/catalogCompatibility";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { fireModpackSuccessConfetti } from "$lib/utils/confetti";
	import {
		modsSearch, modsVersions, modsInstall, modsProjectDetails,
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
	import ModVersionPicker from "$lib/components/mods/ModVersionPicker.svelte";
	import ModpackInstaller from "$lib/components/mods/ModpackInstaller.svelte";
	import InstancePickerModal from "$lib/components/mods/InstancePickerModal.svelte";
	import InstallGuideModal from "$lib/components/mods/InstallGuideModal.svelte";
	import LightboxModal from "$lib/components/mods/LightboxModal.svelte";

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
	let isInstallingModpack = $state(false);
    let cancelRequested = $state(false);
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
	const pageSize = 36;
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
	let activeDetailTab = $state<"overview" | "gallery" | "versions">("overview");
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
		selectedLoader; selectedCategory; selectedSort;
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
        if (!request || isInstallingModpack || showInstancePickerModal || showModpackInstallModal) return;
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
			loadingDetails = false; activeDetailTab = "overview"; return;
		}
		selectedItem = item; modDetails = null; modVersionsList = [];
		loadingDetails = true; activeDetailTab = "overview";
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
		modpackProgressText = ""; isInstallingModpack = false; showModpackInstallModal = true;
	}

	async function confirmModpackInstall() {
		if (!modpackToInstall || isInstallingModpack) return;
		const item = modpackToInstall; const name = modpackInstanceName.trim() || item.title;
		cancelRequested = false;
        isInstallingModpack = true; modpackProgressText = uiText("ui.5a4fa7bb57930284");
		modpackProgressPercent = 0;
		let unlisten: (() => void) | null = null;
		try {
			unlisten = await listen<{ phase: string; current: number; total: number; percent?: number; status: string }>("modpack-progress", (ev) => {
				modpackProgressText = ev.payload.status;
				modpackProgressPercent = ev.payload.percent ?? 0;
			});
			const ver = selectedVersion === "Qualquer Versão" || modpackVersionId ? "" : selectedVersion;
			let versions = await modsVersions(item.sourceId, ver, item.source);
			if (modpackVersionId) versions = versions.filter(version => version.id === modpackVersionId);
			if (versions.length === 0) {
				toast(uiText("ui.c38446c7df13fc52", {arg0: (item.title), arg1: (item.source === "curseforge" ? "CurseForge" : "Modrinth")}), "error");
				isInstallingModpack = false;
				return;
			}
			const extension = item.source === "curseforge" ? ".zip" : ".mrpack";
            const f = versions.flatMap(version => version.files).find(file => file.url && file.filename.toLowerCase().endsWith(extension))
				|| versions.flatMap(version => version.files).find(file => file.url && (file.filename.toLowerCase().endsWith(".zip") || file.filename.toLowerCase().endsWith(".mrpack")));
			if (!f?.url) { toast(uiText("ui.ebe5f9d75823b7c4"), "error"); isInstallingModpack = false; return; }
			modpackProgressText = uiText("ui.18333ba6ce43048e", {arg0: (f.filename)});
			modpackProgressPercent = -1;
			if (cancelRequested) throw new Error(uiText("ui.bb9811b14ec4fb48"));
            const tempPath = await modsDownloadToTemp(f.url, f.filename);
            if (cancelRequested) throw new Error(uiText("ui.bb9811b14ec4fb48"));
			modpackProgressText = uiText("ui.cbc82784c96e08ef");
			modpackProgressPercent = 0;
			const iconUrl = item.iconUrl || item.bannerUrl || "";
			const detectedLoader = item.categories?.find(c => ["forge", "fabric", "neoforge", "quilt"].includes(c.toLowerCase()))?.toLowerCase()
				|| (item.title.toLowerCase().includes("forge") && !item.title.toLowerCase().includes("neoforge") ? "forge" : "")
				|| (item.title.toLowerCase().includes("neoforge") ? "neoforge" : "")
				|| (item.title.toLowerCase().includes("fabric") ? "fabric" : "");
			const cp = item.source === "curseforge"
				? await instanceImportModpack(tempPath, name, item.versions[0] || "1.20.1", detectedLoader, iconUrl, modpackRamMb)
				: await instanceImportMrpack(tempPath, name, iconUrl, modpackRamMb);
			profiles.add({
				id: cp.id,
				name: cp.name,
				icon: iconUrl || "default",
				banner: item.bannerUrl || undefined,
				mcVersion: cp.mcVersion,
				loader: (cp.loader || detectedLoader || "fabric") as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: cp.loaderVersion ?? undefined,
				gameDir: cp.gameDir,
				ramMb: modpackRamMb,
				createdAt: Date.now(),
				updatedAt: Date.now(),
			});
			if (item.bannerUrl) {
				profiles.setBanner(cp.id, item.bannerUrl);
			}
			profiles.activeId = cp.id;
			targetInstanceId = cp.id;
			await profiles.refresh();
			fireModpackSuccessConfetti();
			toast(uiText("ui.1b6566167e91c704", {arg0: (name)}), "success");
			showModpackInstallModal = false;
			modpackToInstall = null;
		} catch (e) {
			const cancelled = cancelRequested || String(e).toLowerCase().includes("cancelad");
			toast(cancelled ? uiText("ui.e6c50b7bfe042bcf") : uiText("ui.66f69086d7d99519", {arg0: (e instanceof Error ? e.message : String(e))}), cancelled ? "info" : "error");
		} finally {
			if (unlisten) unlisten();
			isInstallingModpack = false; cancelRequested = false; modpackProgressText = ""; modpackProgressPercent = 0;
		}
	}

    async function cancelModpackInstall(): Promise<void> {
        if (cancelRequested) return;
        cancelRequested = true;
        try { await instanceCancelImport(); } catch (error) { cancelRequested = false; toast(String(error), "error"); }
    }

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
			await modsInstall({ profileId: pid, projectId: item.sourceId, versionId: vid, source: item.source, contentType: ct, worldName: ct === "datapack" ? chosenWorldName : undefined });
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

<div class="min-h-full flex flex-col xl:flex-row gap-6 select-none">

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
		<div class="flex-1 flex flex-col min-w-0 pr-1">
			<header class="relative mb-6 overflow-hidden rounded-2xl border border-fg/[0.08] bg-bg-elevated p-6 shadow-soft">
                <p class="page-eyebrow mb-3">{uiText("ui.63f0b3839bb0e4a4")}</p>
                <h1 class="page-title">{uiText("ui.ec71124c5fcbe0e7")}</h1>
                <p class="page-description">{uiText("ui.50241729d3a6fcdf")}</p>
                <div class="mt-5 flex flex-wrap items-center gap-2.5">
                    <button
                        type="button"
                        class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center gap-2 cursor-pointer shadow-sm active:scale-95 {selectedSource === 'all' ? 'bg-fg/15 text-fg border border-fg/20 shadow-md' : 'bg-fg/5 text-fg/50 hover:text-fg hover:bg-fg/10 border border-transparent'}"
                        onclick={() => selectedSource = "all"}
                    >
                        <Globe class="w-3.5 h-3.5" />
                        <span>{uiText("ui.8a0108a42be96e26")}</span>
                    </button>

                    <button
                        type="button"
                        class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center gap-2 cursor-pointer active:scale-95 {selectedSource === 'modrinth' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/50 shadow-md shadow-emerald-500/20 ring-1 ring-emerald-500/40' : 'bg-emerald-500/10 text-emerald-400/80 hover:text-emerald-300 hover:bg-emerald-500/15 border border-emerald-500/20'}"
                        onclick={() => selectedSource = selectedSource === "modrinth" ? "all" : "modrinth"}
                    >
                        <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                        <svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24">
                            <path d="M12.252 0C5.485 0 0 5.485 0 12.252c0 3.398 1.385 6.474 3.633 8.692l.006-.006c.24.234.492.456.756.666l.004-.004A12.19 12.19 0 0 0 12.252 24c6.767 0 12.252-5.485 12.252-12.252C24.504 5.485 19.019 0 12.252 0zm.012 3.864c3.418 0 6.36 2.072 7.625 5.034l-2.73 1.576a5.534 5.534 0 0 0-4.895-3.418V3.864zm-5.184 2.99a8.384 8.384 0 0 1 4.184-1.898v3.192a5.538 5.538 0 0 0-2.825 2.196l-2.73-1.576a8.386 8.386 0 0 1 1.371-1.914zm-3.216 5.398c0-.68.083-1.34.238-1.973l2.73 1.576c-.053.259-.082.528-.082.803 0 1.25.42 2.404 1.127 3.332l-2.73 1.576a8.388 8.388 0 0 1-1.283-5.314zm14.47 5.762-2.73-1.576a5.538 5.538 0 0 0 .584-3.377l2.73-1.576a8.388 8.388 0 0 1-.584 6.529zm-5.07 2.122v-3.192a5.538 5.538 0 0 0 3.256-1.547l2.73 1.576a8.388 8.388 0 0 1-5.986 3.163zm-4.896-1.128 2.73-1.576a5.538 5.538 0 0 0 3.166 1.71v3.192a8.388 8.388 0 0 1-5.896-3.326z"/>
                        </svg>
                        <span>Modrinth</span>
                        {#if selectedSource === "modrinth"}
                            <Check class="w-3 h-3 text-emerald-400 stroke-[3]" />
                        {/if}
                    </button>

                    <button
                        type="button"
                        class="px-3.5 py-1.5 rounded-xl text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] flex items-center gap-2 cursor-pointer active:scale-95 {selectedSource === 'curseforge' ? 'bg-orange-500/20 text-orange-300 border border-orange-500/50 shadow-md shadow-orange-500/20 ring-1 ring-orange-500/40' : 'bg-orange-500/10 text-orange-400/80 hover:text-orange-300 hover:bg-orange-500/15 border border-orange-500/20'}"
                        onclick={() => selectedSource = selectedSource === "curseforge" ? "all" : "curseforge"}
                    >
                        <Flame class="w-3.5 h-3.5 text-orange-400" />
                        <span>CurseForge</span>
                        {#if selectedSource === "curseforge"}
                            <Check class="w-3 h-3 text-orange-400 stroke-[3]" />
                        {/if}
                    </button>
                </div>
            </header>

			<div class="relative w-full mb-4">
				<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/30" />
				<input type="search" aria-label={uiText("ui.659ee868fc163d0e")} bind:value={searchQuery} placeholder={uiText("ui.859c2b032a83e5c7", {arg0: (selectedType.toLowerCase())})}
					class="w-full bg-bg-elevated border border-fg/[0.08] focus:border-brand-500/60 rounded-2xl py-3 pl-11 pr-4 text-xs text-fg placeholder-fg/30 focus:outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity] shadow-inner" />
			</div>
			{#if activeFilters.length > 0 || searchQuery}
				<div class="mb-4 flex flex-wrap items-center gap-2" aria-label={uiText("ui.505d23a2b73a2c49")}>
					{#each activeFilters as filter}
						<button type="button" class={button({ variant: "secondary", size: "sm" })} aria-label={uiText("ui.1333e427c3efc526", {arg0: (filter.label)})} onclick={filter.clear}>{filter.label} ×</button>
					{/each}
					<button type="button" class={button({ variant: "ghost", size: "sm" })} onclick={resetFilters}>{uiText("ui.937f8ee080a9f405")}</button>
				</div>
			{/if}

			<div class="flex items-center justify-between mb-4">
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
							<button type="button" aria-label={uiText("ui.7ff31408d36adf67")} class={launcherButton({ variant: "secondary", size: "icon", class: "fixed inset-0 z-20 outline-none" })} onclick={() => sortMenuOpen = false}></button>
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
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Loader2 class="w-8 h-8 text-brand-500 animate-spin" />
						<p class="text-xs text-fg/35 font-medium">{uiText("ui.268872b3efb06a70")}</p>
					</div>
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
                    <CatalogResults items={results} columns={viewMode === 'grid' ? resultColumns : 1} rowHeight={viewMode === 'grid' ? 440 : resultsWidth < 650 ? 144 : 112}>
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

		<ModSearchBar bind:selectedSource bind:selectedType bind:selectedVersion bind:selectedLoader bind:selectedCategory />
	{/if}
</div>

{#if lightboxImage}
	<LightboxModal imageUrl={lightboxImage.url} title={lightboxImage.title} description={lightboxImage.description} onClose={() => lightboxImage = null} />
{/if}

{#if showInstallGuide}
	<InstallGuideModal onClose={() => showInstallGuide = false} />
{/if}

{#if showModpackInstallModal && modpackToInstall}
	<ModpackInstaller modpack={modpackToInstall} bind:instanceName={modpackInstanceName} bind:ramMb={modpackRamMb} isInstalling={isInstallingModpack} progressText={modpackProgressText} progressPercent={modpackProgressPercent} onConfirm={confirmModpackInstall} onCancel={cancelModpackInstall} cancelling={cancelRequested} onClose={() => { if (!isInstallingModpack) showModpackInstallModal = false; }} />
{/if}

{#if showInstancePickerModal && itemToInstall}
	<InstancePickerModal item={itemToInstall.item} {selectedType} bind:chosenInstanceId bind:chosenWorldName onConfirm={confirmInstanceInstall} onClose={() => showInstancePickerModal = false} />
{/if}
