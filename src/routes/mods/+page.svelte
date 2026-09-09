<script lang="ts">
	import { Search, Filter, Download, Star, ArrowLeft, ArrowRight, Zap, Boxes, Compass } from "lucide-svelte";
	import { fly, fade, slide } from "svelte/transition";
	import { backOut } from "svelte/easing";
	
	let searchQuery = $state("");
	let selectedCategory = $state("Todos");
	
	const categories = ["Todos", "Otimização", "Aventura", "RPG", "Tecnologia", "Magia", "Hardcore"];
	
	const modpacks = [
		{ id: 1, name: "Fabulously Optimized", author: "robotkoer", img: "https://images.unsplash.com/photo-1627856013091-fed6e4e30025?w=500", desc: "Alta fidelidade e extremo FPS.", downloads: "12M", rating: "4.9", loader: "Fabric" },
		{ id: 2, name: "Cobblemon Official", author: "Cobblemon", img: "https://images.unsplash.com/photo-1613336026275-d6d473084e85?w=500", desc: "O melhor modpack de Pokémon.", downloads: "8M", rating: "4.8", loader: "Fabric" },
		{ id: 3, name: "Better MC [FORGE]", author: "SHXRKIE", img: "https://images.unsplash.com/photo-1605806616949-1e87b487cb2a?w=500", desc: "A experiência definitiva do Minecraft.", downloads: "15M", rating: "4.7", loader: "Forge" },
		{ id: 4, name: "All the Mods 9", author: "ATMTeam", img: "https://images.unsplash.com/photo-1542751371-adc38448a05e?w=500", desc: "Tudo o que você pode imaginar.", downloads: "5M", rating: "4.9", loader: "Forge" },
		{ id: 5, name: "RLCraft", author: "Shivaxi", img: "https://images.unsplash.com/photo-1518709268805-4e9042af9f23?w=500", desc: "O modpack mais difícil do mundo.", downloads: "20M", rating: "4.6", loader: "Forge" },
		{ id: 6, name: "Vault Hunters", author: "Iskall85", img: "https://images.unsplash.com/photo-1511512578047-dfb367046420?w=500", desc: "Incursões roguelike em cofres.", downloads: "3M", rating: "4.8", loader: "Forge" },
	];
</script>

<div class="h-full flex flex-col pt-4">
	<!-- Top Header -->
	<header class="flex items-center justify-between mb-8" in:fly={{ y: -20, duration: 500, easing: backOut }}>
		<div>
			<h1 class="text-2xl font-black text-white flex items-center gap-3">
				<Compass class="w-7 h-7 text-brand-500" /> Explorar Modpacks
			</h1>
			<p class="text-sm text-white/50 mt-1">Descubra milhares de modpacks e instale com 1 clique.</p>
		</div>
		
		<!-- Search Bar -->
		<div class="relative w-72">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
			<input 
				type="text" 
				bind:value={searchQuery}
				placeholder="Buscar modpacks..."
				class="w-full bg-[#18191c]/80 border border-white/10 rounded-full py-2.5 pl-10 pr-4 text-sm text-white placeholder-white/30 focus:outline-none focus:border-brand-500/50 focus:ring-1 focus:ring-brand-500/50 transition-all shadow-inner"
			/>
		</div>
	</header>

	<!-- Categories Pill Menu -->
	<div class="flex items-center gap-2 mb-8 overflow-x-auto custom-scrollbar pb-2" in:fade={{ duration: 400, delay: 100 }}>
		{#each categories as cat}
			<button 
				class="px-4 py-1.5 rounded-full text-xs font-bold whitespace-nowrap transition-all duration-300 active:scale-95 {selectedCategory === cat ? 'bg-brand-500 text-black shadow-[0_0_15px_rgba(226,184,107,0.4)]' : 'bg-[#18191c] text-white/60 border border-white/5 hover:border-white/20 hover:text-white'}"
				onclick={() => selectedCategory = cat}
			>
				{cat}
			</button>
		{/each}
	</div>

	<!-- Modpack Grid -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 pb-10">
		{#each modpacks.filter(m => m.name.toLowerCase().includes(searchQuery.toLowerCase())) as pack, i}
			<div 
				in:fly={{ y: 30, duration: 500, delay: i * 50, easing: backOut }}
				class="group relative bg-[#18191c]/80 backdrop-blur-md rounded-3xl border border-white/5 overflow-hidden hover:border-brand-500/30 transition-all duration-500 hover:-translate-y-2 hover:shadow-[0_20px_40px_-15px_rgba(226,184,107,0.2)] flex flex-col h-72 cursor-pointer"
			>
				<!-- Banner Image -->
				<div class="h-32 w-full relative overflow-hidden">
					<img src={pack.img} alt={pack.name} class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110 opacity-70 group-hover:opacity-100" />
					<div class="absolute inset-0 bg-gradient-to-t from-[#18191c]/90 to-transparent"></div>
					<!-- Loader Badge -->
					<div class="absolute top-3 right-3 bg-black/60 backdrop-blur-md border border-white/10 px-2.5 py-1 rounded-full text-[9px] font-black uppercase text-white/80 tracking-wider">
						{pack.loader}
					</div>
				</div>
				
				<!-- Content -->
				<div class="p-4 flex-1 flex flex-col">
					<h3 class="font-extrabold text-white text-lg leading-tight group-hover:text-brand-500 transition-colors">{pack.name}</h3>
					<p class="text-[10px] text-brand-500/80 font-bold uppercase tracking-widest mt-1">Por {pack.author}</p>
					
					<p class="text-xs text-white/50 mt-3 line-clamp-2 leading-relaxed">{pack.desc}</p>
					
					<div class="mt-auto pt-4 flex items-center justify-between">
						<div class="flex items-center gap-3">
							<span class="text-[10px] font-bold text-white/40 flex items-center gap-1.5"><Download class="w-3.5 h-3.5 text-white/60" /> {pack.downloads}</span>
							<span class="text-[10px] font-bold text-amber-400 flex items-center gap-1.5"><Star class="w-3.5 h-3.5" fill="currentColor" /> {pack.rating}</span>
						</div>
						
						<!-- One-Click Install Button -->
						<button class="bg-brand-500 hover:bg-brand-400 text-black p-2 rounded-xl transition-all duration-300 active:scale-90 hover:shadow-[0_0_15px_rgba(226,184,107,0.5)]">
							<Download class="w-4 h-4" />
						</button>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
