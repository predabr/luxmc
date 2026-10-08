<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { modsVersions } from "$lib/api/mods";
    import { modpackUpdateAtomic, modpackVersionDiff } from "$lib/api/java";
    import type { ModpackVersionDiff } from '$lib/api/types';
    import GlassSelect from '$lib/components/ui/GlassSelect.svelte';
    let plan=$state<ModpackVersionDiff|null>(null);let plannedVersion=$state('');let plannedProfile=$state('');let planLimit=$state(80);
    const groupLabels:Record<string,string>={added:'studio.added',removed:'studio.removed',updated:'studio.updated',configuration:'studio.configuration'};
    async function review(){if(busy || !selected)return;busy=true;error='';confirm=false;plan=null;const version=selected;const id=profileId;const source=info.source;try{const value=await modpackVersionDiff(id,version,source);if(version===selected && id===profileId && source===info.source){plan=value;plannedVersion=version;plannedProfile=id;confirm=value.available;planLimit=80;}}catch(cause){error=String(cause);}finally{busy=false;}}
    import type { ModpackUpdateInfo, ModVersion } from "$lib/api/types";
    import { appState } from "$lib/stores/app.svelte";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { toast } from "$lib/stores/toasts.svelte";
    import { button } from "$lib/components/ui/button";
    let { profileId, info, onChanged }: { profileId: string; info: ModpackUpdateInfo; onChanged: () => Promise<void> } = $props();
    let versions = $state<ModVersion[]>([]);
    let selected = $state("");
    let busy = $state(false);
    let error = $state("");
    let confirm = $state(false);
    let loadedProject = "";
    $effect(()=>{if(plannedProfile && plannedProfile!==profileId){plan=null;confirm=false;}});
    async function load() {
        if (!info.projectId || busy || loadedProject === info.projectId) return;
        busy = true; error = "";
        try {
            versions = await modsVersions(info.projectId, "", info.source);
            selected = versions[0]?.id || "";
            loadedProject = info.projectId;
        } catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
    async function apply() {
        if (busy || !selected || plannedVersion!==selected || plannedProfile!==profileId || !plan?.available || !info.projectId || appState.isGameRunning || appState.isLaunching) return;
        busy = true; error = "";
        try {
            await modpackUpdateAtomic(profileId, selected, info.source, info.projectId);
            await profiles.refresh(); await onChanged();
            confirm = false; toast(uiText("ui.d438a4bceadd94fb"), "success");
        } catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
</script>

<details class="surface-glass p-5" ontoggle={event => { if (event.currentTarget.open) void load(); }}>
    <summary class="cursor-pointer text-sm font-semibold text-fg">{uiText("ui.002e2ec032e6680b")} {info.currentVersion || uiText("settings.installedVersionBadge")}</summary>
    <div class="mt-4 space-y-3">
        <p class="text-xs text-fg-muted">{uiText("ui.e1afd638f469c06c")}</p>
        {#if error}<p role="alert" class="text-sm text-danger">{error}</p><button class={button({ variant: "secondary" })} disabled={busy} onclick={load}>{uiText("ui.b9e10688be012d8b")}</button>{/if}
        <div class="flex flex-wrap gap-3">
            <GlassSelect bind:value={selected} options={versions.map(version=>({value:version.id,label:version.versionNumber+' · '+version.name}))} label={uiText("ui.021cac6a15e13564")} disabled={busy} onchange={()=>{confirm=false;plan=null;}} />
            <button class={button({ variant: "primary" })} disabled={busy || !selected || appState.isGameRunning || appState.isLaunching} onclick={()=>void review()}>{busy ? uiText("ui.aebd32f10b997477") : uiText("studio.planUpdate")}</button>
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
