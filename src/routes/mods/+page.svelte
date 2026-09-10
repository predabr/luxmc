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
		Check
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { modsSearch, modsSearchTyped, modsVersions, modsInstall } from "$lib/api";

	let searchQuery = $state("");
	let selectedSource = $state<"all" | "modrinth" | "curseforge">("all");
	let selectedType = $state("Mod");
	let selectedLoader = $state<string | null>(null);
	let selectedCategory = $state<string | null>(null);
	let selectedVersion = $state("1.21.4");
	let viewMode = $state<"grid" | "list">("grid");

	let results = $state<Array<{
		slug: string;
		title: string;
		description: string;
		downloads: number;
		iconUrl: string | null;
		categories: string[];
		versions: string[];
		source: string;
		sourceId: string;
	}>>([]);
	let loading = $state(false);
	let searchError = $state<string | null>(null);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let currentPage = $state(1);
	const pageSize = 24;
	let installingIds = $state<Set<string>>(new Set());
	let installedIds = $state<Set<string>>(new Set());
	let hasSearched = $state(false);

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

	async function doSearch(page = currentPage) {
		loading = true;
		searchError = null;
		hasSearched = true;

		try {
			const effectiveVer = selectedVersion === "Qualquer Versão" ? "" : selectedVersion;
			const typeSlug = selectedType.toLowerCase().replace(/\s+/g, "");
			const offset = (page - 1) * pageSize;

			let allResults = await modsSearch(
				searchQuery.trim(),
				effectiveVer,
				pageSize,
				offset,
				typeSlug
			);

			if (selectedSource !== "all") {
				allResults = allResults.filter(r => r.source === selectedSource);
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

	function debouncedSearch() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			doSearch(1);
		}, 300);
	}

	function goToPage(p: number) {
		if (p < 1 || loading) return;
		currentPage = p;
		doSearch(p);
	}

	$effect(() => {
		searchQuery;
		selectedVersion;
		selectedType;
		selectedSource;
		selectedLoader;
		selectedCategory;
		currentPage = 1;
		debouncedSearch();
	});

	async function installItem(item: typeof results[0]) {
		const id = `${item.source}:${item.sourceId}`;
		if (installingIds.has(id)) return;

		installingIds = new Set([...installingIds, id]);

		try {
			const effectiveVer = selectedVersion === "Qualquer Versão" ? "1.21.4" : selectedVersion;
			let versions = await modsVersions(item.sourceId, effectiveVer, item.source);
			if (versions.length === 0 && selectedVersion !== "Qualquer Versão") {
				versions = await modsVersions(item.sourceId, "1.21.4", item.source);
			}
			if (versions.length === 0) {
				toast(`Nenhuma versão compatível encontrada para "${item.title}"`, "error");
				return;
			}

			const version = versions[0];
			await modsInstall({
				profileId: "default",
				projectId: item.sourceId,
				versionId: version.id,
				source: item.source,
			});

			installedIds = new Set([...installedIds, id]);
			toast(`"${item.title}" instalado com sucesso!`, "success");
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
	
	<!-- LEFT MAIN CONTENT -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-1">
		
		<!-- Header matching reference -->
		<header class="mb-5">
			<h1 class="text-2xl font-bold text-white tracking-tight">Central de Conteúdo</h1>
			<p class="text-xs text-white/40 mt-1 font-medium">Encontre e instale modpacks, mods e recursos incríveis</p>
		</header>

		<!-- Search Bar matching reference pill -->
		<div class="relative w-full mb-5">
			<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30" />
			<input 
				type="text" 
				bind:value={searchQuery}
				placeholder="Search modpack..."
				class="w-full bg-[#18191c] border border-white/10 rounded-2xl py-3 pl-11 pr-4 text-xs text-white placeholder-white/30 focus:outline-none focus:border-[#c5a880]/50 transition-colors shadow-inner"
			/>
		</div>

		<!-- Sub-header bar: Section title, Result count, Sort, View mode -->
		<div class="flex items-center justify-between mb-4">
			<div class="flex items-center gap-2">
				<span class="text-xs font-extrabold text-white">{selectedType}</span>
				{#if loading}
					<span class="text-[10px] text-white/40 font-mono font-medium flex items-center gap-1">
						<Loader2 class="w-3 h-3 animate-spin text-[#c5a880]" /> buscando...
					</span>
				{:else}
					<span class="text-[10px] text-white/40 font-mono font-medium">[{results.length} itens • pág. {currentPage}]</span>
				{/if}
			</div>

			<div class="flex items-center gap-2.5">
				<!-- Sort Dropdown -->
				<div class="bg-[#18191c] border border-white/10 px-3 py-1.5 rounded-xl text-xs font-bold text-white/70 flex items-center gap-2 cursor-pointer hover:border-white/20 transition-all">
					<SlidersHorizontal class="w-3 h-3 text-white/50" />
					<span>Downloads</span>
				</div>

				<!-- View Mode Toggle -->
				<div class="flex bg-[#18191c] border border-white/10 rounded-xl p-1 gap-1">
					<button 
						type="button"
						class="p-1 rounded-lg text-white/50 hover:text-white transition-all {viewMode === 'grid' ? 'bg-white/10 text-white shadow-sm' : ''}"
						onclick={() => viewMode = 'grid'}
					>
						<LayoutGrid class="w-3.5 h-3.5" />
					</button>
					<button 
						type="button"
						class="p-1 rounded-lg text-white/50 hover:text-white transition-all {viewMode === 'list' ? 'bg-white/10 text-white shadow-sm' : ''}"
						onclick={() => viewMode = 'list'}
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
					<Loader2 class="w-8 h-8 text-[#c5a880] animate-spin" />
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
			<!-- Cards Grid matching Reference Image 2 -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pb-8">
				{#each results as item}
					{@const id = `${item.source}:${item.sourceId}`}
					{@const isInstalling = installingIds.has(id)}
					{@const isInstalled = installedIds.has(id)}
					<div class="group bg-[#18191c] border border-white/5 rounded-2xl overflow-hidden hover:border-white/20 transition-all shadow-md flex flex-col justify-between">
						
						<!-- Header Banner Image -->
						<div class="h-32 w-full relative bg-[#222328] overflow-hidden">
							{#if item.iconUrl}
								<img src={item.iconUrl} alt={item.title} class="w-full h-full object-cover opacity-85 group-hover:scale-105 transition-transform duration-300" />
							{:else}
								<div class="w-full h-full flex items-center justify-center text-white/10">
									<Layers class="w-10 h-10" />
								</div>
							{/if}
							<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-[#18191c]/20 to-transparent"></div>
							
							<!-- Source Badge matching Image 2 top right circular button -->
							<div class="absolute top-2.5 right-2.5 h-6 w-6 rounded-full bg-black/60 border border-white/10 flex items-center justify-center shadow-md" title={item.source === "modrinth" ? "Modrinth" : "CurseForge"}>
								{#if item.source === "modrinth"}
									<span class="font-black text-[10px] text-emerald-400">m</span>
								{:else}
									<Flame class="w-3 h-3 text-orange-400" />
								{/if}
							</div>

							<!-- Icon Overlay at bottom left of cover -->
							{#if item.iconUrl}
								<div class="absolute bottom-2 left-2.5 h-9 w-9 rounded-xl bg-black/60 border border-white/10 p-0.5 overflow-hidden flex items-center justify-center shadow-md">
									<img src={item.iconUrl} alt={item.title} class="w-full h-full object-cover rounded-lg" />
								</div>
							{/if}
						</div>

						<!-- Details -->
						<div class="p-4 pt-3 flex-1 flex flex-col justify-between">
							<div>
								<h3 class="font-extrabold text-white text-xs truncate group-hover:text-[#caa97c] transition-colors">{item.title}</h3>
								<p class="text-[10px] text-white/40 mt-1 line-clamp-2 leading-relaxed">{item.description}</p>
							</div>

							<div class="flex items-center justify-between mt-4 pt-2.5 border-t border-white/5">
								<div class="text-[10px] text-white/40 font-medium flex items-center gap-2">
									<span class="text-white/60 font-bold flex items-center gap-1">
										<Users class="w-3 h-3 text-white/40" /> {item.slug}
									</span>
									<span>•</span>
									<span class="font-mono flex items-center gap-0.5">
										<Download class="w-2.5 h-2.5 text-white/30" /> {formatDownloads(item.downloads)}
									</span>
								</div>

								<!-- Button matching Reference Image 2 rounded pill -->
								<button 
									type="button"
									class="bg-[#2b2c32] hover:bg-[#383940] active:scale-95 text-white/90 text-xs font-semibold px-3.5 py-1.5 rounded-xl flex items-center gap-1.5 transition-all shadow-sm border border-white/5 disabled:opacity-50 cursor-pointer"
									onclick={() => installItem(item)}
									disabled={isInstalling || isInstalled}
								>
									{#if isInstalling}
										<Loader2 class="w-3 h-3 animate-spin text-[#c5a880]" />
										Instalando
									{:else if isInstalled}
										<Check class="w-3 h-3 text-emerald-400" />
										Instalado
									{:else}
										<Download class="w-3 h-3 text-white/60" />
										Instalar
									{/if}
								</button>
							</div>
						</div>

					</div>
				{/each}
			</div>

			<!-- Pagination Controls matching SKlauncher -->
			{#if results.length > 0}
				<div class="flex items-center justify-between pt-2 pb-8 border-t border-white/5">
					<div class="text-xs text-white/40 font-medium">
						Mostrando página <span class="text-white font-bold">{currentPage}</span>
					</div>

					<div class="flex items-center gap-1.5">
						<button
							type="button"
							class="bg-[#24252c] hover:bg-[#32333c] disabled:opacity-30 disabled:pointer-events-none text-white text-xs font-semibold px-3.5 py-1.5 rounded-xl border border-white/5 transition-all cursor-pointer"
							onclick={() => goToPage(currentPage - 1)}
							disabled={currentPage <= 1 || loading}
						>
							Anterior
						</button>

						{#each [1, 2, 3, 4, 5] as p}
							{@const pageNum = currentPage > 3 ? currentPage - 3 + p : p}
							<button
								type="button"
								class="w-8 h-8 rounded-xl text-xs font-bold transition-all cursor-pointer border {currentPage === pageNum ? 'bg-[#caa97c] text-black border-[#caa97c] font-black shadow-md' : 'bg-[#18191c] text-white/60 border-white/5 hover:bg-[#24252c] hover:text-white'}"
								onclick={() => goToPage(pageNum)}
								disabled={loading}
							>
								{pageNum}
							</button>
						{/each}

						<button
							type="button"
							class="bg-[#24252c] hover:bg-[#32333c] disabled:opacity-30 disabled:pointer-events-none text-white text-xs font-semibold px-3.5 py-1.5 rounded-xl border border-white/5 transition-all cursor-pointer"
							onclick={() => goToPage(currentPage + 1)}
							disabled={results.length < pageSize || loading}
						>
							Próxima
						</button>
					</div>
				</div>
			{/if}
		{/if}

	</div>

	<!-- RIGHT SIDEBAR FILTER PANEL matching Reference Image 2 -->
	<aside class="w-64 shrink-0 h-full bg-[#141518] border border-white/5 rounded-3xl p-5 overflow-y-auto custom-scrollbar flex flex-col justify-between shadow-xl">
		
		<div class="space-y-5">
			<!-- Header with Sliders icon -->
			<div class="flex items-center gap-2 text-xs font-bold text-white uppercase tracking-wider">
				<SlidersHorizontal class="w-3.5 h-3.5 text-[#caa97c]" /> Filtros
			</div>

			<!-- FONTE (Dual pill toggle from Reference Image 2) -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">FONTE</div>
				<div class="flex bg-[#1c1d22] p-1 rounded-xl border border-white/5 gap-1">
					<button 
						type="button"
						class="flex-1 py-1.5 px-2 rounded-lg text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'modrinth' ? 'bg-[#2b2c32] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
						onclick={() => selectedSource = selectedSource === 'modrinth' ? 'all' : 'modrinth'}
					>
						<span class="text-emerald-400 font-bold text-[11px]">●</span> Modrinth
					</button>
					<button 
						type="button"
						class="flex-1 py-1.5 px-2 rounded-lg text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'curseforge' ? 'bg-[#2b2c32] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
						onclick={() => selectedSource = selectedSource === 'curseforge' ? 'all' : 'curseforge'}
					>
						<Flame class="w-3 h-3 text-orange-400" /> CurseForge
					</button>
				</div>
			</div>

			<!-- TIPO DE CONTEÚDO (Pills matching Reference Image 2) -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">TIPO DE CONTEÚDO</div>
				<div class="flex flex-wrap gap-1.5">
					{#each contentTypes as type}
						<button 
							type="button"
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {selectedType === type ? 'bg-[#c5a880]/15 text-[#caa97c] border border-[#c5a880]/35 shadow-sm' : 'bg-[#1c1d22] text-white/60 border border-white/5 hover:border-white/20 hover:text-white'}"
							onclick={() => selectedType = type}
						>
							{type}
						</button>
					{/each}
				</div>
			</div>

			<!-- VERSÃO DO JOGO (Select dropdown) -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">VERSÃO DO JOGO</div>
				<select 
					bind:value={selectedVersion}
					class="w-full bg-[#1c1d22] border border-white/10 rounded-xl px-3 py-2 text-xs font-bold text-white focus:outline-none focus:border-[#c5a880] cursor-pointer"
				>
					{#each mcVersions as ver}
						<option>{ver}</option>
					{/each}
				</select>
			</div>

			<!-- MOD LOADERS (Pills with colors matching Image 2) -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">MOD LOADERS</div>
				<div class="flex flex-wrap gap-1.5">
					{#each modLoaders as loader}
						<button 
							type="button"
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer border {selectedLoader === loader.name ? `${loader.color} shadow-sm` : 'bg-[#1c1d22] text-white/60 border-white/5 hover:border-white/20 hover:text-white'}"
							onclick={() => selectedLoader = selectedLoader === loader.name ? null : loader.name}
						>
							{loader.name}
						</button>
					{/each}
				</div>
			</div>

			<!-- CATEGORIAS (Tag cloud pills) -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">CATEGORIAS</div>
				<div class="flex flex-wrap gap-1.5 max-h-40 overflow-y-auto custom-scrollbar pr-1">
					{#each categories as cat}
						<button 
							type="button"
							class="px-3 py-1 rounded-full text-[11px] font-medium transition-all cursor-pointer {selectedCategory === cat ? 'bg-white/20 text-white border border-white/30' : 'bg-[#1c1d22] text-white/50 border border-white/5 hover:border-white/20 hover:text-white'}"
							onclick={() => selectedCategory = selectedCategory === cat ? null : cat}
						>
							{cat}
						</button>
					{/each}
				</div>
			</div>
		</div>

		<!-- Footer Social Links matching bottom right of Image 2 -->
		<div class="pt-4 border-t border-white/5 flex items-center justify-center gap-4 text-white/30 text-xs">
			<a href="https://discord.com" target="_blank" class="hover:text-white transition-colors">Discord</a>
			<span>•</span>
			<a href="https://x.com" target="_blank" class="hover:text-white transition-colors">X</a>
			<span>•</span>
			<a href="https://youtube.com" target="_blank" class="hover:text-white transition-colors">YouTube</a>
		</div>

	</aside>

</div>
