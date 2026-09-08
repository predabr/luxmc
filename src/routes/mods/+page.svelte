<script lang="ts">
	import { fade } from "svelte/transition";
	import { 
		Search, 
		Package, 
		Layers, 
		Sparkles, 
		Box, 
		Code, 
		Filter, 
		Check, 
		ArrowRight, 
		Clock, 
		Flame, 
		Cpu, 
		ExternalLink,
		Sliders,
		AlertCircle
	} from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	let search = $state("");
	let source = $state<"modrinth" | "curseforge">("modrinth");
	let contentType = $state<"modpack" | "mod" | "resourcepack" | "shader">("modpack");
	let selectedLoader = $state<"all" | "fabric" | "forge" | "neoforge" | "quilt">("all");
	let selectedVersion = $state("1.20.4");
	let notifyOnLaunch = $state(false);

	const plannedCategories = [
		{ name: "Otimização & FPS", count: "Sodium, Lithium, Iris", icon: Cpu },
		{ name: "Aventura & Exploração", count: "Biomes O' Plenty, Alex's Mobs", icon: Sparkles },
		{ name: "Tecnologia & Automação", count: "Create, Applied Energistics", icon: Sliders },
		{ name: "Shaders & Gráficos", count: "Complementary, BSL, Kappa", icon: Layers }
	];
</script>

<div class="flex gap-6 h-full w-full select-none" in:fade={{ duration: 250 }}>
	
	<!-- Main Center Area: Architecture Ready for Mod Ecosystem -->
	<div class="flex-1 flex flex-col min-w-0 h-full overflow-y-auto custom-scrollbar pr-2 space-y-6">
		
		<!-- Header -->
		<div class="flex items-center justify-between mt-1">
			<div>
				<div class="flex items-center gap-3">
					<h1 class="text-3xl font-extrabold text-white tracking-tight">Central de Conteúdo</h1>
					<span class="bg-brand-500/15 text-brand-500 text-[10px] font-bold px-2.5 py-0.5 rounded-lg border border-brand-500/30">
						Em Breve
					</span>
				</div>
				<p class="text-white/50 text-xs mt-1">Navegue e instale mods, shaders e pacotes direto no Luxmc</p>
			</div>

			<!-- Source Switcher -->
			<div class="flex bg-[#18191c] border border-white/10 p-1 rounded-2xl">
				<button 
					class="px-4 py-1.5 rounded-xl text-xs font-bold transition-all {source === 'modrinth' ? 'bg-[#1bd96a]/20 text-[#1bd96a] border border-[#1bd96a]/30 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => source = 'modrinth'}
				>
					Modrinth
				</button>
				<button 
					class="px-4 py-1.5 rounded-xl text-xs font-bold transition-all {source === 'curseforge' ? 'bg-[#f16436]/20 text-[#f16436] border border-[#f16436]/30 shadow-sm' : 'text-white/40 hover:text-white'}"
					onclick={() => source = 'curseforge'}
				>
					CurseForge
				</button>
			</div>
		</div>

		<!-- Search & Type Filter Bar -->
		<div class="flex flex-col sm:flex-row items-center gap-3">
			<div class="relative flex-1 w-full">
				<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
				<input 
					type="text" 
					placeholder="Pesquisar pacotes de mods, shaders e pacotes de recursos..." 
					bind:value={search}
					class="w-full bg-[#18191c] border border-white/10 rounded-2xl py-3 pl-10 pr-4 text-xs text-white placeholder-white/40 focus:border-brand-500 focus:outline-none transition-all shadow-inner"
				/>
			</div>

			<div class="flex bg-[#18191c] border border-white/10 rounded-2xl p-1 gap-1 w-full sm:w-auto overflow-x-auto">
				<button 
					class="px-3.5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-1.5 shrink-0 {contentType === 'modpack' ? 'bg-[#25262c] text-brand-500 border border-brand-500/30' : 'text-white/40 hover:text-white'}"
					onclick={() => contentType = 'modpack'}
				>
					<Layers class="w-3.5 h-3.5" /> Modpacks
				</button>
				<button 
					class="px-3.5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-1.5 shrink-0 {contentType === 'mod' ? 'bg-[#25262c] text-brand-500 border border-brand-500/30' : 'text-white/40 hover:text-white'}"
					onclick={() => contentType = 'mod'}
				>
					<Package class="w-3.5 h-3.5" /> Mods
				</button>
				<button 
					class="px-3.5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-1.5 shrink-0 {contentType === 'shader' ? 'bg-[#25262c] text-brand-500 border border-brand-500/30' : 'text-white/40 hover:text-white'}"
					onclick={() => contentType = 'shader'}
				>
					<Sparkles class="w-3.5 h-3.5" /> Shaders
				</button>
				<button 
					class="px-3.5 py-2 rounded-xl text-xs font-bold transition-all flex items-center gap-1.5 shrink-0 {contentType === 'resourcepack' ? 'bg-[#25262c] text-brand-500 border border-brand-500/30' : 'text-white/40 hover:text-white'}"
					onclick={() => contentType = 'resourcepack'}
				>
					<Box class="w-3.5 h-3.5" /> Recursos
				</button>
			</div>
		</div>

		<!-- Elegant Prepared State: Architecture Ready for Mod Support -->
		<div class="bg-gradient-to-b from-[#18191c] to-[#121316] border border-white/10 rounded-3xl p-10 flex flex-col items-center justify-center text-center shadow-xl relative overflow-hidden">
			
			<div class="h-20 w-20 rounded-3xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center mb-5 text-brand-500 shadow-[0_0_30px_rgba(226,184,107,0.15)]">
				<Package class="w-10 h-10 animate-bounce" />
			</div>

			<h2 class="text-2xl font-black text-white tracking-tight">Suporte a Mods & Modpacks em Construção</h2>
			<p class="text-white/60 text-xs max-w-lg mt-2 leading-relaxed">
				Toda a arquitetura de busca e integração com a API do <strong>Modrinth</strong> e <strong>CurseForge</strong> já está estruturada no Luxmc. No momento atual o launcher suporta Vanilla com foco em estabilidade e na próxima atualização o instalador automático em 1-clique estará liberado!
			</p>

			<!-- Category Preview Badges -->
			<div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-8 w-full max-w-2xl text-left">
				{#each plannedCategories as cat}
					<div class="bg-[#1e1f24] border border-white/5 p-4 rounded-2xl flex items-center gap-4">
						<div class="h-11 w-11 rounded-xl bg-white/5 flex items-center justify-center text-brand-500">
							<cat.icon class="w-5 h-5" />
						</div>
						<div>
							<h4 class="text-xs font-bold text-white">{cat.name}</h4>
							<p class="text-[11px] text-white/40 mt-0.5">{cat.count}</p>
						</div>
					</div>
				{/each}
			</div>

			<!-- Action notification button -->
			<button 
				class="mt-8 bg-brand-500 hover:bg-[#ebd095] text-black font-black text-xs px-7 py-3 rounded-2xl transition-all shadow-lg flex items-center gap-2 cursor-pointer active:scale-95"
				onclick={() => {
					notifyOnLaunch = !notifyOnLaunch;
					toast(notifyOnLaunch ? "Você será notificado assim que o instalador de mods for ativado!" : "Notificação desativada", "info");
				}}
			>
				{#if notifyOnLaunch}
					<Check class="w-4 h-4 stroke-[3]" /> Notificação Ativada
				{:else}
					<Sparkles class="w-4 h-4" /> Ativar Alerta de Lançamento da API
				{/if}
			</button>

		</div>

	</div>

	<!-- Right Filters Sidebar -->
	<aside class="w-[260px] shrink-0 h-full flex flex-col gap-5 overflow-y-auto custom-scrollbar pb-2 select-none border-l border-white/5 pl-4">
		<div>
			<h3 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Versão do Jogo</h3>
			<select 
				bind:value={selectedVersion} 
				class="w-full bg-[#18191c] border border-white/10 rounded-xl px-3 py-2.5 text-xs text-white focus:outline-none font-bold"
			>
				<option value="1.20.4">Minecraft 1.20.4 (Recomendado)</option>
				<option value="1.20.2">Minecraft 1.20.2</option>
				<option value="1.20.1">Minecraft 1.20.1</option>
				<option value="1.19.4">Minecraft 1.19.4</option>
				<option value="1.18.2">Minecraft 1.18.2</option>
				<option value="1.16.5">Minecraft 1.16.5</option>
				<option value="1.12.2">Minecraft 1.12.2</option>
				<option value="1.8.9">Minecraft 1.8.9</option>
			</select>
		</div>

		<div>
			<h3 class="text-xs font-bold text-white uppercase tracking-wider mb-3">Mod Loader</h3>
			<div class="flex flex-col gap-1.5">
				{#each [
					{ id: 'all', name: 'Todos os Loaders' },
					{ id: 'fabric', name: 'Fabric Loader' },
					{ id: 'neoforge', name: 'NeoForge' },
					{ id: 'forge', name: 'Forge clássico' },
					{ id: 'quilt', name: 'Quilt' }
				] as l}
					<button 
						class="w-full p-2.5 rounded-xl text-xs font-bold flex items-center justify-between transition-all {selectedLoader === l.id ? 'bg-[#222328] text-brand-500 border border-white/10' : 'text-white/50 hover:text-white hover:bg-white/5'}"
						onclick={() => selectedLoader = l.id as any}
					>
						<span>{l.name}</span>
						{#if selectedLoader === l.id}
							<Check class="w-3.5 h-3.5 text-brand-500" />
						{/if}
					</button>
				{/each}
			</div>
		</div>

		<div class="mt-auto bg-[#18191c] border border-white/5 p-3 rounded-2xl text-[10px] text-white/40">
			<div class="font-bold text-white mb-1">API Integrada</div>
			Conexão direta com Modrinth v2 REST API e CurseForge GraphQL.
		</div>
	</aside>

</div>
