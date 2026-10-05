<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { goto } from "$app/navigation";
    import { settings, type AppSettings } from "$lib/stores/settings.svelte";
    import { schedulePersist, useSystemLocale } from "$lib/stores/persistence.svelte";
    import { friendsState } from "$lib/stores/friends.svelte";
    import { account } from "$lib/stores/account.svelte";
    import { javaScan } from "$lib/api/java";
    import { appOpenDataDirectory, storageFullReport } from "$lib/api/system";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    import { playSound, startSoundscape, stopSoundscape } from "$lib/utils/sound";
    import { Volume2, BellRing, MonitorCog, ShieldCheck, ChevronDown } from "lucide-svelte";

    let { section }: { section: string } = $props();
    let busy = $state(false);
    let result = $state("");
    function patch(value: Partial<AppSettings>) { settings.patch(value); schedulePersist(); }
    async function perform(action: () => Promise<void>) {
        if (busy) return;
        busy = true; result = "";
        try { await action(); } catch (error) { toast(String(error), "error"); }
        finally { busy = false; }
    }
    async function detectLanguage() {
        await useSystemLocale();
    }
    async function inspectJava() {
        const scan = await javaScan();
        result = scan.runtimes.filter(runtime => runtime.installed).map(runtime => `Java ${runtime.major}: ${runtime.versionString || "instalado"}`).join(" · ") || uiText("ui.4ac2ca9b539ee07d");
    }
    async function copyStorage() {
        const report = await storageFullReport();
        await navigator.clipboard.writeText(JSON.stringify({ totalBytes: report.totalBytes, categories: report.categories.map(item => ({ category: item.category, bytes: item.bytes })), warnings: report.warnings?.length || 0 }, null, 2));
        result = uiText("settings.refinement.storageCopied");
    }
    function toggleSound(enabled: boolean) {
        patch({ soundEnabled: enabled });
        if (!enabled) stopSoundscape();
        else if (settings.value.soundscapesEnabled === true) startSoundscape("overworld");
    }
</script>

<section class="surface-glass space-y-4 p-6" aria-label={uiText("ui.06c23075d12f7697")}>
    <h3 class="text-base font-semibold text-fg">{uiText(section === "general" ? "settings.refinement.feedbackTitle" : "ui.5d951376c42f3cf3")}</h3>
    {#if section === "general"}
        <div class="grid gap-4 md:grid-cols-2">
            <label class="flex items-center gap-4 rounded-2xl border border-border bg-fg/[0.025] p-4 text-sm text-fg"><span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-brand-500/10 text-brand-400"><BellRing class="h-5 w-5" /></span><span class="flex-1"><span class="block font-semibold">{uiText("ui.9a3fc17fda62df41")}</span><span class="mt-1 block text-xs text-fg-muted">{uiText("settings.autoCheckDesc")}</span></span><input class="h-5 w-5 shrink-0 accent-brand-500" type="checkbox" checked={settings.value.autoCheckUpdates !== false} onchange={event => patch({ autoCheckUpdates: event.currentTarget.checked })} /></label>
            <label class="flex items-center gap-4 rounded-2xl border border-border bg-fg/[0.025] p-4 text-sm text-fg"><span class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-brand-500/10 text-brand-400"><Volume2 class="h-5 w-5" /></span><span class="flex-1"><span class="block font-semibold">{uiText("settings.refinement.soundEnabled")}</span><span class="mt-1 block text-xs text-fg-muted">{uiText("settings.refinement.soundEnabledDesc")}</span></span><input class="h-5 w-5 shrink-0 accent-brand-500" type="checkbox" checked={settings.value.soundEnabled !== false} onchange={event => toggleSound(event.currentTarget.checked)} /></label>
            <label class="space-y-3 rounded-2xl border border-border bg-fg/[0.025] p-4 text-sm text-fg"><span class="flex items-center justify-between gap-2 font-semibold">{uiText("settings.refinement.soundVolume")}<span class="text-xs text-brand-400">{Math.round((settings.value.sfxVolume ?? 0.5) * 100)}%</span></span><input class="w-full accent-brand-500" type="range" min="0" max="100" step="1" disabled={settings.value.soundEnabled === false} value={(settings.value.sfxVolume ?? 0.5) * 100} oninput={event => patch({ sfxVolume: Number(event.currentTarget.value) / 100 })} /><button type="button" class={button({ variant: "ghost", size: "sm" })} disabled={settings.value.soundEnabled === false} onclick={() => playSound("chime")}><Volume2 class="h-4 w-4" />{uiText("settings.refinement.previewSound")}</button></label>
            <label class="space-y-3 rounded-2xl border border-border bg-fg/[0.025] p-4 text-sm text-fg"><span class="flex items-center gap-2 font-semibold"><MonitorCog class="h-4 w-4 text-brand-400" />{uiText("ui.87ec6e29dbf3075c")}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.showLogsOnLaunch || "on_crash"} onchange={event => patch({ showLogsOnLaunch: event.currentTarget.value as AppSettings["showLogsOnLaunch"] })}><option value="never">{uiText("ui.1d3f5f89555201e3")}</option><option value="on_crash">{uiText("ui.b306008525bde5ee")}</option><option value="always">{uiText("ui.80d135853d9bc433")}</option></select></label>
        </div>
        <details class="rounded-2xl border border-border p-4"><summary class="flex cursor-pointer list-none items-center gap-3 text-sm font-semibold text-fg"><ShieldCheck class="h-4 w-4 text-brand-400" />{uiText("settings.refinement.advancedDiscord")}<ChevronDown class="ml-auto h-4 w-4 text-fg-muted" /></summary><label class="mt-4 block space-y-2 text-sm text-fg">{uiText("ui.35757b14de5716e5")}<input class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.discordClientId || ""} placeholder={uiText("ui.aead1b74ef7345e3")} inputmode="numeric" pattern={"[0-9]{17,20}"} onchange={event => { const value = event.currentTarget.value.trim(); if (!value || /^[0-9]{17,20}$/.test(value)) patch({ discordClientId: value || undefined }); else toast(uiText("ui.8f40fa1eb5748ca5"), "error"); }} /></label></details>
    {:else if section === "accounts"}
        <p class="text-sm text-fg-muted">{uiText("ui.7a77f2326f2276ee")} {friendsState.me?.username || uiText("ui.78ec4bc1530baf6a")}{uiText("ui.0ac9218c093d0db1")}</p>
        <div class="flex flex-wrap gap-2"><button class={button({ variant: "primary" })} disabled={busy || !account.value} onclick={() => perform(() => friendsState.connect())}>{uiText("ui.f82071b48dd8ef1e")}</button><button class={button({ variant: "secondary" })} onclick={() => friendsState.disconnect()}>{uiText("ui.1c01528b95074e18")}</button><button class={button({ variant: "secondary" })} onclick={() => goto("/skins")}>{uiText("ui.aec5b769995e7e6b")}</button></div>
    {:else if section === "language"}
        <p class="text-sm text-fg-muted">{uiText("ui.b04e86800b0ac55e")}</p>
        <button class={button({ variant: "secondary" })} onclick={detectLanguage}>{uiText("ui.91ad8f507917c96a")}</button>
    {:else if section === "appearance"}
        <label class="flex flex-wrap items-center justify-between gap-3 text-sm text-fg">{uiText("ui.a8c62850bacc1368")}
            <select class="rounded-xl border border-border bg-bg-elevated p-2" value={settings.value.rightSidebarWidth || 320} onchange={event => patch({ rightSidebarWidth: Number(event.currentTarget.value) as 280 | 320 | 360 })}><option value="280">{uiText("ui.a5558c8bd5102eba")}</option><option value="320">{uiText("ui.b94129d5d492a748")}</option><option value="360">{uiText("ui.9b4a498255344398")}</option></select>
        </label>
        <label class="flex items-center gap-3 text-sm text-fg"><input type="checkbox" checked={settings.value.pauseSkinWhileGaming === true} onchange={event => patch({ pauseSkinWhileGaming: event.currentTarget.checked })} /> {uiText("ui.ef0a39bb77c3620e")}</label>
        <div class="grid gap-5 sm:grid-cols-2">
            <label class="space-y-2 text-sm text-fg">{uiText("ui.5d5f6f158a48dbf9")} {settings.value.interfaceOpacity ?? 18}%<input class="w-full accent-brand-500" type="range" min="5" max="90" step="1" value={settings.value.interfaceOpacity ?? 18} oninput={event => patch({interfaceOpacity:Number(event.currentTarget.value)})} /><span class="block text-xs text-fg-muted">{uiText("ui.113be20272b6810c")}</span></label>
            <label class="space-y-2 text-sm text-fg">{uiText("ui.72af96978e4d0d5a")} {settings.value.wallpaperDim ?? 35}%<input class="w-full accent-brand-500" type="range" min="0" max="85" step="1" value={settings.value.wallpaperDim ?? 35} oninput={event => patch({wallpaperDim:Number(event.currentTarget.value)})} /></label>
            <label class="space-y-2 text-sm text-fg">{uiText("ui.58cb5d0c620f3ad0")}<select class="block w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.density} onchange={event => patch({density:event.currentTarget.value as AppSettings["density"]})}><option value="compact">{uiText("settings.densityCompact")}</option><option value="comfortable">{uiText("settings.densityComfortable")}</option><option value="spacious">{uiText("ui.160a9d8bb491ae50")}</option></select></label>
            <label class="space-y-2 text-sm text-fg">{uiText("ui.eb449bd38b044ad5")} {settings.value.capeWindStrength ?? 50}%<input class="w-full accent-brand-500" type="range" min="0" max="100" value={settings.value.capeWindStrength ?? 50} oninput={event => patch({capeWindStrength:Number(event.currentTarget.value)})} /></label>
        </div>
        <div class="flex flex-wrap gap-5 text-sm text-fg"><label class="flex items-center gap-2"><input type="checkbox" checked={settings.value.capePhysics !== false} onchange={event => patch({capePhysics:event.currentTarget.checked})} />{uiText("ui.fc42c1c0722aafeb")}</label><label class="flex items-center gap-2"><input type="checkbox" checked={settings.value.pauseWallpaperOnBlur !== false} onchange={event => patch({pauseWallpaperOnBlur:event.currentTarget.checked})} />{uiText("ui.6e9ef1f332918426")}</label><label class="flex items-center gap-2"><input type="checkbox" checked={settings.value.animatedWallpaperBlur === true} onchange={event => patch({animatedWallpaperBlur:event.currentTarget.checked})} />{uiText("ui.8e849681af5a75cb")}</label></div>
        <button class={button({variant:"secondary"})} onclick={() => patch({interfaceOpacity:18,wallpaperDim:35,capePhysics:true,capeWindStrength:50,density:"comfortable"})}>{uiText("ui.4503cbbc850a42f0")}</button>
    {:else if section === "java"}
        <div class="flex flex-wrap gap-2"><button class={button({ variant: "secondary" })} disabled={busy} onclick={() => perform(inspectJava)}>{uiText("ui.8543ab5386d25fe1")}</button><button class={button({ variant: "secondary" })} onclick={() => { patch({ javaPath: undefined, jvmArgs: undefined }); result = uiText("ui.6be94e582bad3226"); }}>{uiText("ui.1bb75d47d06925a8")}</button></div>
    {:else if section === "commands"}
        <p class="text-sm text-fg-muted">{uiText("ui.4b053dce72c87bbd")}</p>
        <button class={button({ variant: "secondary" })} onclick={() => goto("/instances")}>{uiText("ui.d90583bb2a81fd1e")}</button>
    {:else if section === "privacy"}
        <label class="flex items-center gap-3 text-sm text-fg"><input type="checkbox" checked={settings.value.shareCustomAvatar !== false} onchange={event => { patch({ shareCustomAvatar: event.currentTarget.checked }); void friendsState.refresh(); }} /> {uiText("ui.393b90990b7f5c64")}</label>
        <label class="flex items-center gap-3 text-sm text-fg"><input type="checkbox" checked={settings.value.publishGameActivity !== false} onchange={event => { patch({ publishGameActivity: event.currentTarget.checked }); void friendsState.refresh(); }} /> {uiText("ui.fd71364ef5c40698")}</label>
        <p class="text-xs text-fg-muted">{uiText("ui.c852e745e86cc143")}</p>
    {:else if section === "runtime"}
        <div class="flex flex-wrap gap-2"><button class={button({ variant: "secondary" })} onclick={() => perform(appOpenDataDirectory)}>{uiText("ui.980424aa868d63ed")}</button><button class={button({ variant: "secondary" })} disabled={busy} onclick={() => perform(copyStorage)}>{uiText("ui.08a81ece3ee6af25")}</button></div>
    {/if}
    {#if result}<p role="status" class="text-sm text-fg-muted">{result}</p>{/if}
</section>
