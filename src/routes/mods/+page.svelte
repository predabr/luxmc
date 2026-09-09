<script lang="ts">
	import { Search, Download, Star, SlidersHorizontal, Layers, Check, LayoutGrid, List } from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	
	let searchQuery = $state("");
	let selectedSource = $state<"Modrinth" | "CurseForge">("Modrinth");
	let selectedType = $state("Modpack");
	let selectedLoader = $state<string | null>(null);
	let selectedCategory = $state<string | null>(null);
	let selectedVersion = $state("Qualquer Versão");
	let viewMode = $state<"grid" | "list">("grid");
	
	const contentTypes = ["Modpack", "Mod", "Resource Pack", "Shader", "Data Pack", "World"];
	const modLoaders = ["Fabric", "Quilt", "Forge", "NeoForge"];
	const categories = [
		"Adventure", "Challenging", "Combat", "Kitchen Sink", 
		"Lightweight", "Magic", "Multiplayer", "Optimization", 
		"Quests", "Technology"
	];
	
	const items = [
		{ 
			id: "fo", 
			name: "Fabulously Optimized", 
			author: "robotkoer", 
			img: "/modpack_fo.webp", 
			icon: "/modpack_fo_icon.png",
			desc: "Beautiful graphics, speedy performance and familiar features in a simple package. Chaos Cubed beta!", 
			downloads: "16.9M", 
			type: "Modpack",
			loader: "Fabric",
			category: "Optimization"
		},
		{ 
			id: "zombie100", 
			name: "Zombie Invade 100 Days", 
			author: "FlameFire", 
			img: "https://images.unsplash.com/photo-1542751371-adc38448a05e?w=600&auto=format&fit=crop&q=80", 
			icon: "/modpack_bmc_icon.webp",
			desc: "Same as Forge Labs 100 Days Zombie Apocalypse in new Minecraft.", 
			downloads: "14.1M", 
			type: "Modpack",
			loader: "Forge",
			category: "Challenging"
		},
		{ 
			id: "cobblemon", 
			name: "Cobblemon Official Modpack [Fabric]", 
			author: "CobbledStudios", 
			img: "/modpack_cobblemon.webp", 
			icon: "/modpack_cobblemon_icon.png",
			desc: "The official modpack of the Cobblemon mod, for Fabric!", 
			downloads: "10.4M", 
			type: "Modpack",
			loader: "Fabric",
			category: "Adventure"
		},
		{ 
			id: "cobbleverse", 
			name: "COBBLEVERSE - Pokemon Adventure", 
			author: "LUMYVERSE", 
			img: "https://images.unsplash.com/photo-1613336026275-d6d473084e85?w=600&auto=format&fit=crop&q=80", 
			icon: "/modpack_cobblemon_icon.png",
			desc: "Start a true Pokémon adventure in Minecraft: Cobblemon 1.7.3! ALL 1025 Pokémon! Mega Evolutions | Gyms & Badges.", 
			downloads: "6.5M", 
			type: "Modpack",
			loader: "Fabric",
			category: "Adventure"
		},
		{ 
			id: "bmc2", 
			name: "Better MC [FABRIC] - BMC2", 
			author: "SHXRKIE", 
			img: "/modpack_better_mc.webp", 
			icon: "/modpack_bmc_icon.webp",
			desc: "Version 1.20 | A Proper Vanilla+ Modpack | Don't play Vanilla play this!", 
			downloads: "3.4M", 
			type: "Modpack",
			loader: "Fabric",
			category: "Adventure"
		},
		{ 
			id: "tacz", 
			name: "BattleArmory TACZ", 
			author: "JZ_zhenmeng", 
			img: "https://images.unsplash.com/photo-1605806616949-1e87b487cb2a?w=600&auto=format&fit=crop&q=80", 
			icon: "/modpack_bmc_icon.webp",
			desc: "TACZ Modern Combat Pack with realistic weapons, optics and attachments.", 
			downloads: "3.2M", 
			type: "Modpack",
			loader: "Forge",
			category: "Combat"
		},
		{ 
			id: "sodiumplus", 
			name: "Sodium Plus", 
			author: "FlashDev", 
			img: "/modpack_fo.webp", 
			icon: "/modpack_fo_icon.png",
			desc: "Extra shaders, fast rendering pipeline, zero screen tearing and optimized memory management.", 
			downloads: "2.6M", 
			type: "Modpack",
			loader: "Fabric",
			category: "Optimization"
		},
		{ 
			id: "vanilla_perfected", 
			name: "Vanilla Perfected", 
			author: "VP_Team", 
			img: "/modpack_better_mc.webp", 
			icon: "/modpack_bmc_icon.webp",
			desc: "Pure Vanilla experience enhanced with smooth animations, QoL features and performance boosts.", 
			downloads: "2.4M", 
			type: "Modpack",
			loader: "Quilt",
			category: "Lightweight"
		}
	];

	function installItem(name: string) {
		toast(`Instalação de "${name}" iniciada com sucesso!`, "success");
	}
</script>

<div class="h-full flex gap-6 select-none overflow-hidden">
	
	<!-- LEFT MAIN CONTENT -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-1">
		
		<!-- Header -->
		<header class="mb-6">
			<h1 class="text-2xl font-bold text-white tracking-tight">Central de Conteúdo</h1>
			<p class="text-xs text-white/40 mt-1 font-medium">Encontre e instale modpacks, mods e recursos incríveis</p>
		</header>

		<!-- Search Bar -->
		<div class="relative w-full mb-6">
			<Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-white/30" />
			<input 
				type="text" 
				bind:value={searchQuery}
				placeholder="Search modpack..."
				class="w-full bg-[#18191c] border border-white/10 rounded-2xl py-3 pl-11 pr-4 text-xs text-white placeholder-white/30 focus:outline-none focus:border-brand-500/50 transition-colors shadow-inner"
			/>
		</div>

		<!-- Bar Header (Results count, Sort, View mode) -->
		<div class="flex items-center justify-between mb-4">
			<div class="flex items-center gap-2">
				<span class="text-xs font-extrabold text-white">{selectedType}</span>
				<span class="text-[10px] text-white/40 font-mono font-medium">(18.301 resultados)</span>
			</div>

			<div class="flex items-center gap-3">
				<!-- Sort Dropdown -->
				<div class="bg-[#18191c] border border-white/10 px-3 py-1.5 rounded-xl text-xs font-bold text-white/70 flex items-center gap-2 cursor-pointer hover:border-white/20 transition-all">
					<SlidersHorizontal class="w-3.5 h-3.5 text-white/50" />
					<span>Downloads</span>
				</div>

				<!-- View Mode Toggle -->
				<div class="flex bg-[#18191c] border border-white/10 rounded-xl p-1 gap-1">
					<button 
						type="button"
						class="p-1 rounded-lg text-white/50 hover:text-white transition-all {viewMode === 'grid' ? 'bg-white/10 text-white' : ''}"
						onclick={() => viewMode = 'grid'}
					>
						<LayoutGrid class="w-3.5 h-3.5" />
					</button>
					<button 
						type="button"
						class="p-1 rounded-lg text-white/50 hover:text-white transition-all {viewMode === 'list' ? 'bg-white/10 text-white' : ''}"
						onclick={() => viewMode = 'list'}
					>
						<List class="w-3.5 h-3.5" />
					</button>
				</div>
			</div>
		</div>

		<!-- Items Grid -->
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pb-8">
			{#each items.filter(m => 
				m.name.toLowerCase().includes(searchQuery.toLowerCase()) &&
				(selectedLoader === null || m.loader === selectedLoader) &&
				(selectedCategory === null || m.category === selectedCategory)
			) as item}
				<div class="group bg-[#18191c] border border-white/5 rounded-2xl overflow-hidden hover:border-white/20 transition-all shadow-md flex flex-col justify-between">
					
					<!-- Header Banner Image -->
					<div class="h-32 w-full relative bg-[#222328] overflow-hidden">
						<img src={item.img} alt={item.name} class="w-full h-full object-cover opacity-80 group-hover:scale-105 transition-transform duration-500" />
						<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-transparent"></div>
						
						<!-- Source Badge Icon (Modrinth/CurseForge) -->
						<div class="absolute top-2.5 right-2.5 h-6 w-6 rounded-full bg-black/60 backdrop-blur-md border border-white/10 flex items-center justify-center text-emerald-400 font-bold text-[10px]" title="Modrinth">
							<span class="font-black text-[9px] text-white">m</span>
						</div>
					</div>

					<!-- Details -->
					<div class="p-4 pt-3 flex-1 flex flex-col justify-between">
						<div>
							<h3 class="font-extrabold text-white text-xs truncate group-hover:text-brand-500 transition-colors">{item.name}</h3>
							<p class="text-[10px] text-white/40 mt-1 line-clamp-2 leading-relaxed">{item.desc}</p>
						</div>

						<div class="flex items-center justify-between mt-4 pt-2 border-t border-white/5">
							<div class="text-[10px] text-white/40 font-medium">
								<span class="text-white/60 font-bold">👤 {item.author}</span>
								<span class="ml-2 font-mono">📥 {item.downloads}</span>
							</div>

							<button 
								type="button"
								class="bg-[#2d2e34] hover:bg-brand-500 hover:text-black text-white px-3 py-1.5 rounded-xl font-extrabold text-xs transition-all active:scale-95 cursor-pointer shadow-sm"
								onclick={() => installItem(item.name)}
							>
								Instalar
							</button>
						</div>
					</div>

				</div>
			{/each}
		</div>

	</div>

	<!-- RIGHT SIDEBAR FILTER PANEL (Matches SKlauncher exact design) -->
	<aside class="w-64 shrink-0 h-full bg-[#141518] border border-white/5 rounded-3xl p-5 overflow-y-auto custom-scrollbar flex flex-col justify-between">
		
		<div class="space-y-6">
			<!-- Header -->
			<div class="flex items-center gap-2 text-xs font-bold text-white uppercase tracking-wider">
				<SlidersHorizontal class="w-4 h-4 text-brand-500" /> Filtros
			</div>

			<!-- FONTE -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">FONTE</div>
				<div class="flex bg-[#1c1d22] p-1 rounded-xl border border-white/5 gap-1">
					<button 
						type="button"
						class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedSource === 'Modrinth' ? 'bg-[#2b2c32] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
						onclick={() => selectedSource = 'Modrinth'}
					>
						Modrinth
					</button>
					<button 
						type="button"
						class="flex-1 py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer {selectedSource === 'CurseForge' ? 'bg-[#2b2c32] text-white shadow-sm' : 'text-white/40 hover:text-white'}"
						onclick={() => selectedSource = 'CurseForge'}
					>
						CurseForge
					</button>
				</div>
			</div>

			<!-- TIPO DE CONTEÚDO -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">TIPO DE CONTEÚDO</div>
				<div class="flex flex-wrap gap-1.5">
					{#each contentTypes as type}
						<button 
							type="button"
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {selectedType === type ? 'bg-brand-500/20 text-brand-500 border border-brand-500/30' : 'bg-[#1c1d22] text-white/60 border border-white/5 hover:border-white/20 hover:text-white'}"
							onclick={() => selectedType = type}
						>
							{type}
						</button>
					{/each}
				</div>
			</div>

			<!-- VERSÃO DO JOGO -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">VERSÃO DO JOGO</div>
				<select 
					bind:value={selectedVersion}
					class="w-full bg-[#1c1d22] border border-white/10 rounded-xl px-3 py-2 text-xs font-bold text-white focus:outline-none focus:border-brand-500 cursor-pointer"
				>
					<option>Qualquer Versão</option>
					<option>1.21.4</option>
					<option>1.20.4</option>
					<option>1.16.5</option>
					<option>1.12.2</option>
					<option>1.8.9</option>
				</select>
			</div>

			<!-- MOD LOADERS -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">MOD LOADERS</div>
				<div class="flex flex-wrap gap-1.5">
					{#each modLoaders as loader}
						<button 
							type="button"
							class="px-3 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer {selectedLoader === loader ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' : 'bg-[#1c1d22] text-white/60 border border-white/5 hover:border-white/20 hover:text-white'}"
							onclick={() => selectedLoader = selectedLoader === loader ? null : loader}
						>
							{loader}
						</button>
					{/each}
				</div>
			</div>

			<!-- CATEGORIAS -->
			<div>
				<div class="text-[10px] font-bold text-white/40 uppercase tracking-widest mb-2">CATEGORIAS</div>
				<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto custom-scrollbar pr-1">
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

		<!-- Footer Social Links -->
		<div class="pt-4 border-t border-white/5 flex items-center justify-center gap-4 text-white/30 text-xs">
			<a href="https://discord.com" target="_blank" class="hover:text-white transition-colors">Discord</a>
			<span>•</span>
			<a href="https://x.com" target="_blank" class="hover:text-white transition-colors">X</a>
			<span>•</span>
			<a href="https://youtube.com" target="_blank" class="hover:text-white transition-colors">YouTube</a>
		</div>

	</aside>

</div>
