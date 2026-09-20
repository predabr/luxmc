<script lang="ts">
	import {
		Search,
		ArrowUpDown,
		LayoutGrid,
		List,
		Group,
		ChevronDown
	} from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";

	const { t } = useTranslation();

	interface Props {
		searchQuery?: string;
		sortBy?: "name" | "version" | "date" | "lastPlayed";
		groupFilter?: string;
		viewMode?: "grid" | "list";
		groups?: string[];
		searchInput?: HTMLInputElement | null;
		onSearchChange?: (value: string) => void;
		onSortChange?: (value: string) => void;
		onGroupChange?: (value: string) => void;
	 onViewChange?: (mode: "grid" | "list") => void;
	}

	let {
		searchQuery = $bindable(""),
		sortBy = $bindable("name"),
		groupFilter = $bindable("all"),
		viewMode = $bindable("grid"),
		groups = ["all", "Modded", "Vanilla", "Servers", "Favorites"],
		searchInput = $bindable(null),
		onSearchChange,
		onSortChange,
		onGroupChange,
		onViewChange
	}: Props = $props();

	function getGroupLabel(g: string): string {
		if (g === "all") return t("instances.groupAll");
		if (g === "Modded") return t("instances.groupModded");
		if (g === "Vanilla") return t("instances.groupVanilla");
		if (g === "Servers") return t("instances.groupServers");
		if (g === "Favorites") return t("instances.groupFavorites");
		return g;
	}
</script>

<div class="surface-glass flex flex-wrap items-center gap-3 p-3">
	<div class="relative min-w-48 flex-1">
		<Search class="absolute left-3.5 top-1/2 h-4 w-4 -translate-y-1/2 text-fg/40" />
		<input
			bind:this={searchInput}
			class="h-10 w-full rounded-full pl-10 pr-4 text-xs font-bold outline-none placeholder:text-fg/30 bg-bg-elevated border border-fg/10 focus:border-brand-500 text-fg transition-all"
			placeholder={t("instances.searchPlaceholder")}
			bind:value={searchQuery}
			oninput={() => onSearchChange?.(searchQuery)}
		/>
	</div>

	<div class="relative flex items-center gap-1.5 bg-bg-elevated border border-fg/10 rounded-full pl-3.5 pr-8 h-10">
		<ArrowUpDown class="h-3.5 w-3.5 text-fg/40 shrink-0 pointer-events-none" />
		<select
			class="appearance-none bg-transparent border-0 text-xs font-bold text-fg outline-none cursor-pointer pr-1"
			bind:value={sortBy}
			onchange={() => onSortChange?.(sortBy)}
		>
			<option value="name" class="bg-bg-elevated">{t("instances.sortName")}</option>
			<option value="version" class="bg-bg-elevated">{t("instances.sortVersion")}</option>
			<option value="date" class="bg-bg-elevated">{t("instances.sortDate")}</option>
			<option value="lastPlayed" class="bg-bg-elevated">{t("instances.sortLastPlayed")}</option>
		</select>
		<ChevronDown class="w-3 h-3 text-fg/40 pointer-events-none absolute right-3 top-1/2 -translate-y-1/2" />
	</div>

	<div class="relative flex items-center gap-1.5 bg-bg-elevated border border-fg/10 rounded-full pl-3.5 pr-8 h-10">
		<Group class="h-3.5 w-3.5 text-fg/40 shrink-0 pointer-events-none" />
		<select
			class="appearance-none bg-transparent border-0 text-xs font-bold text-fg outline-none cursor-pointer pr-1"
			bind:value={groupFilter}
			onchange={() => onGroupChange?.(groupFilter)}
		>
			{#each groups as g}
				<option value={g} class="bg-bg-elevated">{getGroupLabel(g)}</option>
			{/each}
		</select>
		<ChevronDown class="w-3 h-3 text-fg/40 pointer-events-none absolute right-3 top-1/2 -translate-y-1/2" />
	</div>

	<div class="flex items-center gap-1 p-1 bg-bg-elevated border border-fg/10 rounded-full h-10">
		<button
			class="grid h-8 w-8 place-items-center rounded-full transition-all cursor-pointer {viewMode === 'grid' ? 'bg-fg/10 text-brand-500 shadow-sm' : 'text-fg/40 hover:text-fg'}"
			onclick={() => { viewMode = "grid"; onViewChange?.("grid"); }}
			aria-label={t("instances.gridView")}
		>
			<LayoutGrid class="h-3.5 w-3.5" />
		</button>
		<button
			class="grid h-8 w-8 place-items-center rounded-full transition-all cursor-pointer {viewMode === 'list' ? 'bg-fg/10 text-brand-500 shadow-sm' : 'text-fg/40 hover:text-fg'}"
			onclick={() => { viewMode = "list"; onViewChange?.("list"); }}
			aria-label={t("instances.listView")}
		>
			<List class="h-3.5 w-3.5" />
		</button>
	</div>
</div>
