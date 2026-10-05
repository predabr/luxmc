<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
    import { focusTrap } from "$lib/utils/focusTrap";
	import { Box, Check, Download, X, Cpu } from "lucide-svelte";
	import { fade, scale } from "svelte/transition";
	import type { ModSearchResultItem } from "$lib/api";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { getIconSrc } from "$lib/utils/icons";
	import { instanceWorldsList } from "$lib/api/instances";
    import type { WorldDetail } from "$lib/api/types";
	import { catalogCompatibility } from "$lib/utils/catalogCompatibility";

	let {
		item,
		selectedType = "Mod",
		chosenInstanceId = $bindable(""),
        chosenWorldName = $bindable(""),
		onConfirm,
		onClose
	}: {
		item: ModSearchResultItem;
		selectedType?: string;
		chosenInstanceId?: string;
        chosenWorldName?: string;
		onConfirm: () => void;
		onClose: () => void;
	} = $props();
	const chosenProfile = $derived(profiles.list.find(profile => profile.id === chosenInstanceId));
	let worlds = $state<WorldDetail[]>([]);
    let loadingWorlds = $state(false);
    let worldError = $state("");
    $effect(() => {
        const id = chosenInstanceId;
        chosenWorldName = ""; worlds = []; worldError = "";
        if (selectedType !== "Data Pack" || !id) { loadingWorlds = false; return; }
        loadingWorlds = true;
        let active = true;
        void instanceWorldsList(id).then(value => { if (active) { worlds = value; chosenWorldName = value[0]?.folderName ?? ""; } }).catch(error => { if (active) worldError = String(error); }).finally(() => { if (active) loadingWorlds = false; });
        return () => { active = false; };
    });
    const canInstall = $derived(!!chosenProfile && !catalogCompatibility(item, chosenProfile, selectedType, false) && (selectedType !== "Data Pack" || (!loadingWorlds && worlds.some(world => world.folderName === chosenWorldName))));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-[999999] bg-bg-overlay/80 backdrop-blur-sm flex items-center justify-center p-6"
	transition:fade={{ easing: quintOut, duration: 220 }}
	onclick={onClose}
>
	<div
		role="dialog" aria-modal="true" aria-label={uiText("ui.328774e0b805dc97")} tabindex="-1" use:focusTrap
        class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 max-w-md w-full shadow-2xl space-y-5"
		transition:scale={{ easing: backOut, start: 0.95, duration: 220 }}
		onclick={(e) => e.stopPropagation()}
        onkeydown={(event) => { if (event.key === "Escape") { event.stopPropagation(); onClose(); } }}
	>
		<div class="flex items-center justify-between border-b border-fg/5 pb-3">
			<div class="flex items-center gap-2.5">
				<div class="w-8 h-8 rounded-xl bg-brand-500/10 text-brand-400 flex items-center justify-center">
					<Box class="w-4 h-4" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-fg">{uiText("ui.6f8f37a5a2949fc1")}</h3>
					<p class="text-[11px] text-fg/40 font-medium">{uiText("ui.0e50c0ecaa67b780")} {selectedType}?</p>
				</div>
			</div>
			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
				aria-label={uiText("ui.4c8167f0e34958d5")} onclick={onClose}
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		<div class="flex items-center gap-3 bg-bg-elevated p-3 rounded-2xl border border-fg/5">
			<div class="w-10 h-10 rounded-xl bg-bg-subtle border border-fg/10 overflow-hidden shrink-0 flex items-center justify-center">
				{#if item.iconUrl}
					<img loading="lazy" decoding="async"
						src={item.iconUrl}
						alt={item.title}
						class="w-full h-full object-contain p-0.5"
						onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
					/>
				{:else}
					<Cpu class="w-5 h-5 text-brand-400" />
				{/if}
			</div>
			<div class="min-w-0 flex-1">
				<h4 class="text-xs font-extrabold text-fg truncate">{item.title}</h4>
				<span class="text-[10px] text-fg/40">{item.author || "Autor"} • {selectedType}</span>
			</div>
		</div>

		<div class="space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-1">
			{#each profiles.list as p (p.id)}
				{@const incompatibility = catalogCompatibility(item, p, selectedType, false)}
				<button
					type="button"
					disabled={!!incompatibility}
					class="w-full text-left p-3 rounded-2xl border transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer disabled:cursor-default disabled:opacity-50 flex items-center justify-between gap-3 {chosenInstanceId === p.id ? 'bg-brand-500/15 border-brand-500 shadow-sm' : 'bg-bg-elevated border-fg/5 hover:border-fg/20'}"
					onclick={() => chosenInstanceId = p.id}
				>
					<div class="flex items-center gap-3 min-w-0">
						<div class="w-9 h-9 rounded-xl bg-fg/5 border border-fg/10 flex items-center justify-center overflow-hidden shrink-0">
							<img loading="lazy" decoding="async"
								src={getIconSrc(p.icon)}
								alt={p.name}
								class="w-full h-full object-contain p-0.5"
								onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/grass_block.png'; }}
							/>
						</div>
						<div class="min-w-0">
							<h5 class="text-xs font-bold text-fg truncate">{p.name}</h5>
							<div class="flex items-center gap-2 text-[10px] text-fg/40 mt-0.5">
								<span class="font-mono">{p.mcVersion}</span>
								<span>•</span>
								<span class="uppercase font-semibold text-brand-400">{p.loader}</span>
								{#if p.modCount}
									<span>•</span>
									<span>{p.modCount} {uiText("ui.695073cb6649c0a4")}</span>
								{/if}
							</div>
							{#if incompatibility}<p class="mt-1 text-[10px] text-warning">{incompatibility}</p>{/if}
						</div>
					</div>

					<div class="w-5 h-5 rounded-full border flex items-center justify-center shrink-0 {chosenInstanceId === p.id ? 'border-brand-500 bg-brand-500 text-fg' : 'border-fg/20 bg-transparent'}">
						{#if chosenInstanceId === p.id}
							<Check class="w-3 h-3 stroke-[3]" />
						{/if}
					</div>
				</button>
			{/each}
		</div>

		{#if selectedType === "Data Pack"}
            <label class="block text-sm text-fg">{uiText("ui.223c439bca70a997")}<select class="mt-2 w-full rounded-xl border border-border bg-bg-elevated p-3" bind:value={chosenWorldName} disabled={loadingWorlds || !worlds.length}><option value="">{loadingWorlds ? uiText("ui.d226527f72cf37cc") : uiText("ui.4d4ad367ef234911")}</option>{#each worlds as world}<option value={world.folderName}>{world.name}</option>{/each}</select></label>
            {#if worldError}<p role="alert" class="text-xs text-danger">{worldError}</p>{:else if !loadingWorlds && chosenInstanceId && !worlds.length}<p class="text-xs text-fg-muted">{uiText("ui.b76f47993283f0b9")}</p>{/if}
        {/if}
        <div class="flex items-center justify-end gap-3 pt-2">
			<button
				type="button"
				class={launcherButton({ variant: "secondary", size: "sm", class: "" })}
				onclick={onClose}
			>
				{uiText("common.cancel")}
			</button>
			<button
				type="button"
				class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50" })}
				disabled={!canInstall}
				onclick={onConfirm}
			>
				<Download class="w-4 h-4" />
				<span>{uiText("ui.b883a5e2c7c7e016")}</span>
			</button>
		</div>
	</div>
</div>
