<script lang="ts">
	import { Search, Download, Star, Compass } from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	
	let searchQuery = $state("");
	let selectedCategory = $state("Todos");
	
	const categories = ["Todos", "Otimização", "Aventura & RPG", "Pokémon", "Tecnologia", "Hardcore"];
	
	const modpacks = [
		{ 
			id: "fo", 
			name: "Fabulously Optimized", 
			author: "robotkoer", 
			img: "/modpack_fo.webp", 
			icon: "/modpack_fo_icon.png",
			desc: "Pacote de alta fidelidade visual, estabilidade máxima e FPS extremo para Linux. Inclui Sodium, Iris e otimizações nativas.", 
			downloads: "16.9M", 
			rating: "4.9", 
			loader: "Fabric",
			category: "Otimização"
		},
		{ 
			id: "cobblemon", 
			name: "Cobblemon Official Modpack", 
			author: "CobbledStudios", 
			img: "/modpack_cobblemon.webp", 
			icon: "/modpack_cobblemon_icon.png",
			desc: "Capture, treine e explore um mundo vivo repleto de criaturas em estilo pixel art com suporte total a servidores.", 
			downloads: "10.4M", 
			rating: "4.8", 
			loader: "Fabric",
			category: "Pokémon"
		},
		{ 
			id: "bmc2", 
			name: "Better MC [FABRIC] - BMC2", 
			author: "SHXRKIE", 
			img: "/modpack_better_mc.webp", 
			icon: "/modpack_bmc_icon.webp",
			desc: "A experiência definitiva do Minecraft Vanilla+ com novas dimensões, chefes customizados, masmorras e centenas de estruturas.", 
			downloads: "3.3M", 
			rating: "4.7", 
			loader: "Fabric",
			category: "Aventura & RPG"
		},
		{ 
			id: "atm9", 
			name: "All the Mods 9", 
			author: "ATMTeam", 
			img: "/modpack_better_mc.webp", 
			icon: "/modpack_bmc_icon.webp",
			desc: "Coleção gigante de automação, magia, tecnologia industrial e quests completas para explorar.", 
			downloads: "5.1M", 
			rating: "4.9", 
			loader: "Forge",
			category: "Tecnologia"
		},
		{ 
			id: "rlcraft", 
			name: "RLCraft", 
			author: "Shivaxi", 
			img: "/modpack_fo.webp", 
			icon: "/modpack_fo_icon.png",
			desc: "O modpack de sobrevivência hardcore mais desafiador do Minecraft, com mecânicas de temperatura, sede e criaturas mitológicas.", 
			downloads: "20.2M", 
			rating: "4.6", 
			loader: "Forge",
			category: "Hardcore"
		}
	];

	function installModpack(name: string) {
		toast(`Instalação de "${name}" iniciada com sucesso!`, "success");
	}
</script>

<div class="h-full flex flex-col pt-2 select-none">
	<!-- Top Header -->
	<header class="flex items-center justify-between mb-6">
		<div>
			<h1 class="text-2xl font-black text-white flex items-center gap-3">
				<Compass class="w-7 h-7 text-brand-500" /> Explorar Modpacks
			</h1>
			<p class="text-xs text-white/50 mt-1">Diretório oficial de modpacks otimizados para Luxmc no Linux.</p>
		</div>
		
		<!-- Search Bar -->
		<div class="relative w-72">
			<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
			<input 
				type="text" 
				bind:value={searchQuery}
				placeholder="Buscar modpacks..."
				class="w-full bg-[#18191c] border border-white/10 rounded-full py-2 pl-10 pr-4 text-xs text-white placeholder-white/30 focus:outline-none focus:border-brand-500 transition-colors shadow-inner"
			/>
		</div>
	</header>

	<!-- Categories Pill Menu -->
	<div class="flex items-center gap-2 mb-6 overflow-x-auto custom-scrollbar pb-1">
		{#each categories as cat}
			<button 
				type="button"
				class="px-4 py-1.5 rounded-full text-xs font-bold whitespace-nowrap transition-all cursor-pointer {selectedCategory === cat ? 'bg-brand-500 text-black shadow-md' : 'bg-[#18191c] text-white/60 border border-white/5 hover:border-white/20 hover:text-white'}"
				onclick={() => selectedCategory = cat}
			>
				{cat}
			</button>
		{/each}
	</div>

	<!-- Modpack Grid -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 pb-8">
		{#each modpacks.filter(m => (selectedCategory === "Todos" || m.category === selectedCategory) && m.name.toLowerCase().includes(searchQuery.toLowerCase())) as pack}
			<div class="group relative bg-[#18191c] rounded-2xl border border-white/5 overflow-hidden hover:border-brand-500/40 transition-all shadow-md flex flex-col h-72">
				<!-- Banner Image -->
				<div class="h-32 w-full relative overflow-hidden bg-[#222328]">
					<img src={pack.img} alt={pack.name} class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105 opacity-80" />
					<div class="absolute inset-0 bg-gradient-to-t from-[#18191c] via-transparent to-transparent"></div>
					<!-- Loader Badge -->
					<div class="absolute top-3 right-3 bg-black/60 border border-white/10 px-2.5 py-0.5 rounded-full text-[9px] font-black uppercase text-white/90 tracking-wider">
						{pack.loader}
					</div>
					<!-- Icon -->
					<div class="absolute bottom-2 left-3 h-10 w-10 rounded-xl overflow-hidden bg-black/60 border border-white/10 p-0.5 shadow-lg">
						<img src={pack.icon} alt={pack.name} class="w-full h-full object-cover rounded-lg" />
					</div>
				</div>
				
				<!-- Content -->
				<div class="p-4 pt-3 flex-1 flex flex-col justify-between">
					<div>
						<h3 class="font-extrabold text-white text-sm truncate group-hover:text-brand-500 transition-colors">{pack.name}</h3>
						<p class="text-[10px] text-brand-500 font-bold uppercase tracking-wider mt-0.5">Por {pack.author}</p>
						<p class="text-xs text-white/50 mt-2 line-clamp-2 leading-relaxed">{pack.desc}</p>
					</div>
					
					<div class="pt-3 flex items-center justify-between border-t border-white/5">
						<div class="flex items-center gap-3 text-[10px] font-bold text-white/40">
							<span class="flex items-center gap-1"><Download class="w-3 h-3 text-white/50" /> {pack.downloads}</span>
							<span class="flex items-center gap-1 text-amber-400"><Star class="w-3 h-3" fill="currentColor" /> {pack.rating}</span>
						</div>
						
						<button 
							type="button"
							class="bg-brand-500 hover:bg-brand-400 text-black px-3 py-1.5 rounded-xl font-bold text-xs flex items-center gap-1.5 transition-all active:scale-95 cursor-pointer shadow-sm"
							onclick={() => installModpack(pack.name)}
						>
							<Download class="w-3.5 h-3.5" /> Instalar
						</button>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
