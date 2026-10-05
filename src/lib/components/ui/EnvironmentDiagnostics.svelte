<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { untrack } from "svelte";
    import { getSystemSpecs, envCheck, type SystemSpecs } from "$lib/api/system";
    import { javaScan } from "$lib/api/java";
    import { doctorInstanceReadiness, type InstanceReadiness } from "$lib/api/doctor";
    import type { EnvCheckResult, JavaScanResult } from "$lib/api/types";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { focusTrap } from "$lib/utils/focusTrap";
    import { toast } from "$lib/stores/toasts.svelte";
    import { button } from "$lib/components/ui/button";
    import { X, RefreshCw, ClipboardCopy, Monitor, ShieldCheck } from "lucide-svelte";

    let { open = false, onClose }: { open?: boolean; onClose: () => void } = $props();
    let loading = $state(false);
    let specs = $state<SystemSpecs | null>(null);
    let environment = $state<EnvCheckResult | null>(null);
    let java = $state<JavaScanResult | null>(null);
    let readiness = $state<InstanceReadiness | null>(null);
    let errors = $state<string[]>([]);
    let profileId = $state("");
    let generation = 0;
    const selectedProfile = $derived(profiles.list.find(profile => profile.id === profileId));
    const report = $derived(JSON.stringify({
        launcher: specs?.launcherVersion,
        system: specs ? { os: specs.osDistro, kernel: specs.kernelVersion, arch: specs.arch, ramMb: specs.totalRamMb, gpu: specs.gpuRenderer, vendor: specs.gpuVendor } : null,
        environment: environment?.issues.map(issue => issue.code),
        java: java?.runtimes.map(runtime => ({ major: runtime.major, installed: runtime.installed, version: runtime.versionString, system: runtime.isSystem })),
        instance: selectedProfile ? { version: selectedProfile.mcVersion, loader: selectedProfile.loader, ready: readiness?.ready, requiredJava: readiness?.requiredJava, allocatedRamMb: readiness?.allocatedRamMb, recommendedRamMb: readiness?.recommendedRamMb, blockers: readiness?.blockers.length, warnings: readiness?.warnings.length } : null,
        incompleteChecks: errors.length
    }, null, 2));

    async function refresh() {
        const request = ++generation;
        loading = true;
        specs = null;
        environment = null;
        java = null;
        readiness = null;
        errors = [];
        const results = await Promise.allSettled([getSystemSpecs(), envCheck(), javaScan(), profileId ? doctorInstanceReadiness(profileId) : Promise.resolve(null)]);
        if (request !== generation || !open) return;
        const [systemResult, environmentResult, javaResult, instanceResult] = results;
        if (systemResult.status === "fulfilled") specs = systemResult.value;
        if (environmentResult.status === "fulfilled") environment = environmentResult.value;
        if (javaResult.status === "fulfilled") java = javaResult.value;
        if (instanceResult.status === "fulfilled") readiness = instanceResult.value;
        errors = results.flatMap((result, index) => result.status === "rejected" ? [`${[uiText("ui.f150afd3c5994d18"), uiText("ui.0d2b7ca65666383e"), "Java", uiText("ui.c641bdc857f2ba84")][index]}: ${String(result.reason)}`] : []);
        loading = false;
    }

    $effect(() => {
        if (open) {
            untrack(() => {
                profileId = profiles.activeId ?? profiles.list[0]?.id ?? "";
                void refresh();
            });
        } else {
            generation++;
            loading = false;
        }
    });

    async function copyReport() {
        try { await navigator.clipboard.writeText(report); toast(uiText("ui.a1bdb563a765ac41"), "success"); }
        catch { toast(uiText("ui.f88678a3b0b84003"), "error"); }
    }
</script>

<svelte:window onkeydown={event => { if (open && event.key === "Escape") onClose(); }} />

{#if open}
    <div class="fixed inset-0 z-[100] flex items-center justify-center bg-bg-overlay/80 p-5 backdrop-blur-md">
        <div use:focusTrap role="dialog" aria-modal="true" aria-labelledby="diagnostics-title" tabindex="-1" class="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-2xl border border-fg/10 bg-bg-elevated shadow-2xl">
            <header class="flex items-center gap-3 border-b border-fg/10 p-5">
                <Monitor class="h-5 w-5 text-brand-400" />
                <div class="flex-1"><h2 id="diagnostics-title" class="font-bold text-fg">{uiText("ui.d5986b954d27350d")}</h2><p class="mt-1 text-xs text-fg/50">{uiText("ui.ca59a545b559eddd")}</p></div>
                <button type="button" aria-label={uiText("ui.b38cfb4f3a35a1bb")} class={button({ variant: "ghost", size: "icon" })} onclick={onClose}><X class="h-4 w-4" /></button>
            </header>
            <div class="space-y-4 overflow-y-auto p-5" aria-busy={loading}>
                <label class="block text-xs font-semibold text-fg/70">{uiText("ui.c641bdc857f2ba84")}
                    <select bind:value={profileId} onchange={() => void refresh()} disabled={loading} class="mt-2 w-full rounded-xl border border-fg/10 bg-bg-subtle p-3 text-sm text-fg">
                        <option value="">{uiText("ui.f0dc6409bb1ff454")}</option>
                        {#each profiles.list as profile (profile.id)}<option value={profile.id}>{profile.name} · {profile.mcVersion}</option>{/each}
                    </select>
                </label>
                {#if loading}<p role="status" class="text-sm text-fg/60">{uiText("ui.de1b55e545971f8a")}</p>{/if}
                {#if specs}
                    <div class="grid grid-cols-2 gap-3 rounded-xl border border-fg/10 bg-bg-subtle p-4 text-xs">
                        <div><span class="text-fg/50">{uiText("ui.f150afd3c5994d18")}</span><p class="mt-1 font-semibold text-fg">{specs.osDistro} · {specs.arch}</p></div>
                        <div><span class="text-fg/50">{uiText("ui.8d30a75d5be82b67")}</span><p class="mt-1 font-semibold text-fg">{(specs.totalRamMb / 1024).toFixed(1)} GB</p></div>
                        <div class="col-span-2"><span class="text-fg/50">GPU</span><p class="mt-1 font-semibold text-fg">{specs.gpuRenderer}</p></div>
                    </div>
                {/if}
                {#if java}<div class="flex flex-wrap gap-2">{#each java.runtimes as runtime}<span class="rounded-lg border border-fg/10 px-3 py-2 text-xs {runtime.installed ? 'text-emerald-400' : 'text-fg/50'}">Java {runtime.major} · {runtime.installed ? uiText("ui.9b99805faf33cdc0") : uiText("ui.d28f68e13f71c267")}</span>{/each}</div>{/if}
                {#if environment}
                    {#each environment.issues as issue}<div class="rounded-xl border border-amber-400/20 bg-amber-400/5 p-3 text-xs"><p class="font-semibold text-amber-300">{issue.message}</p><p class="mt-1 text-fg/60">{issue.fix}</p></div>{/each}
                {/if}
                {#if readiness}
                    <div class="rounded-xl border border-fg/10 p-4"><p class="flex items-center gap-2 text-sm font-semibold {readiness.ready ? 'text-emerald-400' : 'text-amber-300'}"><ShieldCheck class="h-4 w-4" />{readiness.ready ? uiText("ui.7b212c9f20a6446e") : uiText("ui.4d84991abe5e32b5")}</p><p class="mt-2 text-xs text-fg/60">Java {readiness.requiredJava} {uiText("ui.3064d0419bfbd3d3")} {readiness.allocatedRamMb} {uiText("ui.c32de530125f3ab9")} {readiness.recommendedRamMb} MB</p>{#each [...readiness.blockers, ...readiness.warnings] as message}<p class="mt-2 text-xs text-fg/70">{message}</p>{/each}</div>
                {/if}
                {#each errors as error}<p role="alert" class="rounded-xl border border-red-400/20 p-3 text-xs text-red-300">{error}</p>{/each}
                {#if !loading && (specs || java || environment)}<details class="text-xs text-fg/60"><summary class="cursor-pointer">{uiText("ui.f034c826dc97d834")}</summary><p class="mt-2">{uiText("ui.88ab8b3f5fe4719c")}</p><pre class="mt-3 overflow-x-auto rounded-xl bg-bg-subtle p-3 text-[11px]">{report}</pre></details>{/if}
            </div>
            <footer class="flex gap-2 border-t border-fg/10 p-4">
                <button type="button" disabled={loading} class={button({ variant: "secondary", size: "sm" })} onclick={() => void refresh()}><RefreshCw class="mr-2 h-4 w-4" />{uiText("ui.0faf36fe47518b6f")}</button>
                <button type="button" disabled={loading || (!specs && !java && !environment)} class={button({ variant: "primary", size: "sm" })} onclick={copyReport}><ClipboardCopy class="mr-2 h-4 w-4" />{uiText("ui.9bb126076df88e9d")}</button>
            </footer>
        </div>
    </div>
{/if}
