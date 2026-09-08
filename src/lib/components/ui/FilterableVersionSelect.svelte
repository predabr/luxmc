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
</script>

<div class="space-y-2 select-none">
	<!-- Top: Search Bar & Type Filter Tabs -->
	<div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
			<input
				type="text"
				placeholder="Pesquisar versão (ex: 1.21.4, 1.20.1, 1.16.5, snapshot)..."
				bind:value={search}
				onfocus={() => (isDropdownOpen = true)}
				class="w-full h-11 pl-9 pr-9 rounded-xl bg-[#18191c] border border-white/10 focus:border-brand-500 text-xs font-bold text-white placeholder:text-white/30 outline-none transition-all"
			/>
			{#if search}
				<button
					type="button"
					class="absolute right-3 top-1/2 -translate-y-1/2 text-white/40 hover:text-white p-0.5 rounded cursor-pointer"
					onclick={() => (search = "")}
				>
					<X class="w-3.5 h-3.5" />
				</button>
			{/if}
		</div>

		<!-- Filter Pills -->
		<div class="flex items-center gap-1 bg-[#18191c] p-1 rounded-xl border border-white/5 shrink-0">
			{#each filterTabs as tab}
				<button
					type="button"
					class="px-2.5 py-1.5 rounded-lg text-[10px] font-extrabold uppercase transition-all cursor-pointer {typeFilter === tab.id ? 'bg-brand-500 text-black shadow' : 'text-white/50 hover:text-white hover:bg-white/5'}"
					onclick={() => { typeFilter = tab.id; isDropdownOpen = true; }}
				>
					{tab.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Selected Version Display & Selector Box -->
	<div class="rounded-2xl border border-white/10 bg-[#151619] p-2 space-y-2 shadow-inner">
		<div class="flex items-center justify-between px-2 py-1 text-[11px] text-white/50">
			<div class="flex items-center gap-1.5">
				<span>Versão selecionada:</span>
				<span class="text-brand-500 font-black text-xs bg-brand-500/10 px-2 py-0.5 rounded border border-brand-500/20">
					{value || (loading ? "Carregando..." : "Nenhuma selecionada")}
				</span>
			</div>
			<span class="text-[10px] text-white/40">
				{filtered.length} versões encontradas
			</span>
		</div>

		<!-- Versions Grid / List -->
		<div class="max-h-44 overflow-y-auto custom-scrollbar pr-1 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-1.5">
			{#if loading}
				<div class="col-span-full py-8 text-center text-xs text-white/40 flex items-center justify-center gap-2">
					<div class="w-4 h-4 rounded-full border-2 border-brand-500 border-t-transparent animate-spin"></div>
					Carregando versões do Mojang...
				</div>
			{:else if filtered.length === 0}
				<div class="col-span-full py-8 text-center text-xs text-white/40">
					Nenhuma versão corresponde à pesquisa "{search}".
				</div>
			{:else}
				{#each filtered as v (v.id)}
					{@const isSelected = value === v.id}
					<button
						type="button"
						class="flex items-center justify-between px-3 py-2 rounded-xl text-left transition-all cursor-pointer border {isSelected ? 'bg-brand-500 text-black border-brand-500 font-black shadow-md scale-[1.02]' : 'bg-[#1e1f24] hover:bg-[#25262c] text-white/80 hover:text-white border-white/5 hover:border-white/15'}"
						onclick={() => pick(v.id)}
					>
						<div class="min-w-0">
							<span class="text-xs truncate block {isSelected ? 'font-black text-black' : 'font-bold'}">{v.id}</span>
							<span class="text-[8px] uppercase tracking-wider block opacity-70">
								{v.versionType === 'release' ? 'Release' : v.versionType === 'snapshot' ? 'Snapshot' : 'Beta'}
							</span>
						</div>
						{#if isSelected}
							<Check class="w-3.5 h-3.5 stroke-[3] shrink-0 text-black ml-1" />
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	</div>
</div>

