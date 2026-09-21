<script lang="ts">
	import { Search, Check, ChevronDown, Sparkles, X, Filter } from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	type Version = {
		id: string;
		versionType: string;
		releaseTime: string;
	};

	type Props = {
		versions: Version[];
		value: string;
		loading?: boolean;
		onChange?: (id: string) => void;
	};

	let { versions = [], value = $bindable(""), loading = false, onChange }: Props = $props();

	let search = $state("");
	let typeFilter = $state<"all" | "release" | "snapshot" | "old_beta" | "old_alpha">("release");
	let isDropdownOpen = $state(false);

	const filtered = $derived(
		versions
			.filter((v) => (typeFilter === "all" ? true : v.versionType === typeFilter))
			.filter((v) =>
				search.trim() ? v.id.toLowerCase().includes(search.toLowerCase()) : true,
			)
			.toSorted((a, b) => b.releaseTime.localeCompare(a.releaseTime))
			.slice(0, 100)
	);

	function pick(id: string) {
		value = id;
		onChange?.(id);
		isDropdownOpen = false;
	}

	const filterTabs = [
		{ id: "release", label: "Releases" },
		{ id: "all", label: "Todas" },
		{ id: "snapshot", label: "Snapshots" },
		{ id: "old_beta", label: "Betas" }
	] as const;

	const popularVersions = [
		{ id: "1.21.4", label: "1.21.4", tag: "Recente" },
		{ id: "1.20.1", label: "1.20.1", tag: "Modpacks" },
		{ id: "1.16.5", label: "1.16.5", tag: "Forge" },
		{ id: "1.8.9", label: "1.8.9", tag: "PvP" }
	] as const;
</script>

<div class="space-y-2 select-none">
	<!-- Top: Search Bar & Type Filter Tabs -->
	<div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/40" />
			<input
				type="text"
				placeholder="Pesquisar versão (ex: 1.21.4, 1.20.1, 1.16.5, snapshot)..."
				bind:value={search}
				onfocus={() => (isDropdownOpen = true)}
				class="w-full h-11 pl-9 pr-9 rounded-xl bg-bg-elevated border border-fg/10 focus:border-brand-500 text-xs font-bold text-fg placeholder:text-fg/30 outline-none transition-all"
			/>
			{#if search}
				<button
					type="button"
					class="absolute right-3 top-1/2 -translate-y-1/2 text-fg/40 hover:text-fg p-0.5 rounded cursor-pointer"
					onclick={() => (search = "")}
				>
					<X class="w-3.5 h-3.5" />
				</button>
			{/if}
		</div>

		<!-- Filter Pills -->
		<div class="flex items-center gap-1 bg-bg-elevated p-1 rounded-xl border border-fg/5 shrink-0">
			{#each filterTabs as tab}
				<button
					type="button"
					class="px-2.5 py-1.5 rounded-lg text-[10px] font-extrabold uppercase transition-all cursor-pointer {typeFilter === tab.id ? 'bg-brand-500 text-brand-foreground shadow' : 'text-fg/50 hover:text-fg hover:bg-fg/5'}"
					onclick={() => { typeFilter = tab.id; isDropdownOpen = true; }}
				>
					{tab.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Quick Popular Chips -->
	<div class="flex items-center gap-1.5 flex-wrap">
		<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 flex items-center gap-1 pl-1">
			<Sparkles class="w-3 h-3 text-brand-500" /> Populares:
		</span>
		{#each popularVersions as pop}
			<button
				type="button"
				class="px-2.5 py-1 rounded-lg text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5 {value === pop.id ? 'bg-brand-500/20 text-brand-400 border border-brand-500/40 shadow-sm' : 'bg-bg-elevated border border-fg/10 text-fg/70 hover:text-fg hover:border-fg/30'}"
				onclick={() => pick(pop.id)}
			>
				<span>{pop.label}</span>
				<span class="text-[9px] font-normal text-fg/40">({pop.tag})</span>
			</button>
		{/each}
	</div>

	<!-- Selected Version Display & Selector Box -->
	<div class="rounded-2xl border border-fg/10 bg-bg-elevated p-2 space-y-2 shadow-inner">
		<div class="flex items-center justify-between px-2 py-1 text-[11px] text-fg/50">
			<div class="flex items-center gap-1.5">
				<span>Versão selecionada:</span>
				<span class="text-brand-500 font-black text-xs bg-brand-500/10 px-2 py-0.5 rounded border border-brand-500/20">
					{value || (loading ? "Carregando..." : "Nenhuma selecionada")}
				</span>
			</div>
			<span class="text-[10px] text-fg/40">
				{filtered.length} versões encontradas
			</span>
		</div>

		<!-- Versions Grid / List -->
		<div class="max-h-44 overflow-y-auto custom-scrollbar pr-1 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-1.5">
			{#if loading}
				<div class="col-span-full py-8 text-center text-xs text-fg/40 flex items-center justify-center gap-2">
					<div class="w-4 h-4 rounded-full border-2 border-brand-500 border-t-transparent animate-spin"></div>
					Carregando versões do Mojang...
				</div>
			{:else if filtered.length === 0}
				<div class="col-span-full py-8 text-center text-xs text-fg/40">
					Nenhuma versão corresponde à pesquisa "{search}".
				</div>
			{:else}
				{#each filtered as v (v.id)}
					{@const isSelected = value === v.id}
					<button
						type="button"
						class="flex items-center justify-between px-3 py-2 rounded-xl text-left transition-all cursor-pointer border {isSelected ? 'bg-brand-500 text-brand-foreground border-brand-500 font-black shadow-md scale-[1.02]' : 'bg-bg-subtle hover:bg-bg-subtle text-fg/80 hover:text-fg border-fg/5 hover:border-fg/15'}"
						onclick={() => pick(v.id)}
					>
						<div class="min-w-0">
							<span class="text-xs truncate block {isSelected ? 'font-black text-brand-foreground' : 'font-bold'}">{v.id}</span>
							<span class="text-[8px] uppercase tracking-wider block opacity-70">
								{v.versionType === 'release' ? 'Release' : v.versionType === 'snapshot' ? 'Snapshot' : 'Beta'}
							</span>
						</div>
						{#if isSelected}
							<Check class="w-3.5 h-3.5 stroke-[3] shrink-0 text-brand-foreground ml-1" />
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	</div>
</div>

