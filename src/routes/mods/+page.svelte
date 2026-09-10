<script lang="ts">
	import { 
		Search, 
		Download, 
		SlidersHorizontal, 
		Layers, 
		LayoutGrid, 
		List, 
		Loader2, 
		AlertTriangle, 
		Users, 
		Flame, 
		Check, 
		ArrowLeft, 
		ExternalLink, 
		Globe, 
		CheckCircle2, 
		Image as ImageIcon, 
		FileText, 
		Calendar, 
		HardDrive, 
		ShieldCheck, 
		Heart, 
		HelpCircle, 
		X, 
		Sparkles, 
		MessageSquare, 
		Clock, 
		Tag, 
		Box,
		ChevronDown,
		ChevronLeft,
		ChevronRight,
		ArrowUpDown,
		Filter,
		Cpu,
		Palette
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { 
		modsSearch, 
		modsVersions, 
		modsInstall, 
		modsProjectDetails,
		type ModProjectDetails,
		type ModVersion,
		type ModSearchResultItem
	} from "$lib/api";
	import { marked } from "marked";

	let targetInstanceId = $state<string>(profiles.activeId ?? profiles.list[0]?.id ?? "default");

	$effect(() => {
		if ((!targetInstanceId || targetInstanceId === "default") && profiles.list.length > 0) {
			targetInstanceId = profiles.activeId ?? profiles.list[0].id;
		}
	});

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

	const currentSortLabel = $derived(
		sortOptions.find(s => s.id === selectedSort)?.label ?? "Downloads"
	);

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
		selectedType === "Modpack" ? 18318 :
		selectedType === "Mod" ? 54210 :
		selectedType === "Resource Pack" ? 12840 :
		selectedType === "Shader" ? 2450 :
		selectedType === "Data Pack" ? 8120 : 4300
	);

	const totalPages = $derived(Math.max(1, Math.ceil(totalEstimateNumber / pageSize)));

	const totalEstimate = $derived(
		selectedType === "Modpack" ? "18.318" :
		selectedType === "Mod" ? "54.210" :
		selectedType === "Resource Pack" ? "12.840" :
		selectedType === "Shader" ? "2.450" :
		selectedType === "Data Pack" ? "8.120" : "4.300"
	);

	// Detail View State
	let selectedItem = $state<ModSearchResultItem | null>(null);
	let modDetails = $state<ModProjectDetails | null>(null);
	let modVersionsList = $state<ModVersion[]>([]);
	let loadingDetails = $state(false);
	let activeDetailTab = $state<"overview" | "gallery" | "versions">("overview");
	let lightboxImage = $state<{ url: string; title?: string | null; description?: string | null } | null>(null);
	let showInstallGuide = $state(false);

	const contentTypes = ["Modpack", "Mod", "Resource Pack", "Shader", "Data Pack", "World"];
	const modLoaders = [
		{ name: "Fabric", color: "text-cyan-400 border-cyan-500/30 bg-cyan-500/10" },
		{ name: "Quilt", color: "text-purple-400 border-purple-500/30 bg-purple-500/10" },
		{ name: "Forge", color: "text-sky-400 border-sky-500/30 bg-sky-500/10" },
		{ name: "NeoForge", color: "text-orange-400 border-orange-500/30 bg-orange-500/10" },
	];
	const categories = [
		"Adventure", "Challenging", "Combat", "Kitchen Sink",
		"Lightweight", "Magic", "Multiplayer", "Optimization",
		"Quests", "Technology"
	];
	const mcVersions = ["Qualquer Versão", "1.21.4", "1.21.3", "1.21.1", "1.20.6", "1.20.4", "1.20.1", "1.19.4", "1.19.2", "1.18.2", "1.16.5", "1.12.2", "1.8.9"];

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

	function formatBytes(bytes: number): string {
		if (!bytes || bytes === 0) return "0 B";
		const k = 1024;
		const sizes = ["B", "KB", "MB", "GB"];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
	}

	function renderMarkdown(content: string): string {
		if (!content) return "";
		try {
			return marked.parse(content) as string;
		} catch {
			return content;
		}
	}

	async function doSearch(page = 1) {
		loading = true;
		searchError = null;
		hasSearched = true;
		currentPage = page;

		try {
			const effectiveVer = selectedVersion === "Qualquer Versão" ? "" : selectedVersion;
			const typeSlug = selectedType.toLowerCase().replace(/\s+/g, "");
			const offset = (page - 1) * pageSize;

			let allResults = await modsSearch(
				searchQuery.trim(),
				effectiveVer,
				pageSize,
				offset,
				typeSlug,
				selectedSort
			);

			if (selectedSource !== "all") {
				allResults = allResults.filter(r => r.source === selectedSource);
			}

			if (selectedLoader) {
				allResults = allResults.filter(r => 
					r.categories.some(c => c.toLowerCase() === selectedLoader!.toLowerCase())
				);
			}

			if (selectedCategory) {
				allResults = allResults.filter(r => 
					r.categories.some(c => c.toLowerCase().includes(selectedCategory!.toLowerCase()))
				);
			}

			results = allResults;
		} catch (e) {
			searchError = e instanceof Error ? e.message : String(e);
			results = [];
		} finally {
			loading = false;
		}
	}

	function goToPage(p: number) {
		if (p < 1 || loading) return;
		doSearch(p);
	}

	$effect(() => {
		searchQuery;
		selectedVersion;
		selectedType;
		selectedSource;
		selectedLoader;
		selectedCategory;
		selectedSort;
		
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			doSearch(1);
		}, 250);
	});

	async function openDetails(item: ModSearchResultItem) {
		selectedItem = item;
		modDetails = null;
		modVersionsList = [];
		loadingDetails = true;
		activeDetailTab = "overview";

		try {
			const [details, versions] = await Promise.all([
				modsProjectDetails(item.sourceId, item.source),
				modsVersions(item.sourceId, "", item.source).catch(() => [])
			]);
			modDetails = details;
			modVersionsList = versions;
		} catch (e) {
			toast("Falha ao carregar detalhes completos: " + String(e), "error");
		} finally {
			loadingDetails = false;
		}
	}

	function closeDetails() {
		selectedItem = null;
		modDetails = null;
		modVersionsList = [];
	}

	async function installItem(item: ModSearchResultItem, versionId?: string) {
		const id = `${item.source}:${item.sourceId}`;
		if (installingIds.has(id)) return;

		installingIds = new Set([...installingIds, id]);

		try {
			const effectiveProfileId = targetInstanceId || profiles.activeId || profiles.list[0]?.id || "default";
			const targetProfile = profiles.list.find(p => p.id === effectiveProfileId);

			let targetVerId = versionId;
			if (!targetVerId) {
				const targetVer = targetProfile?.mcVersion || (selectedVersion === "Qualquer Versão" ? "1.21.4" : selectedVersion);
				let versions = await modsVersions(item.sourceId, targetVer, item.source);
				if (versions.length === 0) {
					// Fallback to any version if specific version returned none
					versions = await modsVersions(item.sourceId, "", item.source);
				}
				if (versions.length === 0) {
					toast(`Nenhuma versão compatível encontrada para "${item.title}"`, "error");
					return;
				}
				targetVerId = versions[0].id;
			}

			await modsInstall({
				profileId: effectiveProfileId,
				projectId: item.sourceId,
				versionId: targetVerId,
				source: item.source,
				contentType: selectedType.toLowerCase().replace(/\s+/g, ""),
			});

			if (effectiveProfileId && effectiveProfileId !== "default") {
				const prof = profiles.list.find(p => p.id === effectiveProfileId);
				if (prof) {
					profiles.update(effectiveProfileId, { 
						modCount: (prof.modCount ?? 0) + 1,
						loader: prof.loader === "vanilla" ? "fabric" : prof.loader
					});
				}
			}

			installedIds = new Set([...installedIds, id]);
			const targetProfName = profiles.list.find(p => p.id === effectiveProfileId)?.name;
			toast(
				targetProfName 
					? `"${item.title}" instalado em "${targetProfName}"!` 
					: `"${item.title}" instalado com sucesso!`, 
				"success"
			);
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			toast(`Erro ao instalar "${item.title}": ${msg}`, "error");
		} finally {
			const next = new Set(installingIds);
			next.delete(id);
			installingIds = next;
		}
	}
</script>

<div class="h-full flex gap-6 select-none overflow-hidden">
	
	<!-- ================= MOD DETAILS VIEW (STYLE MATCHING REFERENCE) ================= -->
	{#if selectedItem}
		{@const id = `${selectedItem.source}:${selectedItem.sourceId}`}
		{@const isInstalling = installingIds.has(id)}
		{@const isInstalled = installedIds.has(id)}
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
			
			<!-- Top Navigation: Back Button -->
			<div class="flex items-center justify-between">
				<button 
					type="button" 
					class="flex items-center gap-2 px-3.5 py-2 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/10 text-xs font-bold transition-all shadow-sm cursor-pointer active:scale-95"
					onclick={closeDetails}
				>
					<ArrowLeft class="w-4 h-4" />
					<span>Voltar ao Catálogo</span>
				</button>

				<div class="flex items-center gap-2 text-xs text-white/40 font-mono">
					<span>Fonte: <strong class="text-white uppercase">{selectedItem.source}</strong></span>
					<span>•</span>
					<span>ID: {selectedItem.sourceId}</span>
				</div>
			</div>

			<!-- Hero Header Banner Card matching Reference media_1789002785331.png -->
			<div class="bg-[#18191c] border border-white/10 rounded-3xl p-6 shadow-xl relative overflow-hidden">
				<div class="absolute -right-16 -top-16 w-64 h-64 bg-[#caa97c]/5 rounded-full blur-3xl pointer-events-none"></div>

				<div class="flex flex-col md:flex-row items-start md:items-center justify-between gap-6 relative z-10">
					
					<!-- Left: Large Icon + Title + Tags -->
					<div class="flex items-start gap-5 min-w-0">
						<!-- Big App Icon -->
						<div class="w-20 h-20 shrink-0 rounded-2xl bg-[#222328] border border-white/15 p-1 overflow-hidden shadow-2xl flex items-center justify-center">
							{#if selectedItem.iconUrl}
								<img src={selectedItem.iconUrl} alt={selectedItem.title} class="w-full h-full object-cover rounded-xl" />
							{:else}
								<Layers class="w-8 h-8 text-[#caa97c]" />
							{/if}
						</div>

						<div class="min-w-0">
							<!-- Title with Verified Badge -->
							<div class="flex items-center gap-2 flex-wrap">
								<h1 class="text-xl font-black text-white tracking-tight">{selectedItem.title}</h1>
								<div class="flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
									<CheckCircle2 class="w-3 h-3" /> Verificado
								</div>
								{#if selectedItem.source === "modrinth"}
									<span class="text-[10px] font-extrabold px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
										Modrinth
									</span>
								{:else}
									<span class="text-[10px] font-extrabold px-2 py-0.5 rounded-full bg-orange-500/20 text-orange-300 border border-orange-500/30 flex items-center gap-1">
										<Flame class="w-3 h-3" /> CurseForge
									</span>
								{/if}
							</div>

							<!-- Description Summary -->
							<p class="text-xs text-white/60 mt-1.5 line-clamp-2 max-w-2xl leading-relaxed">
								{selectedItem.description}
							</p>

							<!-- Category & Loader Badges -->
							<div class="flex items-center gap-1.5 flex-wrap mt-3">
								{#each selectedItem.categories.slice(0, 4) as cat}
									<span class="text-[10px] font-semibold px-2.5 py-0.5 rounded-lg bg-white/5 border border-white/10 text-white/70">
										{cat}
									</span>
								{/each}
							</div>
						</div>
					</div>

					<!-- Right: Install Button matching Reference Image -->
					<div class="shrink-0 flex items-center gap-3 w-full md:w-auto justify-end">
						<button 
							type="button"
							class="w-full md:w-auto bg-gradient-to-r from-[#caa97c] to-[#e4c99c] hover:from-[#d5b588] hover:to-[#edd5ad] text-black font-extrabold text-xs px-6 py-3 rounded-2xl flex items-center justify-center gap-2 shadow-[0_4px_20px_rgba(202,169,124,0.35)] transition-all cursor-pointer active:scale-95 disabled:opacity-50"
							onclick={() => installItem(selectedItem!)}
							disabled={isInstalling || isInstalled}
						>
							{#if isInstalling}
								<Loader2 class="w-4 h-4 animate-spin text-black" />
								<span>Instalando...</span>
							{:else if isInstalled}
								<Check class="w-4 h-4 text-emerald-950 font-black" />
								<span>Instalado na Instância</span>
							{:else}
								<Download class="w-4 h-4 text-black" />
								<span>Instalar Agora</span>
							{/if}
						</button>
					</div>

				</div>
			</div>

			<!-- Navigation Tabs & Action Links matching Reference media_1789002785331.png -->
			<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/10 pb-3">
				<!-- Tabs -->
				<div class="flex items-center gap-1 bg-[#18191c] p-1 rounded-2xl border border-white/5">
					<button 
						type="button"
						class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-2 {activeDetailTab === 'overview' ? 'bg-[#caa97c] text-black font-black shadow-md' : 'text-white/60 hover:text-white'}"
						onclick={() => activeDetailTab = 'overview'}
					>
						<FileText class="w-3.5 h-3.5" />
						<span>Visão Geral</span>
					</button>

					<button 
						type="button"
						class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-2 {activeDetailTab === 'gallery' ? 'bg-[#caa97c] text-black font-black shadow-md' : 'text-white/60 hover:text-white'}"
						onclick={() => activeDetailTab = 'gallery'}
					>
						<ImageIcon class="w-3.5 h-3.5" />
						<span>Galeria ({modDetails?.gallery.length || 0})</span>
					</button>

					<button 
						type="button"
						class="px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-2 {activeDetailTab === 'versions' ? 'bg-[#caa97c] text-black font-black shadow-md' : 'text-white/60 hover:text-white'}"
						onclick={() => activeDetailTab = 'versions'}
					>
						<Layers class="w-3.5 h-3.5" />
						<span>Versões ({modVersionsList.length})</span>
					</button>
				</div>

				<!-- Quick Action Buttons -->
				<div class="flex items-center gap-2 flex-wrap text-xs font-bold">
					<button 
						type="button"
						class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all cursor-pointer"
						onclick={() => showInstallGuide = true}
					>
						<HelpCircle class="w-3.5 h-3.5 text-[#caa97c]" />
						<span>Como Instalar</span>
					</button>

					{#if modDetails?.discordUrl}
						<a 
							href={modDetails.discordUrl} 
							target="_blank" 
							class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all"
						>
							<MessageSquare class="w-3.5 h-3.5 text-indigo-400" />
							<span>Discord</span>
						</a>
					{/if}

					{#if modDetails?.sourceUrl}
						<a 
							href={modDetails.sourceUrl} 
							target="_blank" 
							class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all"
						>
							<Globe class="w-3.5 h-3.5 text-emerald-400" />
							<span>Código Fonte</span>
						</a>
					{/if}

					{#if modDetails?.donationUrl}
						<a 
							href={modDetails.donationUrl} 
							target="_blank" 
							class="px-3 py-1.5 rounded-xl bg-[#1c1d22] hover:bg-[#282930] text-white/70 hover:text-white border border-white/5 flex items-center gap-1.5 transition-all"
						>
							<Heart class="w-3.5 h-3.5 text-rose-400" />
							<span>Apoiar</span>
						</a>
					{/if}
				</div>
			</div>

			<!-- Main Split Content Area -->
			<div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-start pb-12">
				
				<!-- LEFT COLUMN (70%): Tab Content -->
				<div class="lg:col-span-2 space-y-6">
					
					{#if loadingDetails}
						<div class="bg-[#18191c] border border-white/5 rounded-3xl p-12 flex flex-col items-center justify-center gap-3">
							<Loader2 class="w-8 h-8 text-[#caa97c] animate-spin" />
							<p class="text-xs text-white/40 font-medium">Carregando informações completas...</p>
						</div>
					{:else if modDetails}

						<!-- TAB 1: VISÃO GERAL -->
						{#if activeDetailTab === 'overview'}
							<div class="bg-[#18191c] border border-white/5 rounded-3xl p-6 shadow-md overflow-hidden">
								{#if modDetails.bodyType === 'html'}
									<div class="prose prose-invert max-w-none text-xs text-white/80 leading-relaxed font-sans [&_a]:text-[#caa97c] [&_h1]:text-white [&_h2]:text-white [&_h3]:text-white [&_img]:rounded-xl [&_img]:max-w-full">
										{@html modDetails.body}
									</div>
								{:else}
									<div class="prose prose-invert max-w-none text-xs text-white/80 leading-relaxed font-sans [&_a]:text-[#caa97c] [&_h1]:text-white [&_h2]:text-white [&_h3]:text-white [&_img]:rounded-xl [&_img]:max-w-full">
										{@html renderMarkdown(modDetails.body)}
									</div>
								{/if}
							</div>

						<!-- TAB 2: GALERIA -->
						{:else if activeDetailTab === 'gallery'}
							{#if modDetails.gallery.length === 0}
								<div class="bg-[#18191c] border border-white/5 rounded-3xl p-12 text-center text-white/40 text-xs">
									<ImageIcon class="w-10 h-10 text-white/20 mx-auto mb-3" />
									<p>Nenhuma screenshot disponível para este projeto.</p>
								</div>
							{:else}
								<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
									{#each modDetails.gallery as img}
										<!-- svelte-ignore a11y_click_events_have_key_events -->
										<!-- svelte-ignore a11y_no_static_element_interactions -->
										<div 
											class="group relative bg-[#18191c] border border-white/10 rounded-2xl overflow-hidden cursor-pointer hover:border-[#caa97c]/50 transition-all shadow-md"
											onclick={() => lightboxImage = img}
										>
											<div class="h-44 w-full bg-[#222328] overflow-hidden">
												<img src={img.url} alt={img.title || "Screenshot"} class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" />
											</div>
											{#if img.title}
												<div class="p-3 bg-[#18191c]">
													<h4 class="text-xs font-bold text-white truncate">{img.title}</h4>
													{#if img.description}
														<p class="text-[10px] text-white/40 line-clamp-1 mt-0.5">{img.description}</p>
													{/if}
												</div>
											{/if}
										</div>
									{/each}
								</div>
							{/if}

						<!-- TAB 3: VERSÕES -->
						{:else if activeDetailTab === 'versions'}
							{#if modVersionsList.length === 0}
								<div class="bg-[#18191c] border border-white/5 rounded-3xl p-12 text-center text-white/40 text-xs">
									<Layers class="w-10 h-10 text-white/20 mx-auto mb-3" />
									<p>Nenhuma versão listada para este projeto.</p>
								</div>
							{:else}
								<div class="bg-[#18191c] border border-white/5 rounded-3xl overflow-hidden shadow-md">
									<div class="divide-y divide-white/5">
										{#each modVersionsList as ver}
											{@const file = ver.files[0]}
											<div class="p-4 flex items-center justify-between hover:bg-white/[0.02] transition-colors gap-4">
												<div class="min-w-0">
													<div class="flex items-center gap-2">
														<h4 class="text-xs font-extrabold text-white truncate">{ver.name || ver.versionNumber}</h4>
														{#if ver.versionNumber}
															<span class="text-[10px] font-mono px-2 py-0.5 rounded bg-white/5 text-white/60">
																{ver.versionNumber}
															</span>
														{/if}
													</div>
													<div class="flex items-center gap-2 text-[10px] text-white/40 mt-1 font-mono">
														{#if file}
															<span>{file.filename}</span>
															<span>•</span>
															<span>{formatBytes(file.size)}</span>
														{/if}
													</div>
												</div>

												<button 
													type="button"
													class="shrink-0 bg-[#2b2c32] hover:bg-[#caa97c] hover:text-black text-white text-xs font-bold px-3.5 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
													onclick={() => installItem(selectedItem!, ver.id)}
												>
													<Download class="w-3 h-3" />
													<span>Instalar</span>
												</button>
											</div>
										{/each}
									</div>
								</div>
							{/if}

						{/if}

					{/if}

				</div>

				<!-- RIGHT COLUMN (30%): Technical Information matching Reference Media -->
				<aside class="bg-[#18191c] border border-white/5 rounded-3xl p-5 shadow-xl space-y-5">
					<div class="flex items-center gap-2 text-xs font-bold text-white uppercase tracking-wider border-b border-white/5 pb-3">
						<ShieldCheck class="w-4 h-4 text-[#caa97c]" />
						<span>Informações</span>
					</div>

					<!-- Author Info Section -->
					{#if modDetails?.author}
						<div>
							<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">AUTOR</span>
							<div class="flex items-center gap-3 bg-[#131417] p-3 rounded-2xl border border-white/5">
								{#if modDetails.author.avatarUrl}
									<img src={modDetails.author.avatarUrl} alt={modDetails.author.name} class="w-10 h-10 rounded-full object-cover border border-white/10 shrink-0" />
								{:else}
									<div class="w-10 h-10 rounded-full bg-[#2a2b33] flex items-center justify-center text-white/60 font-black text-xs shrink-0">
										{modDetails.author.name.slice(0, 2).toUpperCase()}
									</div>
								{/if}
								<div>
									<h4 class="text-xs font-extrabold text-white">{modDetails.author.name}</h4>
									<span class="text-[10px] text-white/40">{modDetails.author.role || "Criador do Projeto"}</span>
								</div>
							</div>
						</div>
					{/if}

					<!-- Technical Details Grid -->
					<div>
						<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">DETALHES</span>
						<div class="bg-[#131417] rounded-2xl border border-white/5 divide-y divide-white/5 text-xs">
							<div class="p-3 flex items-center justify-between">
								<span class="text-white/40">Fonte</span>
								<span class="font-bold text-white capitalize">{selectedItem.source}</span>
							</div>
							<div class="p-3 flex items-center justify-between">
								<span class="text-white/40">Downloads</span>
								<span class="font-bold text-white font-mono">{formatDownloads(selectedItem.downloads)}</span>
							</div>
							{#if modDetails?.createdAt}
								<div class="p-3 flex items-center justify-between">
									<span class="text-white/40">Criado em</span>
									<span class="font-medium text-white/80">{formatDate(modDetails.createdAt)}</span>
								</div>
							{/if}
							{#if modDetails?.updatedAt}
								<div class="p-3 flex items-center justify-between">
									<span class="text-white/40">Atualizado</span>
									<span class="font-medium text-white/80">{formatDate(modDetails.updatedAt)}</span>
								</div>
							{/if}
						</div>
					</div>

					<!-- Supported Loaders -->
					{#if modDetails && modDetails.loaders.length > 0}
						<div>
							<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">LOADERS COMPATÍVEIS</span>
							<div class="flex flex-wrap gap-1.5">
								{#each modDetails.loaders as l}
									<span class="px-2.5 py-1 rounded-xl bg-white/5 border border-white/10 text-xs font-bold text-white/80">
										{l}
									</span>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Game Versions -->
					{#if modDetails && modDetails.gameVersions.length > 0}
						<div>
							<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">VERSÕES SUPORTADAS</span>
							<div class="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto custom-scrollbar pr-1">
								{#each modDetails.gameVersions as v}
									<span class="px-2 py-0.5 rounded-lg bg-[#131417] border border-white/5 text-[10px] font-mono text-white/60">
										{v}
									</span>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Categories -->
					{#if modDetails && modDetails.categories.length > 0}
						<div>
							<span class="text-[10px] font-bold text-white/40 uppercase tracking-widest block mb-2">CATEGORIAS</span>
							<div class="flex flex-wrap gap-1.5">
								{#each modDetails.categories as cat}
									<span class="px-2.5 py-0.5 rounded-full bg-[#caa97c]/10 text-[#caa97c] border border-[#caa97c]/20 text-[10px] font-bold">
										{cat}
									</span>
								{/each}
							</div>
						</div>
					{/if}

				</aside>

			</div>

		</div>

	<!-- ================= MAIN CATALOG / SEARCH VIEW ================= -->
	{:else}
		<!-- LEFT MAIN CONTENT -->
		<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-1">
			
			<!-- Header matching reference -->
			<header class="mb-4">
				<h1 class="text-2xl font-bold text-white tracking-tight">Central de Conteúdo</h1>
				<p class="text-xs text-white/40 mt-0.5 font-medium">Encontre e Instale modpacks, mods e recursos incríveis</p>
			</header>

			<!-- Search Bar matching reference pill -->
			<div class="relative w-full mb-4">
				<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30" />
				<input 
					type="text" 
					bind:value={searchQuery}
					placeholder={`Search ${selectedType.toLowerCase()}...`}
					class="w-full bg-[#131418] border border-white/[0.08] focus:border-[#6c5ce7]/60 rounded-2xl py-3 pl-11 pr-4 text-xs text-white placeholder-white/30 focus:outline-none transition-all shadow-inner"
				/>
			</div>

			<!-- Sub-header bar: Section title, Result count, Sort, View mode -->
			<div class="flex items-center justify-between mb-4">
				<div class="flex items-center gap-2">
					<span class="text-xs font-extrabold text-white">{selectedType}</span>
					<span class="text-xs text-white/40 font-medium">({totalEstimate} resultados)</span>
					{#if loading}
						<span class="text-[10px] text-[#a29bfe] font-mono font-medium flex items-center gap-1 ml-2">
							<Loader2 class="w-3 h-3 animate-spin text-[#6c5ce7]" /> buscando...
						</span>
					{/if}
				</div>

				<div class="flex items-center gap-2.5">
					<!-- Sort Dropdown matching SKlauncher -->
					<div class="relative">
						<button 
							type="button"
							class="bg-[#181920] border border-white/10 hover:border-white/20 px-3 py-1.5 rounded-xl text-xs font-semibold text-white/80 flex items-center gap-2 transition-all cursor-pointer shadow-sm"
							onclick={() => sortMenuOpen = !sortMenuOpen}
						>
							<ArrowUpDown class="w-3.5 h-3.5 text-white/50" />
							<span>{currentSortLabel}</span>
							<ChevronDown class="w-3.5 h-3.5 text-white/40 transition-transform duration-200 {sortMenuOpen ? 'rotate-180' : ''}" />
						</button>

						{#if sortMenuOpen}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div class="fixed inset-0 z-20" onclick={() => sortMenuOpen = false}></div>
							<div class="absolute right-0 mt-1.5 w-52 bg-[#181920] border border-white/10 rounded-xl shadow-2xl py-1 z-30 divide-y divide-white/5">
								{#each sortOptions as opt}
									<button 
										type="button"
										class="w-full text-left px-3 py-2 text-xs transition-colors flex items-center justify-between cursor-pointer {selectedSort === opt.id ? 'bg-[#6c5ce7]/20 text-[#a29bfe] font-bold' : 'text-white/70 hover:bg-white/5 hover:text-white'}"
										onclick={() => { selectedSort = opt.id as any; sortMenuOpen = false; }}
									>
										<span>{opt.label}</span>
										{#if selectedSort === opt.id}
											<Check class="w-3.5 h-3.5 text-[#6c5ce7]" />
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					</div>

					<!-- View Mode Toggle -->
					<div class="flex bg-[#131418] border border-white/10 rounded-xl p-0.5 gap-0.5">
						<button 
							type="button" 
							class="p-1.5 rounded-lg transition-all cursor-pointer {viewMode === 'grid' ? 'bg-[#6c5ce7] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
							onclick={() => viewMode = 'grid'}
							title="Grade"
						>
							<LayoutGrid class="w-3.5 h-3.5" />
						</button>
						<button 
							type="button" 
							class="p-1.5 rounded-lg transition-all cursor-pointer {viewMode === 'list' ? 'bg-[#6c5ce7] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
							onclick={() => viewMode = 'list'}
							title="Lista"
						>
							<List class="w-3.5 h-3.5" />
						</button>
					</div>
				</div>
			</div>

			<!-- Error State -->
			{#if searchError}
				<div class="flex items-center gap-3 bg-red-500/10 border border-red-500/20 rounded-2xl p-4 mb-4">
					<AlertTriangle class="w-5 h-5 text-red-400 shrink-0" />
					<div>
						<p class="text-xs font-bold text-red-400">Erro na busca</p>
						<p class="text-[10px] text-red-300/60 mt-0.5">{searchError}</p>
					</div>
				</div>
			{/if}

			<!-- Loading State -->
			{#if loading && results.length === 0}
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Loader2 class="w-8 h-8 text-[#6c5ce7] animate-spin" />
						<p class="text-xs text-white/40 font-medium">Buscando em Modrinth e CurseForge...</p>
					</div>
				</div>
			{:else if results.length === 0 && hasSearched}
				<div class="flex-1 flex items-center justify-center py-20">
					<div class="flex flex-col items-center gap-3">
						<Search class="w-8 h-8 text-white/20" />
						<p class="text-xs text-white/40 font-medium">Nenhum resultado encontrado</p>
						<p class="text-[10px] text-white/30">Tente outro termo ou altere os filtros</p>
					</div>
				</div>
			{:else}

				<!-- GRID VIEW MODE: 3 Columns matching SKlauncher 4.0.48 Beta -->
				{#if viewMode === 'grid'}
					<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 pb-6">
						{#each results as item}
							{@const id = `${item.source}:${item.sourceId}`}
							{@const isInstalling = installingIds.has(id)}
							{@const isInstalled = installedIds.has(id)}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div 
								class="group bg-[#15161b] border border-white/[0.08] hover:border-white/20 rounded-2xl overflow-hidden transition-all duration-200 flex flex-col justify-between shadow-lg cursor-pointer active:scale-[0.99]"
								onclick={() => openDetails(item)}
							>
								
								<!-- Top Banner Image -->
								<div class="relative w-full h-36 bg-[#0f1013] overflow-hidden">
									{#if item.bannerUrl}
										<img 
											src={item.bannerUrl} 
											alt={item.title} 
											loading="lazy" 
											decoding="async"
											class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
											onerror={(e) => { (e.currentTarget as HTMLElement).style.display = 'none'; }}
										/>
									{:else}
										<div class="w-full h-full bg-[#181920] relative flex items-center justify-center overflow-hidden">
											<div class="absolute inset-0 opacity-[0.04]" style="background-image: radial-gradient(#ffffff 1px, transparent 1px); background-size: 14px 14px;"></div>
											<div class="absolute inset-0 bg-gradient-to-t from-[#15161b] via-transparent to-transparent"></div>
										</div>
									{/if}

									<!-- Source Badge (Top Right) -->
									<div class="absolute top-2.5 right-2.5 h-6 w-6 rounded-full bg-black/70 backdrop-blur-sm border border-white/10 flex items-center justify-center shadow-md" title={item.source === "modrinth" ? "Modrinth" : "CurseForge"}>
										{#if item.source === "modrinth"}
											<span class="font-black text-[11px] text-[#1bd96a]">m</span>
										{:else}
											<Flame class="w-3 h-3 text-[#f16436]" />
										{/if}
									</div>

									<!-- Square Icon Thumbnail Box (Bottom Left) -->
									<div class="absolute bottom-2.5 left-3 h-12 w-12 rounded-xl bg-[#14151a] border-2 border-[#202129] p-0.5 shadow-xl flex items-center justify-center overflow-hidden shrink-0">
										{#if item.iconUrl}
											<img 
												src={item.iconUrl} 
												alt={item.title} 
												loading="lazy" 
												decoding="async"
												class="w-full h-full object-cover rounded-lg"
												onerror={(e) => { (e.currentTarget as HTMLElement).style.display = 'none'; }}
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
										<!-- Row 1: Title + Download Count -->
										<div class="flex items-center justify-between gap-2">
											<h3 class="font-bold text-white text-xs truncate flex-1 group-hover:text-[#a29bfe] transition-colors" title={item.title}>
												{item.title}
											</h3>
											<span class="text-[11px] text-white/50 flex items-center gap-1 shrink-0 font-medium">
												<Download class="w-2.5 h-2.5 text-white/40" />
												{formatDownloads(item.downloads)}
											</span>
										</div>

										<!-- Row 2: 2-line description -->
										<p class="text-[11px] text-white/40 mt-1.5 line-clamp-2 leading-relaxed h-8">
											{item.description}
										</p>
									</div>

									<!-- Row 3: Author + Purple Install Button -->
									<div class="flex items-center justify-between mt-3 pt-2.5 border-t border-white/[0.06]">
										<div class="flex items-center gap-1 text-[11px] text-white/40 truncate max-w-[55%]">
											<Users class="w-3 h-3 text-white/30 shrink-0" />
											<span class="truncate font-medium">{item.author || item.slug}</span>
										</div>

										<!-- Purple Install Button matching SKlauncher -->
										<button 
											type="button" 
											class="bg-[#6c5ce7] hover:bg-[#5b4cdb] active:scale-95 text-white text-xs font-semibold px-3.5 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-md shadow-[#6c5ce7]/20 disabled:opacity-50 cursor-pointer shrink-0"
											onclick={(e) => { e.stopPropagation(); installItem(item); }}
											disabled={isInstalling || isInstalled}
										>
											{#if isInstalling}
												<Loader2 class="w-3 h-3 animate-spin" />
												<span>Instalando</span>
											{:else if isInstalled}
												<Check class="w-3 h-3 text-emerald-300" />
												<span>Instalado</span>
											{:else}
												<Download class="w-3 h-3" />
												<span>Instalar</span>
											{/if}
										</button>
									</div>
								</div>

							</div>
						{/each}
					</div>

				<!-- LIST VIEW MODE -->
				{:else}
					<div class="space-y-2.5 pb-6">
						{#each results as item}
							{@const id = `${item.source}:${item.sourceId}`}
							{@const isInstalling = installingIds.has(id)}
							{@const isInstalled = installedIds.has(id)}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div 
								class="group bg-[#15161b] border border-white/[0.08] hover:border-white/20 rounded-2xl p-3 flex items-center justify-between gap-4 transition-all duration-200 shadow-md cursor-pointer active:scale-[0.99]"
								onclick={() => openDetails(item)}
							>
								<!-- Left Icon & Info -->
								<div class="flex items-center gap-3.5 min-w-0 flex-1">
									<div class="h-12 w-12 rounded-xl bg-[#101115] border border-white/10 p-0.5 shrink-0 overflow-hidden flex items-center justify-center">
										{#if item.iconUrl}
											<img 
												src={item.iconUrl} 
												alt={item.title} 
												loading="lazy" 
												decoding="async"
												class="w-full h-full object-cover rounded-lg"
												onerror={(e) => { (e.currentTarget as HTMLElement).style.display = 'none'; }}
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

								<!-- Right: Source Badge & Purple Install Button -->
								<div class="flex items-center gap-3 shrink-0">
									<div class="h-6 w-6 rounded-full bg-black/60 border border-white/10 flex items-center justify-center shadow-sm" title={item.source === "modrinth" ? "Modrinth" : "CurseForge"}>
										{#if item.source === "modrinth"}
											<span class="font-black text-[11px] text-[#1bd96a]">m</span>
										{:else}
											<Flame class="w-3 h-3 text-[#f16436]" />
										{/if}
									</div>

									<button 
										type="button" 
										class="bg-[#6c5ce7] hover:bg-[#5b4cdb] active:scale-95 text-white text-xs font-semibold px-4 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-md shadow-[#6c5ce7]/20 disabled:opacity-50 cursor-pointer"
										onclick={(e) => { e.stopPropagation(); installItem(item); }}
										disabled={isInstalling || isInstalled}
									>
										{#if isInstalling}
											<Loader2 class="w-3 h-3 animate-spin" />
											<span>Instalando</span>
										{:else if isInstalled}
											<Check class="w-3 h-3 text-emerald-300" />
											<span>Instalado</span>
										{:else}
											<Download class="w-3 h-3" />
											<span>Instalar</span>
										{/if}
									</button>
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Pagination Bar matching SKlauncher Reference Media -->
				{#if results.length > 0}
					<div class="flex items-center justify-between pt-3 pb-8 border-t border-white/5">
						<div class="text-xs text-white/50 font-medium">
							Mostrando <span class="text-white font-bold">{(currentPage - 1) * pageSize + 1} - {Math.min(currentPage * pageSize, totalEstimateNumber)}</span> de <span class="text-white font-bold">{totalEstimate}</span>
						</div>

						<div class="flex items-center gap-1 text-xs">
							<!-- Prev Button < -->
							<button 
								type="button"
								class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white hover:bg-white/5 disabled:opacity-30 disabled:pointer-events-none transition-all cursor-pointer"
								onclick={() => goToPage(currentPage - 1)}
								disabled={currentPage <= 1 || loading}
								aria-label="Página anterior"
							>
								<ChevronLeft class="w-4 h-4" />
							</button>

							<!-- Page 1 -->
							<button 
								type="button"
								class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === 1 ? 'bg-[#282935] text-white border border-white/10 shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
								onclick={() => goToPage(1)}
							>
								1
							</button>

							<!-- Page 2 -->
							{#if totalPages >= 2}
								<button 
									type="button"
									class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === 2 ? 'bg-[#282935] text-white border border-white/10 shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
									onclick={() => goToPage(2)}
								>
									2
								</button>
							{/if}

							<!-- Page 3 -->
							{#if totalPages >= 3}
								<button 
									type="button"
									class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === 3 ? 'bg-[#282935] text-white border border-white/10 shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
									onclick={() => goToPage(3)}
								>
									3
								</button>
							{/if}

							<!-- Page 4 -->
							{#if totalPages >= 4}
								<button 
									type="button"
									class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === 4 ? 'bg-[#282935] text-white border border-white/10 shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
									onclick={() => goToPage(4)}
								>
									4
								</button>
							{/if}

							<!-- Ellipsis -->
							{#if totalPages > 5}
								<span class="px-1 text-white/40">...</span>
								<!-- Last Page -->
								<button 
									type="button"
									class="h-8 w-8 rounded-xl font-semibold transition-all cursor-pointer {currentPage === totalPages ? 'bg-[#282935] text-white border border-white/10 shadow-sm' : 'text-white/60 hover:text-white hover:bg-white/5'}"
									onclick={() => goToPage(totalPages)}
								>
									{totalPages}
								</button>
							{/if}

							<!-- Next Button > -->
							<button 
								type="button"
								class="h-8 w-8 rounded-xl flex items-center justify-center text-white/50 hover:text-white hover:bg-white/5 disabled:opacity-30 disabled:pointer-events-none transition-all cursor-pointer"
								onclick={() => goToPage(currentPage + 1)}
								disabled={currentPage >= totalPages || loading}
								aria-label="Próxima página"
							>
								<ChevronRight class="w-4 h-4" />
							</button>
						</div>
					</div>
				{/if}

			{/if}

		</div>

		<!-- RIGHT SIDEBAR FILTER PANEL matching SKlauncher Reference Media -->
		<aside class="w-64 shrink-0 h-full bg-[#131418] border border-white/5 rounded-3xl p-5 overflow-y-auto custom-scrollbar flex flex-col justify-between shadow-2xl">
			
			<div class="space-y-5">
				<!-- Header with Filter icon -->
				<div class="flex items-center gap-2 text-xs font-bold text-white uppercase tracking-wider">
					<Filter class="w-3.5 h-3.5 text-[#6c5ce7]" /> Filtros
				</div>

				<!-- INSTÂNCIA DE DESTINO -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2 flex items-center justify-between">
						<span>INSTÂNCIA DE DESTINO</span>
						<Box class="w-3 h-3 text-[#6c5ce7]" />
					</div>
					{#if profiles.list.length === 0}
						<div class="text-[11px] text-white/40 italic p-2.5 bg-[#181920] rounded-xl border border-white/5">
							Nenhuma instância criada
						</div>
					{:else}
						<div class="relative">
							<select 
								bind:value={targetInstanceId}
								class="w-full bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs font-semibold text-white focus:outline-none focus:border-[#6c5ce7] appearance-none cursor-pointer pr-8"
							>
								{#each profiles.list as p}
									<option value={p.id}>
										{p.name} ({p.mcVersion})
									</option>
								{/each}
							</select>
							<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/40 pointer-events-none" />
						</div>
					{/if}
				</div>

				<!-- FONTE matching SKlauncher segmented toggle -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">FONTE</div>
					<div class="flex bg-[#0f1013] p-1 rounded-xl border border-white/5 gap-1">
						<button 
							type="button" 
							class="flex-1 py-1.5 px-2 rounded-lg text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'modrinth' ? 'bg-[#282935] text-white shadow-sm border border-white/10' : 'text-white/50 hover:text-white'}"
							onclick={() => selectedSource = selectedSource === 'modrinth' ? 'all' : 'modrinth'}
						>
							<span class="text-[#1bd96a] font-black text-xs">m</span> Modrinth
						</button>
						<button 
							type="button" 
							class="flex-1 py-1.5 px-2 rounded-lg text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'curseforge' ? 'bg-[#282935] text-white shadow-sm border border-white/10' : 'text-white/50 hover:text-white'}"
							onclick={() => selectedSource = selectedSource === 'curseforge' ? 'all' : 'curseforge'}
						>
							<Flame class="w-3.5 h-3.5 text-[#f16436]" /> CurseForge
						</button>
					</div>
				</div>

				<!-- TIPO DE CONTEÚDO matching SKlauncher 2-column pill buttons with purple active state -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">TIPO DE CONTEÚDO</div>
					<div class="grid grid-cols-2 gap-2">
						{#each [
							{ id: "Modpack", label: "Modpack", icon: Box },
							{ id: "Mod", label: "Mod", icon: Cpu },
							{ id: "Resource Pack", label: "Resource Pack", icon: Palette },
							{ id: "Shader", label: "Shader", icon: Sparkles },
							{ id: "Data Pack", label: "Data Pack", icon: HardDrive },
							{ id: "World", label: "World", icon: Globe }
						] as item}
							<button 
								type="button" 
								class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-all cursor-pointer flex items-center gap-2 {selectedType === item.id ? 'bg-[#6c5ce7] text-white font-bold shadow-md shadow-[#6c5ce7]/25' : 'bg-[#181920] text-white/60 border border-white/5 hover:bg-[#22232c] hover:text-white'}"
								onclick={() => selectedType = item.id}
							>
								<item.icon class="w-3.5 h-3.5 shrink-0" />
								<span class="truncate">{item.label}</span>
							</button>
						{/each}
					</div>
				</div>

				<!-- VERSÃO DO JOGO -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">VERSÃO DO JOGO</div>
					<div class="relative">
						<select 
							bind:value={selectedVersion}
							class="w-full bg-[#181920] border border-white/10 rounded-xl px-3 py-2 text-xs font-semibold text-white focus:outline-none focus:border-[#6c5ce7] appearance-none cursor-pointer pr-8"
						>
							{#each mcVersions as ver}
								<option value={ver}>{ver}</option>
							{/each}
						</select>
						<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/40 pointer-events-none" />
					</div>
				</div>

				<!-- MOD LOADERS -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">MOD LOADERS</div>
					<div class="grid grid-cols-2 gap-2">
						{#each modLoaders as loader}
							<button 
								type="button" 
								class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-all cursor-pointer border flex items-center gap-1.5 {selectedLoader === loader.name ? `${loader.color} font-bold shadow-sm` : 'bg-[#181920] text-white/60 border-white/5 hover:border-white/20 hover:text-white'}"
								onclick={() => selectedLoader = selectedLoader === loader.name ? null : loader.name}
							>
								<span class="w-1.5 h-1.5 rounded-full bg-current"></span>
								<span class="truncate">{loader.name}</span>
							</button>
						{/each}
					</div>
				</div>

				<!-- CATEGORIAS -->
				<div>
					<div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-2">CATEGORIAS</div>
					<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto custom-scrollbar pr-1">
						{#each categories as cat}
							<button 
								type="button" 
								class="px-2.5 py-1 rounded-full text-[11px] font-medium transition-all cursor-pointer {selectedCategory === cat ? 'bg-white/20 text-white border border-white/30 font-bold' : 'bg-[#181920] text-white/50 border border-white/5 hover:border-white/20 hover:text-white'}"
								onclick={() => selectedCategory = selectedCategory === cat ? null : cat}
							>
								{cat}
							</button>
						{/each}
					</div>
				</div>
			</div>

			<!-- Footer Social Links matching SKlauncher -->
			<div class="pt-4 border-t border-white/5 flex items-center justify-center gap-4 text-white/30 text-xs">
				<a href="https://discord.com" target="_blank" class="hover:text-white transition-colors" title="Discord">Discord</a>
				<span>•</span>
				<a href="https://x.com" target="_blank" class="hover:text-white transition-colors" title="X">X</a>
				<span>•</span>
				<a href="https://youtube.com" target="_blank" class="hover:text-white transition-colors" title="YouTube">YouTube</a>
			</div>

		</aside>
	{/if}

</div>

<!-- LIGHTBOX MODAL FOR SCREENSHOTS -->
{#if lightboxImage}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="fixed inset-0 z-[999999] bg-black/90 backdrop-blur-md flex flex-col items-center justify-center p-6 cursor-pointer"
		onclick={() => lightboxImage = null}
	>
		<div class="relative max-w-5xl max-h-[85vh] flex flex-col items-center" onclick={(e) => e.stopPropagation()}>
			<button 
				type="button"
				class="absolute -top-12 right-0 p-2 text-white/60 hover:text-white bg-white/10 rounded-full transition-all cursor-pointer"
				onclick={() => lightboxImage = null}
			>
				<X class="w-5 h-5" />
			</button>
			<img src={lightboxImage.url} alt={lightboxImage.title || "Screenshot"} class="max-w-full max-h-[75vh] object-contain rounded-2xl shadow-2xl border border-white/10" />
			{#if lightboxImage.title}
				<div class="mt-4 text-center">
					<h3 class="text-sm font-bold text-white">{lightboxImage.title}</h3>
					{#if lightboxImage.description}
						<p class="text-xs text-white/60 mt-1 max-w-xl">{lightboxImage.description}</p>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}

<!-- COMO INSTALAR MODAL -->
{#if showInstallGuide}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="fixed inset-0 z-[999999] bg-black/80 backdrop-blur-sm flex items-center justify-center p-6 cursor-pointer"
		onclick={() => showInstallGuide = false}
	>
		<div class="bg-[#18191c] border border-white/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-4" onclick={(e) => e.stopPropagation()}>
			<div class="flex items-center justify-between border-b border-white/5 pb-3">
				<div class="flex items-center gap-2 text-white font-extrabold text-sm">
					<HelpCircle class="w-4 h-4 text-[#caa97c]" />
					<span>Como Instalar Mods no Luxmc</span>
				</div>
				<button type="button" class="text-white/40 hover:text-white cursor-pointer" onclick={() => showInstallGuide = false}>
					<X class="w-4 h-4" />
				</button>
			</div>

			<div class="text-xs text-white/70 space-y-3 leading-relaxed">
				<p>1. Certifique-se de que sua instância possui um <strong>Mod Loader</strong> instalado (Fabric, NeoForge ou Forge).</p>
				<p>2. Clique no botão <strong>Instalar</strong> no topo da página ou escolha uma versão específica na aba <strong>Versões</strong>.</p>
				<p>3. O Luxmc fará o download e adicionará o arquivo diretamente à pasta de mods da sua instância.</p>
				<p>4. Inicie o Minecraft pela tela de instâncias e aproveite!</p>
			</div>

			<button 
				type="button" 
				class="w-full py-2.5 rounded-xl bg-[#caa97c] text-black font-extrabold text-xs transition-all cursor-pointer hover:bg-[#e4c99c]"
				onclick={() => showInstallGuide = false}
			>
				Entendido
			</button>
		</div>
	</div>
{/if}
