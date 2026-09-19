<script lang="ts">
	import { sanitizeHtml } from "$lib/utils/sanitizeHtml";
	import { onMount, untrack } from "svelte";
	import { deepLinks } from "$lib/stores/deepLinks.svelte";
	import type { DeepLinkAction } from "$lib/utils/deepLink";
	import {
		Search, LayoutGrid, List, Loader2, AlertTriangle,
		Check, ChevronLeft, ChevronRight, ChevronDown
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import {
		modsSearch, modsVersions, modsInstall, modsProjectDetails,
		modsDownloadToTemp, instanceImportMrpack, instanceImportModpack, instanceCancelImport,
		curseforgeStatus,
		type ModProjectDetails, type ModVersion, type ModSearchResultItem
	} from "$lib/api";
	import { listen } from "$lib/api/client";
	import { marked } from "marked";
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
	let modpackInstanceName = $state("");
	let modpackRamMb = $state(4096);
	let isInstallingModpack = $state(false);
    let cancelRequested = $state(false);
	let modpackProgressText = $state("");
	let modpackProgressPercent = $state(0);

	let showInstancePickerModal = $state(false);
	let itemToInstall = $state<{ item: ModSearchResultItem; versionId?: string } | null>(null);
	let chosenInstanceId = $state<string>(profiles.activeId ?? profiles.list[0]?.id ?? "");

	let curseforgeActive = $state(true);
	let searchQuery = $state("");
	let selectedSource = $state<"all" | "modrinth" | "curseforge">("all");
	let selectedType = $state("Modpack");
	let selectedLoader = $state<string | null>(null);
	let selectedCategory = $state<string | null>(null);
	let selectedVersion = $state("Qualquer Versão");
	let selectedSort = $state<"downloads" | "relevance" | "updated" | "newest">("downloads");
	let sortMenuOpen = $state(false);
	let viewMode = $state<"grid" | "list">("grid");

	const sortOptions = [
		{ id: "downloads", label: "Downloads" },
		{ id: "relevance", label: "Relevância" },
		{ id: "updated", label: "Atualizado recentemente" },
		{ id: "newest", label: "Mais recentes" }
	] as const;

	const currentSortLabel = $derived(sortOptions.find(s => s.id === selectedSort)?.label ?? "Downloads");

	let results = $state<Array<ModSearchResultItem>>([]);
	let loading = $state(false);
	let searchError = $state<string | null>(null);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let currentPage = $state(1);
	const pageSize = 21;
	let installingIds = $state<Set<string>>(new Set());
	let installedIds = $state<Set<string>>(new Set());
	let hasSearched = $state(false);

	const totalEstimateNumber = $derived(
		selectedType === "Modpack" ? 18318 : selectedType === "Mod" ? 54210 :
		selectedType === "Resource Pack" ? 12840 : selectedType === "Shader" ? 2450 :
		selectedType === "Data Pack" ? 8120 : 4300
	);
	const totalPages = $derived(Math.max(1, Math.ceil(totalEstimateNumber / pageSize)));
	const totalEstimate = $derived(
		selectedType === "Modpack" ? "18.318" : selectedType === "Mod" ? "54.210" :
		selectedType === "Resource Pack" ? "12.840" : selectedType === "Shader" ? "2.450" :
		selectedType === "Data Pack" ? "8.120" : "4.300"
	);

	let selectedItem = $state<ModSearchResultItem | null>(null);
	let modDetails = $state<ModProjectDetails | null>(null);
	let modVersionsList = $state<ModVersion[]>([]);
	let loadingDetails = $state(false);
	let activeDetailTab = $state<"overview" | "gallery" | "versions">("overview");
	let lightboxImage = $state<{ url: string; title?: string | null; description?: string | null } | null>(null);
	let showInstallGuide = $state(false);

	function renderMarkdown(content: string): string {
		if (!content) return "";
		try { return marked.parse(content) as string; } catch { return content; }
	}


	function processDescription(body: string, isHtml: boolean): string {
		if (!body) return "";
		let html = isHtml ? body : renderMarkdown(body);
		html = html.replace(
			/<iframe[^>]*src=["'](?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/embed\/|youtu\.be\/)([\w-]+)[^"']*["'][^>]*>.*?<\/iframe>/gi,
			(_m, vid) => `
				<div class="my-5 rounded-2xl overflow-hidden border border-fg/[0.06] bg-bg-overlay/60 shadow-xl max-w-2xl">
					<div class="relative aspect-video w-full group">
						<img src="https://img.youtube.com/vi/${vid}/hqdefault.jpg" class="w-full h-full object-cover opacity-85 group-hover:opacity-100 transition-opacity" alt="YouTube Preview" loading="lazy" />
						<a href="https://www.youtube.com/watch?v=${vid}" target="_blank" rel="noopener" class="absolute inset-0 flex items-center justify-center bg-bg-overlay/30 hover:bg-bg-overlay/10 transition-colors">
							<div class="w-16 h-12 rounded-2xl bg-red-600/90 text-fg flex items-center justify-center shadow-2xl hover:scale-110 hover:bg-red-600 transition-transform">
								<svg class="w-6 h-6 fill-current" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
							</div>
						</a>
					</div>
					<div class="p-3 bg-bg-elevated flex items-center justify-between text-xs text-fg/70 font-medium">
						<span class="flex items-center gap-1.5"><span class="w-2 h-2 rounded-full bg-red-500"></span> Vídeo de Demonstração (YouTube)</span>
						<a href="https://www.youtube.com/watch?v=${vid}" target="_blank" rel="noopener" class="text-brand-500 hover:underline font-bold">Assistir no Navegador ↗</a>
					</div>
				</div>`
		);
		html = html.replace(/<video([^>]*)>([\s\S]*?)<\/video>/gi,
			'<video$1 preload="metadata" controls playsinline class="rounded-xl max-w-full my-3 border border-fg/[0.06]">$2</video>');
		return sanitizeHtml(html);
	}

	async function doSearch(page = 1) {
		loading = true; searchError = null; hasSearched = true; currentPage = page;
		try {
			const ver = selectedVersion === "Qualquer Versão" ? "" : selectedVersion;
			results = await modsSearch(searchQuery.trim(), ver, pageSize, (page - 1) * pageSize,
				selectedType.toLowerCase().replace(/\s+/g, ""), selectedSort,
				selectedLoader ?? undefined, selectedCategory ?? undefined, selectedSource);
		} catch (e) {
			searchError = e instanceof Error ? e.message : String(e); results = [];
		} finally { loading = false; }
	}

	function goToPage(p: number) { if (p < 1 || loading) return; doSearch(p); }

	$effect(() => {
		searchQuery; selectedVersion; selectedType; selectedSource;
		selectedLoader; selectedCategory; selectedSort;
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => doSearch(1), 250);
		return () => { if (debounceTimer) clearTimeout(debounceTimer); };
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
            if (!targetInstanceId) throw new Error("Crie uma instância com versão e loader compatíveis com este mod.");
            itemToInstall = { item };
            chosenInstanceId = targetInstanceId;
            showInstancePickerModal = true;
        } catch (error) { toast(`Não foi possível carregar o projeto: ${String(error)}`, "error"); }
    }

	async function openDetails(item: ModSearchResultItem) {
		selectedItem = item; modDetails = null; modVersionsList = [];
		loadingDetails = true; activeDetailTab = "overview";
		try {
			const [d, v] = await Promise.all([
				modsProjectDetails(item.sourceId, item.source),
				modsVersions(item.sourceId, "", item.source).catch(() => [])
			]);
			modDetails = d; modVersionsList = v;
		} catch (e) {
			toast("Falha ao carregar detalhes completos: " + String(e), "error");
		} finally { loadingDetails = false; }
	}

	function closeDetails() { selectedItem = null; modDetails = null; modVersionsList = []; }

	function isItemModpack(item: ModSearchResultItem): boolean {
		if (selectedType === "Modpack") return true;
		const lowerTitle = item.title.toLowerCase();
		if (lowerTitle.includes("modpack") || lowerTitle.includes(" [pack]") || lowerTitle.includes(" pack ")) return true;
		if (item.categories?.some(c => c.toLowerCase().includes("modpack") || c.toLowerCase().includes("modpacks"))) return true;
		return false;
	}

	function promptInstall(item: ModSearchResultItem, versionId?: string) {
		if (isItemModpack(item)) { openModpackInstall(item); return; }
		if (profiles.list.length === 0) { toast("Crie uma instância primeiro para instalar mods!", "error"); return; }
		itemToInstall = { item, versionId };
		chosenInstanceId = targetInstanceId || profiles.activeId || profiles.list[0]?.id || "";
		showInstancePickerModal = true;
	}

	function openModpackInstall(item: ModSearchResultItem) {
		modpackToInstall = item; modpackInstanceName = item.title; modpackRamMb = 4096;
		modpackProgressText = ""; isInstallingModpack = false; showModpackInstallModal = true;
	}

	async function confirmModpackInstall() {
		if (!modpackToInstall || isInstallingModpack) return;
		const item = modpackToInstall; const name = modpackInstanceName.trim() || item.title;
		cancelRequested = false;
        isInstallingModpack = true; modpackProgressText = "Buscando arquivos do modpack...";
		modpackProgressPercent = 0;
		let unlisten: (() => void) | null = null;
		try {
			unlisten = await listen<{ phase: string; current: number; total: number; percent?: number; status: string }>("modpack-progress", (ev) => {
				modpackProgressText = ev.payload.status;
				modpackProgressPercent = ev.payload.percent ?? 0;
			});
			let versions = await modsVersions(item.sourceId, "", item.source);
			if (versions.length === 0) {
				const ver = selectedVersion === "Qualquer Versão" ? "" : selectedVersion;
				if (ver) {
					versions = await modsVersions(item.sourceId, ver, item.source);
				}
			}
			if (versions.length === 0) {
				toast(`Nenhuma versão encontrada para "${item.title}". Verifique a conexão com o ${item.source === "curseforge" ? "CurseForge" : "Modrinth"}.`, "error");
				isInstallingModpack = false;
				return;
			}
			const extension = item.source === "curseforge" ? ".zip" : ".mrpack";
            const f = versions.flatMap(version => version.files).find(file => file.url && file.filename.toLowerCase().endsWith(extension));
			if (!f?.url) { toast("Arquivo de download não disponível para este modpack.", "error"); isInstallingModpack = false; return; }
			modpackProgressText = `Baixando pacote (${f.filename})...`;
			modpackProgressPercent = -1;
			if (cancelRequested) throw new Error("Importação cancelada");
            const tempPath = await modsDownloadToTemp(f.url, f.filename);
            if (cancelRequested) throw new Error("Importação cancelada");
			modpackProgressText = "Configurando nova instância e extraindo mods...";
			modpackProgressPercent = 0;
			const iconUrl = item.iconUrl || "";
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
				mcVersion: cp.mcVersion,
				loader: (cp.loader || detectedLoader || "fabric") as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
				loaderVersion: cp.loaderVersion ?? undefined,
				gameDir: cp.gameDir,
				ramMb: modpackRamMb,
				createdAt: Date.now(),
				updatedAt: Date.now(),
			});
			profiles.activeId = cp.id;
			targetInstanceId = cp.id;
			await profiles.refresh();
			toast(`Instância "${name}" criada com sucesso!`, "success");
			showModpackInstallModal = false;
			modpackToInstall = null;
		} catch (e) {
			toast(cancelRequested ? "Importação cancelada." : `Erro ao criar instância: ${e instanceof Error ? e.message : String(e)}`, cancelRequested ? "info" : "error");
		} finally {
			if (unlisten) unlisten();
			isInstallingModpack = false; modpackProgressText = ""; modpackProgressPercent = 0;
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
			let vid = versionId;
			if (!vid) {
				const tv = prof?.mcVersion || (selectedVersion === "Qualquer Versão" ? "" : selectedVersion);
				let vs = await modsVersions(item.sourceId, tv, item.source);
				if (vs.length === 0) vs = await modsVersions(item.sourceId, "", item.source);
				if (vs.length === 0) { toast(`Nenhuma versão compatível para "${item.title}"`, "error"); return; }
				vid = vs[0].id;
			}
			const ct = selectedType.toLowerCase().replace(/\s+/g, "");
			await modsInstall({ profileId: pid, projectId: item.sourceId, versionId: vid, source: item.source, contentType: ct });
			if (pid && pid !== "default") {
				const p = profiles.list.find(p => p.id === pid);
				if (p && ct === "mod") profiles.update(pid, { modCount: (p.modCount ?? 0) + 1, loader: p.loader === "vanilla" ? "fabric" : p.loader });
			}
			installedIds = new Set([...installedIds, id]);
			const pn = profiles.list.find(p => p.id === pid)?.name;
			toast(pn ? `"${item.title}" instalado em "${pn}"!` : `"${item.title}" instalado com sucesso!`, "success");
		} catch (e) {
			toast(`Erro ao instalar "${item.title}": ${e instanceof Error ? e.message : String(e)}`, "error");
		} finally { const n = new Set(installingIds); n.delete(id); installingIds = n; }
	}
</script>

<div class="h-full flex gap-6 select-none overflow-hidden">

	{#if selectedItem}
		{@const id = `${selectedItem.source}:${selectedItem.sourceId}`}
		<ModDetailView
			item={selectedItem}
			details={modDetails}
			versions={modVersionsList}
			{loadingDetails}
			bind:activeTab={activeDetailTab}
			isInstalling={installingIds.has(id)}
			isInstalled={installedIds.has(id)}
			contentType={selectedType}
			onBack={closeDetails}
			onInstall={() => (selectedItem && isItemModpack(selectedItem)) ? openModpackInstall(selectedItem) : promptInstall(selectedItem!)}
			onOpenGuide={() => showInstallGuide = true}
			onInstallVersion={(verId) => (selectedItem && isItemModpack(selectedItem)) ? openModpackInstall(selectedItem) : promptInstall(selectedItem!, verId)}
		>
			{#if loadingDetails}
				<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-12 flex flex-col items-center justify-center gap-3">
					<Loader2 class="w-8 h-8 text-brand-500 animate-spin" />
					<p class="text-xs text-fg/35 font-medium">Carregando informações completas...</p>
				</div>
			{:else if modDetails}
				{#if activeDetailTab === 'overview'}
					<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-6 shadow-sm overflow-hidden">
						<div class="prose prose-invert max-w-none text-xs text-fg/80 leading-relaxed font-sans [&_a]:text-brand-500 [&_h1]:text-fg [&_h2]:text-fg [&_h3]:text-fg [&_img]:rounded-xl [&_img]:max-w-full">
							{@html processDescription(modDetails.body, modDetails.bodyType === 'html')}
						</div>
					</div>
				{:else if activeDetailTab === 'gallery'}
					{#if modDetails.gallery.length === 0}
						<div class="bg-bg-elevated border border-fg/[0.06] rounded-3xl p-12 text-center text-fg/35 text-xs">
							<p>Nenhuma screenshot disponível para este projeto.</p>
						</div>
					{:else}
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
							{#each modDetails.gallery as img}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div class="group relative bg-bg-elevated border border-fg/[0.06] rounded-2xl overflow-hidden cursor-pointer hover:border-brand-500/30 transition-all shadow-sm" onclick={() => lightboxImage = img}>
									<div class="h-44 w-full bg-bg-subtle overflow-hidden">
										<img src={img.url} alt={img.title || "Screenshot"} class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
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
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-1">
			<header class="relative mb-6 overflow-hidden rounded-2xl border border-fg/[0.08] bg-bg-elevated/80 p-6 backdrop-blur-2xl">
                <div class="pointer-events-none absolute -right-12 -top-16 h-48 w-48 rounded-full bg-brand-500/10 blur-3xl"></div>
                <p class="page-eyebrow mb-3">Descubra · Instale · Explore</p>
                <h1 class="page-title">Seu Minecraft, sem limites.</h1>
                <p class="page-description">Mods, modpacks e novos mundos. Tudo em um só lugar.</p>
                <div class="mt-5 flex items-center gap-3 text-[10px] font-semibold text-fg-muted"><span class="rounded-full border border-success/20 bg-success/5 px-3 py-1.5">Modrinth</span><span class="rounded-full border border-orange-500/20 bg-orange-500/5 px-3 py-1.5">CurseForge</span></div>
            </header>

			<div class="relative w-full mb-4">
				<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/30" />
				<input type="text" bind:value={searchQuery} placeholder={`Buscar ${selectedType.toLowerCase()}…`}
					class="w-full bg-bg-elevated border border-fg/[0.08] focus:border-brand-500/60 rounded-2xl py-3 pl-11 pr-4 text-xs text-fg placeholder-fg/30 focus:outline-none transition-all shadow-inner" />
			</div>

			<div class="flex items-center justify-between mb-4">
				<div class="flex items-center gap-2">
					<span class="text-xs font-extrabold text-fg">{selectedType}</span>
					<span class="text-xs text-fg/35 font-medium">({results.length} nesta página)</span>
					{#if loading}
						<span class="text-[10px] text-brand-500 font-mono font-medium flex items-center gap-1 ml-2">
							<Loader2 class="w-3 h-3 animate-spin text-brand-500" /> buscando...
						</span>
					{/if}
				</div>
				<div class="flex items-center gap-2.5">
					<div class="relative">
						<button type="button" class="bg-bg-elevated border border-fg/[0.06] hover:border-fg/[0.15] px-3 py-1.5 rounded-xl text-xs font-semibold text-fg/80 flex items-center gap-2 transition-all cursor-pointer shadow-sm" onclick={() => sortMenuOpen = !sortMenuOpen}>
							<span>{currentSortLabel}</span>
							<ChevronDown class="w-3.5 h-3.5 text-fg/35 transition-transform duration-300 {sortMenuOpen ? 'rotate-180' : ''}" />
						</button>
						{#if sortMenuOpen}
							<button type="button" aria-label="Fechar menu" class="fixed inset-0 z-20 cursor-default bg-transparent border-none p-0 outline-none" onclick={() => sortMenuOpen = false}></button>
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
						<button type="button" class="p-1.5 rounded-lg transition-all cursor-pointer {viewMode === 'grid' ? 'bg-brand-500 text-fg shadow-sm' : 'text-fg/35 hover:text-fg'}" onclick={() => viewMode = 'grid'} title="Grade"><LayoutGrid class="w-3.5 h-3.5" /></button>
						<button type="button" class="p-1.5 rounded-lg transition-all cursor-pointer {viewMode === 'list' ? 'bg-brand-500 text-fg shadow-sm' : 'text-fg/35 hover:text-fg'}" onclick={() => viewMode = 'list'} title="Lista"><List class="w-3.5 h-3.5" /></button>
					</div>
				</div>
			</div>


			{#if searchError}
				<div class="flex items-center gap-3 bg-red-500/10 border border-red-500/20 rounded-2xl p-4 mb-4">
					<AlertTriangle class="w-5 h-5 text-red-400 shrink-0" />
					<div>
						<p class="text-xs font-bold text-red-400">Erro na busca</p>
						<p class="text-[10px] text-red-300/60 mt-0.5">{searchError}</p>
					</div>
				</div>
			{/if}

			{#if loading && results.length === 0}
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Loader2 class="w-8 h-8 text-brand-500 animate-spin" />
						<p class="text-xs text-fg/35 font-medium">Buscando em Modrinth e CurseForge...</p>
					</div>
				</div>
			{:else if results.length === 0 && hasSearched}
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Search class="w-8 h-8 text-fg/20" />
						<p class="text-xs text-fg/35 font-medium">Nenhum resultado encontrado</p>
						<p class="text-[10px] text-fg/30">Tente outro termo ou altere os filtros</p>
					</div>
				</div>
			{:else}
				{#if viewMode === 'grid'}
					<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 pb-6">
						{#each results as item}
							{@const id = `${item.source}:${item.sourceId}`}
							<ModCard {item} isInstalling={installingIds.has(id)} isInstalled={installedIds.has(id)} contentType={selectedType} onOpenDetails={openDetails} onInstall={promptInstall} />
						{/each}
					</div>
				{:else}
					<div class="space-y-2.5 pb-6">
						{#each results as item}
							{@const id = `${item.source}:${item.sourceId}`}
							<ModCardList {item} isInstalling={installingIds.has(id)} isInstalled={installedIds.has(id)} contentType={selectedType} onOpenDetails={openDetails} onInstall={promptInstall} />
						{/each}
					</div>
				{/if}

				{#if results.length > 0}
					<div class="flex items-center justify-between pt-3 pb-8 border-t border-fg/[0.06]">
						<div class="text-xs text-fg/35 font-medium">
							Mostrando <span class="text-fg font-bold">{(currentPage - 1) * pageSize + 1} - {Math.min(currentPage * pageSize, totalEstimateNumber)}</span> de <span class="text-fg font-bold">{totalEstimate}</span>
						</div>
						<div class="flex items-center gap-1 text-xs">
							<button type="button" class="h-8 w-8 rounded-xl flex items-center justify-center text-fg/35 hover:text-fg hover:bg-fg/5 disabled:opacity-30 disabled:pointer-events-none transition-all cursor-pointer" onclick={() => goToPage(currentPage - 1)} disabled={currentPage <= 1 || loading}><ChevronLeft class="w-4 h-4" /></button>
							{#each [1, 2, 3, 4] as p}
								{#if totalPages >= p}
									<button type="button" class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === p ? 'bg-bg-subtle text-fg border border-fg/[0.06] shadow-sm' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}" onclick={() => goToPage(p)}>{p}</button>
								{/if}
							{/each}
							{#if totalPages > 5}
								<span class="px-1 text-fg/35">...</span>
								<button type="button" class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === totalPages ? 'bg-bg-subtle text-fg border border-fg/[0.06] shadow-sm' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}" onclick={() => goToPage(totalPages)}>{totalPages}</button>
							{/if}
							<button type="button" class="h-8 w-8 rounded-xl flex items-center justify-center text-fg/35 hover:text-fg hover:bg-fg/5 disabled:opacity-30 disabled:pointer-events-none transition-all cursor-pointer" onclick={() => goToPage(currentPage + 1)} disabled={currentPage >= totalPages || loading}><ChevronRight class="w-4 h-4" /></button>
						</div>
					</div>
				{/if}
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
	<InstancePickerModal item={itemToInstall.item} {selectedType} bind:chosenInstanceId onConfirm={confirmInstanceInstall} onClose={() => showInstancePickerModal = false} />
{/if}
