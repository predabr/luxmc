<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { Search, Check, Sparkles, X } from "lucide-svelte";

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
	}

	const filterTabs = [
		{ id: "release", label: uiText("ui.1a6f7cae06cc1fc9") },
		{ id: "all", label: uiText("instances.groupAll") },
		{ id: "snapshot", label: uiText("ui.f187f78e07efb26e") },
		{ id: "old_beta", label: uiText("ui.4d1cbbcdc56bb30d") }
	] as const;

	const latestRelease = $derived(versions.filter(version => version.versionType === "release").toSorted((a,b) => b.releaseTime.localeCompare(a.releaseTime))[0]?.id);
    const popularVersions = $derived([
		{ id: latestRelease || "", label: latestRelease || uiText("common.loading"), tag: "Mais recente" },
		{ id: "1.20.1", label: uiText("ui.774e064308df6c12"), tag: "Modpacks" },
		{ id: "1.16.5", label: uiText("ui.7d92573f62b0facb"), tag: "Forge" },
		{ id: "1.8.9", label: uiText("ui.397b98314838406b"), tag: "PvP" }
	] as const);
</script>

<div class="version-selector min-w-0 space-y-3 select-none">
	<!-- Top: Search Bar & Type Filter Tabs -->
	<div class="flex min-w-0 flex-col gap-2">
		<div class="relative min-w-0">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-fg/40" />
			<input
				type="text"
				placeholder={uiText("ui.904237f8de88ff52")}
				bind:value={search}
				aria-label={uiText("ui.904237f8de88ff52")}
				class="w-full h-11 pl-9 pr-9 rounded-xl bg-bg-elevated border border-fg/10 focus:border-brand-500 text-xs font-bold text-fg placeholder:text-fg/30 outline-none transition-[color,background-color,border-color,box-shadow,transform,opacity]"
			/>
			{#if search}
				<button
					type="button"
					class={launcherButton({ variant: "ghost", size: "icon", class: "absolute right-3 top-1/2 -translate-y-1/2" })}
					onclick={() => (search = "")}
					aria-label={uiText("statusBanner.dismiss")}
				>
					<X class="w-3.5 h-3.5" />
				</button>
			{/if}
		</div>

		<!-- Filter Pills -->
		<div class="version-filters grid grid-cols-4 gap-1 bg-bg-elevated p-1 rounded-xl border border-fg/10">
			{#each filterTabs as tab}
				<button
					type="button"
					aria-pressed={typeFilter === tab.id}
					class:selection-control={typeFilter === tab.id}
					class="version-filter min-w-0 px-2 py-2.5 rounded-lg text-[11px] font-semibold transition-[color,background-color,box-shadow] cursor-pointer {typeFilter === tab.id ? 'bg-brand-500 shadow' : 'text-fg-muted hover:text-fg hover:bg-fg/5'}"
					onclick={() => { typeFilter = tab.id; }}
				>
					<span>{tab.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Quick Popular Chips -->
	<div class="flex items-center gap-1.5 flex-wrap">
		<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 flex items-center gap-1 pl-1">
			<Sparkles class="w-3 h-3 text-brand-500" /> {uiText("ui.59a8433c136ff3f5")}
		</span>
		{#each popularVersions as pop}
			<button
				type="button"
				class="px-2.5 py-1 rounded-lg text-xs font-bold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer flex items-center gap-1.5 {value === pop.id ? 'bg-brand-500/20 text-brand-400 border border-brand-500/40 shadow-sm' : 'bg-bg-elevated border border-fg/10 text-fg/70 hover:text-fg hover:border-fg/30'}"
				onclick={() => pick(pop.id)}
			>
				<span>{pop.label}</span>
				<span class="text-[9px] font-normal text-fg/40">({pop.tag})</span>
			</button>
		{/each}
	</div>

	<!-- Selected Version Display & Selector Box -->
	<div class="rounded-2xl border border-fg/10 bg-bg-elevated p-2 space-y-2 shadow-inner">
		<div class="flex flex-wrap items-center justify-between gap-2 px-2 py-1 text-[11px] text-fg-muted">
			<div class="flex min-w-0 flex-wrap items-center gap-1.5">
				<span>{uiText("ui.825e12c5880a1577")}</span>
				<span class="text-brand-500 font-black text-xs bg-brand-500/10 px-2 py-0.5 rounded border border-brand-500/20">
					{value || (loading ? uiText("app.loading") : uiText("ui.a7114af0c26462f5"))}
				</span>
			</div>
			<span class="text-[10px] text-fg/40">
				{filtered.length} {uiText("ui.f8e2c7ea1fb757ee")}
			</span>
		</div>

		<!-- Versions Grid / List -->
		<div class="version-grid overflow-y-auto custom-scrollbar grid gap-2">
			{#if loading}
				<div class="col-span-full py-8 text-center text-xs text-fg/40 flex items-center justify-center gap-2">
					<div class="w-4 h-4 rounded-full border-2 border-brand-500 border-t-transparent animate-spin"></div>
					{uiText("ui.09d2e14e0489a66b")}
				</div>
			{:else if filtered.length === 0}
				<div class="col-span-full py-8 text-center text-xs text-fg/40">
					{uiText("ui.d3b77524e66dab9d")}{search}".
				</div>
			{:else}
				{#each filtered as v (v.id)}
					{@const isSelected = value === v.id}
					<button
						type="button"
						class="version-option min-w-0 flex items-center justify-between gap-2 px-3 py-3 rounded-xl text-left cursor-pointer border {isSelected ? 'bg-brand-500 border-brand-500 font-semibold shadow-md' : 'bg-bg-subtle text-fg/80 hover:text-fg border-fg/5 hover:border-fg/20'}"
						aria-pressed={isSelected}
						class:selection-control={isSelected}
						onclick={() => pick(v.id)}
					>
						<div class="min-w-0">
							<span class="version-label text-xs block font-semibold">{v.id}</span>
							<span class="mt-1 text-[10px] uppercase tracking-wider block opacity-80">
								{v.versionType === 'release' ? 'Release' : v.versionType === 'snapshot' ? 'Snapshot' : 'Beta'}
							</span>
						</div>
						{#if isSelected}
							<Check class="w-3.5 h-3.5 stroke-[3] shrink-0" />
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	</div>
</div>

<style>
	.version-grid { grid-template-columns: repeat(auto-fill, minmax(min(118px, 100%), 1fr)); max-height: 248px; padding: 3px 5px 5px 3px; scrollbar-gutter: stable; }
	.version-option { min-height: 64px; transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, translate 220ms cubic-bezier(.16,1,.3,1); }
	.version-option:hover { translate: 0 -2px; }
	.version-label { overflow-wrap: anywhere; line-height: 1.4; }
	.version-filter[aria-pressed="true"], .version-option[aria-pressed="true"] { background-color: rgb(var(--brand-action)) !important; color: rgb(var(--brand-foreground)) !important; }
	.version-filter[aria-pressed="true"] > span, .version-option[aria-pressed="true"] .version-label { color: rgb(var(--brand-foreground)) !important; }
	.version-filter:focus-visible, .version-option:focus-visible { outline: 2px solid rgb(var(--brand-400)); outline-offset: 2px; }
	:global(html.no-ui-motion) .version-option, :global(html.no-anim) .version-option { transition: none; translate: none; }
	@media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .version-option { transition: none; translate: none; } }
</style>

