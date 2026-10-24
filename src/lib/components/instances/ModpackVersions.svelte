<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { modsVersions } from "$lib/api/mods";
    import { untrack } from 'svelte';
    import { modpackVersionDiff } from "$lib/api/java";
    import { modpackUpdates } from '$lib/stores/modpackUpdates.svelte';
    import type { ModpackVersionDiff } from '$lib/api/types';
    import GlassSelect from '$lib/components/ui/GlassSelect.svelte';
    let plan=$state<ModpackVersionDiff|null>(null);let plannedVersion=$state('');let plannedProfile=$state('');let planLimit=$state(80);
    const groupLabels:Record<string,string>={added:'studio.added',removed:'studio.removed',updated:'studio.updated',configuration:'studio.configuration'};
    async function review() {
        if (busy || updating || !selected || sameVersion || appState.isGameRunning || appState.isLaunching) return;
        busy = true; error = ''; confirm = false; plan = null;
        const version = selected, id = profileId, source = info.source;
        try {
            const value = await modpackVersionDiff(id, version, source);
            if (version === selected && id === profileId && source === info.source) {
                plan = value; plannedVersion = version; plannedProfile = id; confirm = value.available; planLimit = 80;
            }
        } catch (cause) { if (id === profileId && source === info.source) error = String(cause); }
        finally { busy = false; }
    }

    import type { ModpackUpdateInfo, ModVersion } from "$lib/api/types";
    import { appState } from "$lib/stores/app.svelte";
    import { button } from "$lib/components/ui/button";
    import { ArrowUpRight, CheckCircle2, RefreshCw, ShieldCheck } from 'lucide-svelte';
    let { profileId, info, onChanged }: { profileId: string; info: ModpackUpdateInfo; onChanged: () => Promise<void> } = $props();
    let versions = $state<ModVersion[]>([]);
    let selected = $state("");
    let busy = $state(false);
    let error = $state("");
    let confirm = $state(false);
    let loadedProject = "";
    let selector: HTMLDetailsElement;
    let loadingRequest: Promise<void> | null = null;
    let loadingKey = "";
    let displayedProject = '';
    const sameVersion = $derived(Boolean(selected && (selected === info.currentVersionId || versions.find(version => version.id === selected)?.versionNumber === info.currentVersion)));
    const updating = $derived(modpackUpdates.task?.busy ?? false);
    $effect(()=>{if(plannedProfile && plannedProfile!==profileId){plan=null;confirm=false;}});
    $effect(() => {
        const key = `${profileId}:${info.source}:${info.projectId}`;
        untrack(() => {
            if (displayedProject === key) return;
            displayedProject = key; loadedProject = ''; versions = []; selected = ''; plan = null; confirm = false; error = '';
            if (selector?.open) void load();
        });
    });
    async function load() {
        const key = `${profileId}:${info.source}:${info.projectId}`;
        if (!info.projectId || loadedProject === key) return;
        if (loadingRequest) { await loadingRequest; if (loadedProject !== key) return load(); return; }
        loadingKey = key;
        busy = true; error = "";
        const project = info.projectId, source = info.source;
        const request = (async () => {
            try {
                const result = await modsVersions(project, "", source);
                if (key !== `${profileId}:${info.source}:${info.projectId}`) return;
                versions = result;
                selected = versions.find(version => version.id === info.currentVersionId || version.id === info.currentVersion || version.versionNumber === info.currentVersion)?.id || "";
                loadedProject = key;
            } catch (cause) { if (loadingKey === key && key === `${profileId}:${info.source}:${info.projectId}`) error = String(cause); }
            finally { busy = false; loadingRequest = null; }
        })();
        loadingRequest = request;
        return request;
    }
    async function reviewLatest() {
        if (busy || updating || !info.hasUpdate || !info.versionId) return;
        const id = profileId, target = info.versionId;
        await load();
        if (profileId !== id || info.versionId !== target || error) return;
        selected = target;
        if (selector) selector.open = true;
        await review();
    }
    async function apply() {
        if (busy || updating || !selected || plannedVersion!==selected || plannedProfile!==profileId || !plan?.available || !info.projectId || appState.isGameRunning || appState.isLaunching) return;
        busy = true; error = "";
        try {
            const id = profileId;
            await modpackUpdates.apply(id, selected, info.source, info.projectId, versions.find(version => version.id === selected)?.versionNumber || versions.find(version => version.id === selected)?.name);
            if (profileId === id) { await onChanged(); confirm = false; }
        } catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
</script>

<section data-modpack-update class="surface-glass space-y-4 p-5">
    <header class="flex flex-wrap items-center justify-between gap-4">
        <div class="flex min-w-0 items-start gap-3"><span class="rounded-xl bg-brand-500/10 p-2.5 text-brand-400">{#if info.hasUpdate}<ArrowUpRight class="h-5 w-5" />{:else}<CheckCircle2 class="h-5 w-5" />{/if}</span><div><h3 class="text-sm font-semibold">{uiText(info.hasUpdate ? 'updateCenter.available' : 'updateCenter.current')}</h3><p class="mt-1 text-xs text-fg-muted">{uiText('updateCenter.installed')}: <strong class="font-medium text-fg">{info.currentVersion || '—'}</strong>{#if info.hasUpdate}<span class="mx-2" aria-hidden="true">→</span>{uiText('updateCenter.availableVersion')}: <strong class="font-medium text-brand-400">{info.latestVersion || '—'}</strong>{/if}</p></div></div>
        <div class="flex gap-2"><button type="button" class={button({variant:'ghost',size:'icon'})} aria-label={uiText('updateCenter.check')} disabled={busy || updating || modpackUpdates.forProfile(profileId)?.checking} onclick={() => { error=''; void modpackUpdates.check(profileId,true).catch(cause => error=String(cause)); }}><RefreshCw class="h-4 w-4 {modpackUpdates.forProfile(profileId)?.checking ? 'animate-spin' : ''}" /></button>{#if info.hasUpdate}<button type="button" class={button({variant:'primary'})} disabled={busy || updating || appState.isLaunching || appState.isGameRunning} onclick={() => void reviewLatest()}>{busy ? uiText('ui.aebd32f10b997477') : uiText('updateCenter.review')}</button>{/if}</div>
    </header>
    {#if info.hasUpdate}<p class="flex items-center gap-2 text-xs leading-5 text-fg-muted"><ShieldCheck class="h-4 w-4 shrink-0 text-success" />{uiText('updateCenter.backup')}</p>{/if}
    {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
    <details bind:this={selector} class="border-t border-border pt-3" ontoggle={event => { if (event.currentTarget.open) void load(); }}>
    <summary class="cursor-pointer text-xs font-semibold text-fg-muted">{uiText('updateCenter.otherVersions')}</summary>
    <div class="mt-4 space-y-3">
        <p class="text-xs text-fg-muted">{uiText("ui.e1afd638f469c06c")}</p>
        {#if error}<button class={button({ variant: "secondary" })} disabled={busy} onclick={load}>{uiText("ui.b9e10688be012d8b")}</button>{/if}
        <div class="flex flex-wrap gap-3">
            <GlassSelect bind:value={selected} options={versions.map(version=>({value:version.id,label:version.name.includes(version.versionNumber) ? version.name : version.versionNumber+' · '+version.name}))} label={uiText("ui.021cac6a15e13564")} disabled={busy || updating} onchange={()=>{confirm=false;plan=null;}} />
            <button class={button({ variant: "secondary" })} disabled={busy || updating || !selected || sameVersion || appState.isGameRunning || appState.isLaunching} onclick={()=>void review()}>{busy || updating ? uiText("ui.aebd32f10b997477") : sameVersion ? uiText('updateCenter.alreadyInstalled') : uiText("studio.planUpdate")}</button>
        </div>
        {#if plan}
            <section class="space-y-4 rounded-2xl border border-brand-400/25 bg-brand-400/5 p-4" aria-label={uiText('studio.planUpdate')}>
                <p class="text-sm font-semibold">{plan.targetMinecraft || '—'} · {plan.targetLoader || '—'} · {plan.worlds || 0} {uiText('studio.worldsProtected')}</p>
                {#each [...(plan.risks || []),...(plan.dependencies || [])] as risk}<p class="rounded-xl border border-warning/20 bg-warning/5 p-3 text-xs leading-relaxed">{risk}</p>{/each}
                <div class="grid gap-3 sm:grid-cols-2">{#each [{title:'added',items:plan.added},{title:'removed',items:plan.removed},{title:'updated',items:plan.updated},{title:'configuration',items:plan.configuration || []}] as group}<details class="rounded-xl border border-border bg-bg-elevated p-3"><summary class="cursor-pointer text-xs font-semibold">{uiText(groupLabels[group.title])} · {group.items.length}</summary><ul class="mt-3 max-h-52 space-y-1 overflow-auto text-[11px] text-fg-muted">{#each group.items.slice(0,planLimit) as name}<li class="break-all">{name}</li>{/each}</ul>{#if group.items.length>planLimit}<button type="button" class={button({variant:'ghost',size:'sm'})} onclick={()=>planLimit+=80}>{uiText('studio.more')}</button>{/if}</details>{/each}</div>
                <p class="text-[11px] leading-relaxed text-fg-muted">{uiText('studio.planHelp')}</p>
            </section>
        {/if}
        {#if confirm}
            <div role="group" aria-label={uiText("ui.9512e28d505b97ce")} class="rounded-xl border border-warning/30 bg-warning/10 p-4">
                <p class="mb-3 text-sm text-fg">{uiText("ui.68470d5eae6bc777")} {versions.find(version => version.id === selected)?.versionNumber}?</p>
                <div class="flex gap-2"><button class={button({ variant: "primary" })} disabled={busy} onclick={apply}>{uiText("ui.092559626a31ce6b")}</button><button class={button({ variant: "secondary" })} disabled={busy} onclick={() => confirm = false}>{uiText("common.cancel")}</button></div>
            </div>
        {/if}
    </div>
</details>
</section>
