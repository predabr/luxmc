<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { ChevronDown, SlidersHorizontal, Check, Globe } from "lucide-svelte";
	import LoaderLogo from "$lib/components/mods/LoaderLogo.svelte";
	import ProviderLogo from "./ProviderLogo.svelte";
	import { catalogTypes } from "$lib/utils/catalogTypes";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { onMount } from "svelte";
	import { versionsList } from "$lib/api/launch";

	let {
		selectedType = $bindable("Modpack"),
		selectedSource = $bindable("all"),
		selectedVersion = $bindable("Qualquer Versão"),
		selectedLoader = $bindable(null),
		selectedCategory = $bindable(null)
	}: {
		selectedType: string;
		selectedSource: "all" | "modrinth" | "curseforge" | "ftb";
		selectedVersion: string;
		selectedLoader: string | null;
		selectedCategory: string | null;
	} = $props();

    const sources = $derived<Array<{ id: "all" | "modrinth" | "curseforge" | "ftb"; label: string; icon?: typeof Globe }>>([
        { id: 'all', label: uiText('ui.8a0108a42be96e26'), icon: Globe },
        { id: 'modrinth', label: 'Modrinth' },
        { id: 'curseforge', label: 'CurseForge' },
        ...(selectedType === 'Modpack' ? [{ id: 'ftb' as const, label: 'Feed The Beast' }] : [])
    ]);
    const activeCount = $derived(Number(selectedSource !== 'all') + Number(selectedVersion !== 'Qualquer Versão') + Number(!!selectedLoader) + Number(!!selectedCategory));
	const showLoaders = $derived(selectedType === 'Mod' || selectedType === 'Modpack');
	const showCategories = $derived(selectedType === 'Modpack' || selectedType === 'Mod');

	const modLoaders = [
		{ name: "Fabric" },
		{ name: "Quilt" },
		{ name: "Forge" },
		{ name: "NeoForge" }
	];

	const modpackCategories = [
		{ id: "Adventure", label: "Aventura" }, { id: "Challenging", label: "Desafiante" },
		{ id: "Combat", label: "Combate" }, { id: "Kitchen-Sink", label: "Kitchen-Sink" },
		{ id: "Lightweight", label: "Leve" }, { id: "Magic", label: "Magia" },
		{ id: "Multiplayer", label: "Multijogador" }, { id: "Optimization", label: "Otimização" },
		{ id: "Quests", label: "Missões" }, { id: "Technology", label: "Tecnologia" }
	];

	const modCategories = [
		{ id: "adventure", label: "Aventura" }, { id: "technology", label: "Tecnologia" },
		{ id: "magic", label: "Magia" }, { id: "utility", label: "Utilidades" },
		{ id: "optimization", label: "Otimização" }, { id: "worldgen", label: "Geração de mundo" },
		{ id: "equipment", label: "Equipamentos" }, { id: "food", label: "Comida" },
		{ id: "storage", label: "Armazenamento" }, { id: "transportation", label: "Transporte" }
	];

	const categories = $derived(selectedType === 'Mod' ? modCategories : modpackCategories);

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

</script>

<aside id="catalog-filter-panel" class="catalog-filters" aria-label={uiText('ui.57ac13ce3d5bedff')}>
	<header class="drawer-head">
		<p class="drawer-title"><SlidersHorizontal class="h-4 w-4" /> {uiText('ui.57ac13ce3d5bedff')}{#if activeCount}<span class="filter-count">{activeCount}</span>{/if}</p>
	</header>
	<div class="drawer-body">
			<div class="filter-grid">
				<fieldset class="catalog-provider-row">
					<legend class="filter-label">{uiText('catalogFilters.source')}</legend>
					{#each sources as source (source.id)}
						<button type="button" aria-pressed={selectedSource === source.id} onclick={() => selectedSource = source.id}>
							<span class="provider-symbol" class:provider-modrinth={source.id === 'modrinth'} class:provider-curseforge={source.id === 'curseforge'} class:provider-ftb={source.id === 'ftb'}>
								{#if source.id === 'all'}<Globe class="h-4 w-4" />{:else}<ProviderLogo source={source.id} class="h-4 w-4" />{/if}
							</span><span>{source.label}</span>{#if selectedSource === source.id}<Check class="ml-auto h-3.5 w-3.5" />{/if}
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

				{#if showLoaders}
					<div>
						<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("catalogFilters.loader")}</div>
						<div class="grid grid-cols-2 gap-2">
							{#each modLoaders as loader (loader.name)}
								<button
									type="button"
									aria-pressed={selectedLoader?.toLowerCase() === loader.name.toLowerCase()}
									class="filter-choice loader-choice px-2.5 py-2 rounded-xl text-xs font-semibold transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer border flex items-center gap-1.5 {selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? 'bg-brand-500/15 text-brand-action border-brand-400/45 font-bold' : 'bg-bg-elevated text-fg/80 border-fg/15 hover:border-fg/30 hover:text-fg'}"
									onclick={() => selectedLoader = selectedLoader?.toLowerCase() === loader.name.toLowerCase() ? null : loader.name.toLowerCase()}
								>
									{#if selectedLoader?.toLowerCase() === loader.name.toLowerCase()}<Check class="h-3 w-3" />{:else}<LoaderLogo loader={loader.name} size={14} />{/if}
									<span class="truncate">{loader.name}</span>
								</button>
							{/each}
						</div>
					</div>
				{/if}

				{#if showCategories}
					<div class="categories">
						<div class="text-[11px] font-bold text-fg-muted uppercase tracking-wider mb-2">{uiText("ui.3adb5537c59ac03f")}</div>
						<div class="flex flex-wrap gap-1.5">
							{#each categories as cat (cat.id)}
								<button
									type="button"
									aria-pressed={selectedCategory === cat.id}
									class="filter-choice px-2.5 py-1 rounded-full text-[11px] font-medium transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedCategory === cat.id ? 'bg-brand-500/15 text-brand-action border border-brand-400/45 font-bold' : 'bg-bg-elevated text-fg/80 border border-fg/15 hover:border-fg/30 hover:text-fg'}"
									onclick={() => selectedCategory = selectedCategory === cat.id ? null : cat.id}
								>
									{uiText(`catalogFilters.categories.${cat.id}`) || cat.label}
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>
			<p class="filter-help">{uiText("ui.c7984fbe61202b7f")}</p>
		</div>
</aside>

<style>
	.catalog-filters { min-width: 0; width: 100%; border: 1px solid rgb(var(--border)); border-radius: 18px; background: rgb(var(--bg-elevated) / .96); overflow: hidden; }
	:global(html.has-custom-wallpaper) .catalog-filters { background: rgb(var(--bg-elevated) / max(.88, var(--interface-opacity, .88))) !important; }
	.drawer-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 9px 10px; border-bottom: 1px solid rgb(var(--border)); }
	.drawer-title { display: flex; align-items: center; gap: 7px; font-size: 11px; font-weight: 700; color: rgb(var(--fg)); }
	.drawer-body { padding: 10px; }
	.filter-grid { display: grid; grid-template-columns: minmax(0, 1fr); align-items: start; gap: 13px; }
	.filter-grid > div { min-width: 0; }
	.filter-count { padding: 1px 6px; border-radius: 6px; background: rgb(var(--brand-500) / .14); color: rgb(var(--brand-action)); font-size: 10px; font-weight: 700; }
	.filter-label { margin-bottom: 6px; color: rgb(var(--fg-muted)); font-size: 10px; font-weight: 700; letter-spacing: .06em; }
	.catalog-provider-row { display: grid; grid-template-columns: minmax(0, 1fr); gap: 8px; min-width: 0; border: 0; padding: 0; margin: 0; }
	.catalog-provider-row button { display: flex; align-items: center; justify-content: flex-start; gap: 9px; min-width: 0; width: 100%; min-height: 40px; padding: 7px 9px; border: 1px solid rgb(var(--border)); border-radius: 12px; color: rgb(var(--fg-muted)); background: rgb(var(--bg-elevated) / .92); font-size: 11px; font-weight: 650; text-align: left; transition: background-color 180ms ease, border-color 180ms ease, color 180ms ease; }
	.catalog-provider-row button:hover { color: rgb(var(--fg)); background: rgb(var(--bg-elevated) / .98); }
	.catalog-provider-row button[aria-pressed='true'] { color: rgb(var(--brand-action)); border-color: rgb(var(--brand-400) / .45); background: rgb(var(--brand-500) / .15); }
	.provider-symbol { display: grid; place-items: center; width: 24px; height: 24px; flex-shrink: 0; border-radius: 8px; background: rgb(var(--fg) / .07); }
	.provider-modrinth { color: rgb(var(--success)); background: rgb(var(--success) / .12); }
	.provider-curseforge { color: rgb(var(--warning)); background: rgb(var(--warning) / .14); }
	.filter-choice { min-height: 30px; }
	.categories > div:last-child { max-height: 150px; overflow-y: auto; overscroll-behavior: contain; scrollbar-width: thin; scrollbar-color: rgb(var(--border)) transparent; padding: 1px 2px 4px 0; gap: 8px; }
	.filter-choice:focus-visible, .catalog-provider-row button:focus-visible { outline: 2px solid rgb(var(--brand-action)); outline-offset: 2px; }
	:global(html.has-custom-wallpaper) .catalog-provider-row button,
	:global(html.has-custom-wallpaper) .filter-choice,
	:global(html.has-custom-wallpaper) .catalog-filters select { background-color: rgb(var(--bg-elevated) / max(.84, var(--interface-opacity, .84))) !important; }
	:global(html.has-custom-wallpaper) .catalog-provider-row button[aria-pressed='true'],
	:global(html.has-custom-wallpaper) .filter-choice[aria-pressed='true'] { background-color: rgb(var(--brand-500) / .22) !important; }
	.filter-help { margin-top: 10px; padding-top: 9px; border-top: 1px solid rgb(var(--border)); color: rgb(var(--fg-muted)); font-size: 10px; line-height: 1.5; }
</style>
