<script lang="ts">
	import { ChevronDown, Filter, Box, Cpu, Palette, Sparkles, HardDrive, Globe, Flame } from "lucide-svelte";
	import { profiles } from "$lib/stores/profiles.svelte";

	let {
		selectedSource = $bindable("all"),
		selectedType = $bindable("Modpack"),
		selectedVersion = $bindable("Qualquer Versão"),
		selectedLoader = $bindable(null),
		selectedCategory = $bindable(null)
	}: {
		selectedSource: "all" | "modrinth" | "curseforge";
		selectedType: string;
		selectedVersion: string;
		selectedLoader: string | null;
		selectedCategory: string | null;
	} = $props();

	const modLoaders = [
		{ name: "Fabric", color: "text-cyan-400 border-cyan-500/30 bg-cyan-500/10" },
		{ name: "Quilt", color: "text-purple-400 border-purple-500/30 bg-purple-500/10" },
		{ name: "Forge", color: "text-amber-300 border-amber-500/30 bg-amber-500/15" },
		{ name: "NeoForge", color: "text-orange-400 border-orange-500/30 bg-orange-500/10" },
	];

	const categories = [
		"Adventure", "Challenging", "Combat", "Kitchen Sink",
		"Lightweight", "Magic", "Multiplayer", "Optimization",
		"Quests", "Technology"
	];

	const mcVersions = ["Qualquer Versão", "1.21.4", "1.21.3", "1.21.1", "1.20.6", "1.20.4", "1.20.1", "1.19.4", "1.19.2", "1.18.2", "1.16.5", "1.12.2", "1.8.9"];

	const contentTypeItems = [
		{ id: "Modpack", label: "Modpack", icon: Box },
		{ id: "Mod", label: "Mod", icon: Cpu },
		{ id: "Resource Pack", label: "Resource Pack", icon: Palette },
		{ id: "Shader", label: "Shader", icon: Sparkles },
		{ id: "Data Pack", label: "Data Pack", icon: HardDrive },
		{ id: "World", label: "World", icon: Globe }
	];
</script>

<aside class="w-64 shrink-0 h-full bg-bg-elevated border border-fg/5 rounded-3xl p-5 overflow-y-auto custom-scrollbar flex flex-col justify-between shadow-2xl">
	<div class="space-y-5">
		<div class="flex items-center gap-2 text-xs font-bold text-fg uppercase tracking-wider">
			<Filter class="w-3.5 h-3.5 text-[#1bd96a]" /> Filtros
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg/40 uppercase tracking-wider mb-2">FONTE</div>
			<div class="grid grid-cols-2 bg-bg/40 p-1 rounded-xl border border-fg/5 gap-1">
				<button
					type="button"
					class="col-span-2 py-2 px-2 rounded-xl text-xs font-semibold transition-all cursor-pointer flex items-center justify-center {selectedSource === 'all' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'all'}
				>
					Todos
				</button>
				<button
					type="button"
					class="py-2 px-2 rounded-xl text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'modrinth' ? 'bg-[#1bd96a]/15 text-[#1bd96a] shadow-sm border border-[#1bd96a]/30' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'modrinth'}
				>
					<span class="text-[#1bd96a] font-black text-xs">m</span> Modrinth
				</button>
				<button
					type="button"
					class="py-2 px-2 rounded-xl text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5 {selectedSource === 'curseforge' ? 'bg-bg-subtle text-fg shadow-sm border border-fg/10' : 'text-fg/50 hover:text-fg'}"
					onclick={() => selectedSource = 'curseforge'}
				>
					<Flame class="w-3.5 h-3.5 text-orange-400" /> CurseForge
				</button>
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg/40 uppercase tracking-wider mb-2">TIPO DE CONTEÚDO</div>
			<div class="grid grid-cols-2 gap-2">
				{#each contentTypeItems as item}
					<button
						type="button"
						class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-all cursor-pointer flex items-center gap-2 {selectedType === item.id ? 'bg-[#1bd96a] text-[#090a0f] font-black shadow-md' : 'bg-bg-elevated text-fg/60 border border-fg/5 hover:bg-bg-subtle hover:text-fg'}"
						onclick={() => selectedType = item.id}
					>
						<item.icon class="w-3.5 h-3.5 shrink-0" />
						<span class="truncate">{item.label}</span>
					</button>
				{/each}
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg/40 uppercase tracking-wider mb-2">VERSÃO DO JOGO</div>
			<div class="relative">
				<select
					bind:value={selectedVersion}
					class="w-full bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs font-semibold text-fg focus:outline-none focus:border-brand-500 appearance-none cursor-pointer pr-8"
				>
					{#each mcVersions as ver}
						<option value={ver}>{ver}</option>
					{/each}
				</select>
				<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-fg/40 pointer-events-none" />
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg/40 uppercase tracking-wider mb-2">MOD LOADERS</div>
			<div class="grid grid-cols-2 gap-2">
				{#each modLoaders as loader}
					<button
						type="button"
						class="px-2.5 py-2 rounded-xl text-xs font-semibold transition-all cursor-pointer border flex items-center gap-1.5 {selectedLoader === loader.name ? `${loader.color} font-bold shadow-sm` : 'bg-bg-elevated text-fg/60 border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedLoader = selectedLoader === loader.name ? null : loader.name}
					>
						<span class="w-1.5 h-1.5 rounded-full bg-current"></span>
						<span class="truncate">{loader.name}</span>
					</button>
				{/each}
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg/40 uppercase tracking-wider mb-2">CATEGORIAS</div>
			<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto custom-scrollbar pr-1">
				{#each categories as cat}
					<button
						type="button"
						class="px-2.5 py-1 rounded-full text-[11px] font-medium transition-all cursor-pointer {selectedCategory === cat ? 'bg-[#1bd96a]/20 text-[#1bd96a] border border-[#1bd96a]/40 font-bold' : 'bg-bg-elevated text-fg/50 border border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedCategory = selectedCategory === cat ? null : cat}
					>
						{cat}
					</button>
				{/each}
			</div>
		</div>
	</div>

	<div class="pt-4 border-t border-fg/5 flex items-center justify-center gap-4 text-fg/30 text-xs">
		<a href="https://discord.com" target="_blank" class="hover:text-fg transition-colors" title="Discord">Discord</a>
		<span>•</span>
		<a href="https://x.com" target="_blank" class="hover:text-fg transition-colors" title="X">X</a>
		<span>•</span>
		<a href="https://youtube.com" target="_blank" class="hover:text-fg transition-colors" title="YouTube">YouTube</a>
	</div>
</aside>
