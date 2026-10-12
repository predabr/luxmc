<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { ChevronDown, SlidersHorizontal, Check, Globe, Package, Flame } from "lucide-svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { onMount } from "svelte";
	import { versionsList } from "$lib/api/launch";

	let {
		selectedSource = $bindable("all"),
		selectedVersion = $bindable("Qualquer Versão"),
		selectedLoader = $bindable(null),
		selectedCategory = $bindable(null),
        expanded = $bindable(false)
	}: {
		selectedSource: "all" | "modrinth" | "curseforge";
		selectedVersion: string;
		selectedLoader: string | null;
		selectedCategory: string | null;
        expanded?: boolean;
	} = $props();

    const sources = $derived([
        { id: 'all', label: uiText('ui.8a0108a42be96e26'), icon: Globe },
        { id: 'modrinth', label: 'Modrinth', icon: Package },
        { id: 'curseforge', label: 'CurseForge', icon: Flame }
    ] as const);
    const activeCount = $derived(Number(selectedSource !== 'all') + Number(selectedVersion !== 'Qualquer Versão') + Number(!!selectedLoader) + Number(!!selectedCategory));

	const modLoaders = [
		{ name: "Fabric", color: "text-cyan-400 border-cyan-500/30 bg-cyan-500/10" },
		{ name: "Quilt", color: "text-purple-400 border-purple-500/30 bg-purple-500/10" },
		{ name: "Forge", color: "text-amber-300 border-amber-500/30 bg-amber-500/15" },
		{ name: "NeoForge", color: "text-orange-400 border-orange-500/30 bg-orange-500/10" },
	];

	const categories = [
		"Adventure", "Challenging", "Combat", "Kitchen-Sink",
		"Lightweight", "Magic", "Multiplayer", "Optimization",
		"Quests", "Technology"
	];

	let releaseVersions = $state<string[]>([]);
	const mcVersions = $derived(["Qualquer Versão", ...new Set([
		...(selectedVersion !== "Qualquer Versão" ? [selectedVersion] : []),
		...profiles.list.map(profile => profile.mcVersion),
		...releaseVersions
	])]);
	let versionsUnavailable = $state(false);
	onMount(() => {
		let disposed = false;
		void versionsList().then(manifest => {
			if (!disposed) releaseVersions = manifest.versions.filter(version => version.versionType === "release").map(version => version.id);
		}).catch(() => { if (!disposed) versionsUnavailable = true; });
		return () => { disposed = true; };
	});

    onMount(() => {
        const desktop = window.matchMedia('(min-width: 1200px)');
        const update = () => { expanded = desktop.matches; };
        update();
        desktop.addEventListener('change', update);
        return () => desktop.removeEventListener('change', update);
    });
</script>

<aside id="catalog-filter-panel" class="catalog-filters">
    <details bind:open={expanded}>
        <summary class="filter-heading"><span class="filter-symbol"><SlidersHorizontal class="h-4 w-4" /></span>{uiText('ui.57ac13ce3d5bedff')}{#if activeCount}<span class="filter-count">{activeCount}</span>{/if}<ChevronDown class="ml-auto h-4 w-4 text-fg-muted transition-transform {expanded ? 'rotate-180' : ''}" /></summary>
        <div class="space-y-6 border-t border-border p-5">
        <fieldset class="catalog-provider-row">
            <legend class="filter-label">{uiText('catalogFilters.source')}</legend>
            {#each sources as source}
                <button type="button" aria-pressed={selectedSource === source.id} onclick={() => selectedSource = source.id}>
                    <span class="provider-symbol" class:provider-modrinth={source.id === 'modrinth'} class:provider-curseforge={source.id === 'curseforge'}><source.icon class="h-4 w-4" /></span><span>{source.label}</span>{#if selectedSource === source.id}<Check class="ml-auto h-3.5 w-3.5" />{/if}
                </button>
            {/each}
        </fieldset>

		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.5df9df4db410ba86")}</div>
			<div class="relative">
				<select
					bind:value={selectedVersion}
					aria-label={uiText("instances.version")}
					class="w-full bg-bg-elevated border border-fg/10 rounded-xl px-3 py-2 text-xs font-semibold text-fg focus:outline-none focus:border-brand-500 appearance-none cursor-pointer pr-8"
				>
					{#each mcVersions as ver}
						<option value={ver}>{ver === "Qualquer Versão" ? uiText("instances.allVersions") : ver}</option>
					{/each}
				</select>
				<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-fg/40 pointer-events-none" />
			</div>
			{#if versionsUnavailable}<p class="mt-2 text-xs text-fg-muted">{uiText("ui.939284277cae2162")}</p>{/if}
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("catalogFilters.loader")}</div>
			<div class="grid grid-cols-2 gap-2">
				{#each modLoaders as loader}
					<button
						type="button"
						aria-pressed={selectedLoader?.toLowerCase() === loader.name.toLowerCase()}
						class="filter-choice px-2.5 py-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer border flex items-center gap-1.5 {selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? `${loader.color} font-bold shadow-sm` : 'bg-bg-elevated text-fg/60 border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedLoader = selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? null : loader.name.toLowerCase()}
					>
						{#if selectedLoader?.toLowerCase() === loader.name.toLowerCase()}<Check class="h-3 w-3" />{:else}<span class="w-1.5 h-1.5 rounded-full bg-current"></span>{/if}
						<span class="truncate">{loader.name}</span>
					</button>
				{/each}
			</div>
		</div>
		<div>
			<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.3adb5537c59ac03f")}</div>
			<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto custom-scrollbar pr-1">
				{#each categories as cat}
					<button
						type="button"
                        aria-pressed={selectedCategory === cat}
						class="filter-choice px-2.5 py-1 rounded-full text-[11px] font-medium transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedCategory === cat ? 'bg-success/20 text-success border border-success/40 font-bold' : 'bg-bg-elevated text-fg/50 border border-fg/5 hover:border-fg/20 hover:text-fg'}"
						onclick={() => selectedCategory = selectedCategory === cat ? null : cat}
					>
						{uiText(`catalogFilters.categories.${cat}`)}
					</button>
				{/each}
			</div>
		</div>
            <p class="border-t border-border pt-4 text-xs leading-relaxed text-fg-muted">{uiText("ui.c7984fbe61202b7f")}</p>
        </div>
    </details>
</aside>

<style>
    .catalog-filters { min-width:0; width:100%; align-self: flex-start; position: sticky; top: 0; max-height:calc(100dvh - 120px); overflow-y:auto; scrollbar-width:thin; border: 1px solid rgb(var(--border)); border-radius: 22px; background: rgb(var(--bg-elevated) / .97); box-shadow: inset 0 1px 0 rgb(var(--fg) / .04); }
    .filter-heading { display:flex; align-items:center; gap:10px; padding:16px; list-style:none; cursor:pointer; color:rgb(var(--fg)); font-size:13px; font-weight:650; }
    .filter-symbol { display:grid; place-items:center; width:32px; height:32px; border:1px solid rgb(var(--brand-400) / .2); border-radius:10px; color:rgb(var(--brand-action)); background:rgb(var(--brand-500) / .08); }
    .filter-count { padding:2px 6px; border-radius:6px; background:rgb(var(--brand-500) / .12); color:rgb(var(--brand-action)); font-size:10px; }
    .filter-label { margin-bottom:10px; color:rgb(var(--fg-muted)); font-size:11px; font-weight:700; letter-spacing:.06em; }
    .catalog-provider-row { display:flex; flex-direction:column; gap:6px; min-width:0; }
    .catalog-provider-row button { display:flex; align-items:center; gap:10px; width:100%; min-height:42px; padding:7px 10px; border:1px solid transparent; border-radius:12px; color:rgb(var(--fg-muted)); font-size:12px; font-weight:550; text-align:left; transition:background-color 180ms ease,border-color 180ms ease,color 180ms ease; }
    .catalog-provider-row button:hover { color:rgb(var(--fg)); background:rgb(var(--fg) / .04); }
    .catalog-provider-row button[aria-pressed='true'] { color:rgb(var(--brand-action)); border-color:rgb(var(--brand-400) / .3); background:rgb(var(--brand-500) / .08); }
    .provider-symbol { display:grid; place-items:center; width:28px; height:28px; flex-shrink:0; border-radius:8px; background:rgb(var(--fg) / .04); }
    .provider-modrinth { color:rgb(var(--success)); background:rgb(var(--success) / .08); }
    .provider-curseforge { color:rgb(var(--warning)); background:rgb(var(--warning) / .08); }
    summary::-webkit-details-marker { display: none; }
    .filter-choice { min-height: 34px; }
    .filter-choice:focus-visible, summary:focus-visible, .catalog-provider-row button:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 3px; }
    @media (max-width: 1199px) { .catalog-filters { position: static; max-height:none; order: -1; } }
</style>
