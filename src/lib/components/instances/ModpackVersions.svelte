<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { modsVersions } from "$lib/api/mods";
    import { modpackUpdateAtomic } from "$lib/api/java";
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
        if (busy || !selected || !info.projectId || appState.isGameRunning || appState.isLaunching) return;
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
            <select aria-label={uiText("ui.021cac6a15e13564")} class="min-w-0 flex-1 rounded-xl border border-border bg-bg-elevated p-3 text-sm text-fg" bind:value={selected} disabled={busy}>{#each versions as version}<option value={version.id}>{version.versionNumber} · {version.name}</option>{/each}</select>
            <button class={button({ variant: "primary" })} disabled={busy || !selected || appState.isGameRunning || appState.isLaunching} onclick={() => confirm = true}>{busy ? uiText("ui.aebd32f10b997477") : uiText("ui.dd2c50a8e696b84d")}</button>
        </div>
        {#if confirm}
            <div role="group" aria-label={uiText("ui.9512e28d505b97ce")} class="rounded-xl border border-warning/30 bg-warning/10 p-4">
                <p class="mb-3 text-sm text-fg">{uiText("ui.68470d5eae6bc777")} {versions.find(version => version.id === selected)?.versionNumber}?</p>
                <div class="flex gap-2"><button class={button({ variant: "primary" })} disabled={busy} onclick={apply}>{uiText("ui.092559626a31ce6b")}</button><button class={button({ variant: "secondary" })} disabled={busy} onclick={() => confirm = false}>{uiText("common.cancel")}</button></div>
            </div>
        {/if}
    </div>
</details>
