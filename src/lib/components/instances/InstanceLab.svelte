<script lang="ts">
import { translateUi as uiText, currentUiLocale } from "$lib/i18n/useTranslation.svelte";
    import { onMount } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { Archive, Activity, Bug, Download, RotateCcw, ShieldCheck, Sparkles } from "lucide-svelte";
    import Button from "$lib/components/ui/Button.svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { toast } from "$lib/stores/toasts.svelte";
    import { listenGameTelemetry } from "$lib/api/events";
    import {
        instanceBenchmarkImport, instanceBenchmarksList,
        instanceCapsuleCreate, instanceCapsuleRestore, instanceCapsulesList,
        instanceIsolationReport, instanceIsolationRestore, instanceIsolationStart, instanceIsolationStatus,
        type BenchmarkResult, type CapsuleInfo, type IsolationState
    } from "$lib/api/instanceLab";

    let { profileId }: { profileId: string } = $props();
    let isolation = $state<IsolationState | null>(null);
    let capsules = $state<CapsuleInfo[]>([]);
    let benchmarks = $state<BenchmarkResult[]>([]);
    let capsuleLabel = $state("");
    let benchmarkLabel = $state("");
    let busy = $state(false);
    let loading = $state(true);

    const comparison = $derived(benchmarks.length >= 2 ? {
        fps: benchmarks[0].averageFps - benchmarks[1].averageFps,
        low: benchmarks[0].lowOnePercentFps - benchmarks[1].lowOnePercentFps,
        drops: benchmarks[0].fpsDrops - benchmarks[1].fpsDrops
    } : null);

    function date(value: string) {
        return new Date(value).toLocaleString(currentUiLocale(), { dateStyle: "short", timeStyle: "short" });
    }

    async function refresh() {
        loading = true;
        try {
            [isolation, capsules, benchmarks] = await Promise.all([
                instanceIsolationStatus(profileId), instanceCapsulesList(profileId), instanceBenchmarksList(profileId)
            ]);
        } catch (error) {
            toast(uiText("ui.5b54cfbf91e10c0c", {arg0: (String(error))}), "error");
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        void refresh();
        let disposed = false;
        let unlisten: (() => void) | undefined;
        void listenGameTelemetry(summary => {
            if (disposed || summary.profileId !== profileId || !isolation || isolation.phase !== "testing") return;
            if (!summary.cleanExit && ![130, 137, 143].includes(summary.exitCode)) {
                void report(true, true);
            }
        }).then(stop => {
            if (disposed) stop(); else unlisten = stop;
        }).catch(() => {});
        return () => { disposed = true; unlisten?.(); };
    });

    async function startIsolation() {
        busy = true;
        try {
            isolation = await instanceIsolationStart(profileId);
            toast(uiText("ui.0fc152fc56cd6792"), "info");
        } catch (error) {
            toast(uiText("ui.7eaed94a7d05193a", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }

    async function report(crashed: boolean, automatic = false) {
        if (busy || !isolation || isolation.phase !== "testing") return;
        busy = true;
        try {
            isolation = await instanceIsolationReport(profileId, crashed);
            if (isolation.phase === "found") toast(uiText("ui.73295b7980c8df2a", {arg0: (isolation.suspects[0])}), "success");
            else toast(uiText("ui.0d9c9ed023479f26", {arg0: (automatic ? "Crash detectado. " : "")}), "info");
        } catch (error) {
            toast(uiText("ui.6fa1dbdcdf5cc032", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }

    async function restoreMods() {
        busy = true;
        try {
            await instanceIsolationRestore(profileId);
            isolation = null;
            toast(uiText("ui.3ea2274b1f1f0e58"), "success");
        } catch (error) {
            toast(uiText("ui.63c8bc3070176d47", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }

    async function createCapsule() {
        busy = true;
        try {
            const created = await instanceCapsuleCreate(profileId, capsuleLabel);
            capsules = [created, ...capsules];
            capsuleLabel = "";
            toast(uiText("ui.8d829330ffe4468f"), "success");
        } catch (error) {
            toast(uiText("ui.bf2b120f84608395", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }

    async function restoreCapsule(filename: string) {
        if (!window.confirm(uiText("ui.edf7d35a28ea475b"))) return;
        busy = true;
        try {
            const previous = await instanceCapsuleRestore(profileId, filename);
            capsules = [previous, ...capsules];
            await profiles.refresh();
            toast(uiText("ui.5c7d2620facc7cb3"), "success");
        } catch (error) {
            toast(uiText("ui.fe7c349cfdd0b11d", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }

    async function importBenchmark() {
        try {
            const selected = await open({ multiple: false, filters: [{ name: uiText("ui.4df84e0733508cbc"), extensions: ["csv"] }] });
            if (typeof selected !== "string") return;
            busy = true;
            const result = await instanceBenchmarkImport(profileId, benchmarkLabel, selected);
            benchmarks = [result, ...benchmarks];
            benchmarkLabel = "";
            toast(uiText("ui.c0557781e56b0c8e"), "success");
        } catch (error) {
            toast(uiText("ui.9182cb93125f5cf6", {arg0: (String(error))}), "error");
        } finally { busy = false; }
    }
</script>

<div class="space-y-5" aria-label={uiText("ui.e606941bce63b8e9")}>
    {#if loading}
        <div class="rounded-2xl border border-fg/10 bg-bg-elevated p-8 text-center text-sm text-fg-muted">{uiText("ui.f09b61a967dfd423")}</div>
    {:else}
        <div class="grid gap-5 xl:grid-cols-2">
            <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
                <div class="flex items-center gap-3"><Bug class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">{uiText("ui.cbdc1fbacc555cdb")}</h3><p class="text-xs text-fg-muted">{uiText("ui.eff27b6278a64d31")}</p></div></div>
                {#if isolation}
                    {#if isolation.phase === "found"}
                        <div class="rounded-xl border border-success/30 bg-success/10 p-4 text-sm text-fg"><ShieldCheck class="inline h-4 w-4 text-success mr-1" />{uiText("ui.7b7e467011a46fea")} <strong>{isolation.suspects[0]}</strong>{uiText("ui.48c22a0572ff868d")}</div>
                    {:else}
                        <div class="rounded-xl border border-brand-500/30 bg-brand-500/10 p-4 text-sm text-fg">{uiText("ui.d25fb53eaf4133bf")} {isolation.round}: {isolation.trialDisabled.length} {uiText("ui.bdd0ed7aae499af1")} <strong>{uiText("instances.play")}</strong> {uiText("ui.a38da57367575e4e")}</div>
                        <div class="flex flex-wrap gap-2">
                            <Button variant="primary" disabled={busy || appState.isGameRunning} onclick={() => report(false)}>{uiText("ui.24ca624640399451")}</Button>
                            <Button variant="danger" disabled={busy || appState.isGameRunning} onclick={() => report(true)}>{uiText("ui.f88aaedd12e9727a")}</Button>
                        </div>
                    {/if}
                    <Button variant="secondary" disabled={busy || appState.isGameRunning} onclick={restoreMods}><RotateCcw class="h-4 w-4" /> {uiText("ui.82a6cb4162f8d99a")}</Button>
                {:else}
                    <p class="text-xs leading-relaxed text-fg-muted">{uiText("ui.b4e9889d576c2332")}</p>
                    <Button variant="primary" disabled={busy || appState.isGameRunning} onclick={startIsolation}>{uiText("ui.1db6bc53b9ce594a")}</Button>
                {/if}
            </section>

            <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
                <div class="flex items-center gap-3"><Archive class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">{uiText("ui.bc9fedc12543e17f")}</h3><p class="text-xs text-fg-muted">{uiText("ui.4b27bf2d47a6cc1d")}</p></div></div>
                <div class="flex flex-wrap gap-2">
                    <input class="luxmc-control min-w-0 flex-1" aria-label={uiText("ui.4515ad9e34d4fbb5")} placeholder={uiText("ui.4515ad9e34d4fbb5")} maxlength="48" bind:value={capsuleLabel} />
                    <Button variant="primary" disabled={busy || appState.isGameRunning || !!isolation} onclick={createCapsule}><Archive class="h-4 w-4" /> {uiText("ui.34a299064e17005d")}</Button>
                </div>
                {#if capsules.length === 0}<p class="text-xs text-fg-muted">{uiText("ui.cfd71096ddc28528")}</p>{/if}
                <div class="max-h-60 space-y-2 overflow-y-auto">
                    {#each capsules as capsule (capsule.filename)}
                        <div class="flex items-center justify-between gap-3 rounded-xl border border-fg/10 bg-bg-subtle/70 p-3">
                            <div class="min-w-0"><div class="truncate text-sm font-semibold text-fg">{capsule.label}</div><div class="text-xs text-fg-muted">{date(capsule.createdAt)} · {(capsule.sizeBytes / 1048576).toFixed(1)} MB</div></div>
                            <Button variant="secondary" size="sm" disabled={busy || appState.isGameRunning || !!isolation} onclick={() => restoreCapsule(capsule.filename)}><RotateCcw class="h-3.5 w-3.5" /> {uiText("shortcutsModal.reset")}</Button>
                        </div>
                    {/each}
                </div>
            </section>
        </div>

        <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
            <div class="flex items-center gap-3"><Activity class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">{uiText("ui.744280c5889299f6")}</h3><p class="text-xs text-fg-muted">{uiText("ui.698a259982f9ec62")}</p></div></div>
            <p class="text-xs leading-relaxed text-fg-muted">{uiText("ui.ed2c271c365c14eb")}</p>
            <div class="flex flex-wrap gap-2">
                <input class="luxmc-control min-w-0 flex-1" aria-label={uiText("ui.469c37625b0b31fb")} placeholder={uiText("ui.358d1996c41f64f5")} maxlength="48" bind:value={benchmarkLabel} />
                <Button variant="primary" disabled={busy} onclick={importBenchmark}><Download class="h-4 w-4" /> {uiText("ui.f720ad8dae7ed442")}</Button>
            </div>
            {#if comparison}
                <div class="rounded-xl border border-brand-500/20 bg-brand-500/10 p-4 text-sm text-fg flex flex-wrap gap-x-6 gap-y-1">
                    <span>{uiText("ui.12038cf069364b7f")} <strong>{comparison.fps >= 0 ? '+' : ''}{comparison.fps.toFixed(1)}</strong></span>
                    <span>{uiText("ui.fd7adcb199c40d18")} <strong>{comparison.low >= 0 ? '+' : ''}{comparison.low.toFixed(1)}</strong></span>
                    <span>{uiText("ui.8f3368955ea9d941")} <strong>{comparison.drops >= 0 ? '+' : ''}{comparison.drops}</strong></span>
                    <span class="text-xs text-fg-muted">{uiText("ui.541dd2fbf71c8cba")}</span>
                </div>
            {/if}
            {#if benchmarks.length === 0}<p class="text-xs text-fg-muted">{uiText("ui.97943b237fa9eda1")}</p>{/if}
            <div class="grid gap-2 md:grid-cols-2">
                {#each benchmarks as result (result.id)}
                    <div class="rounded-xl border border-fg/10 bg-bg-subtle/70 p-3">
                        <div class="flex items-center gap-2"><Sparkles class="h-4 w-4 text-brand-400" /><strong class="truncate text-sm text-fg">{result.label}</strong></div>
                        <div class="mt-1 text-xs text-fg-muted">{date(result.createdAt)} · {result.samples} {uiText("ui.7af27baaf512f513")}</div>
                        <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-fg"><span>{result.averageFps.toFixed(1)} {uiText("ui.26c862a552544602")}</span><span>{result.lowOnePercentFps.toFixed(1)} {uiText("ui.88818e57a9bb8890")}</span><span>{result.fpsDrops} {uiText("ui.95da92a37294bf4f")}</span></div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>
