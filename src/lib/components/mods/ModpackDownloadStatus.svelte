<script lang="ts">
    import { modpackInstallation } from "$lib/stores/modpackInstallation.svelte";
    import { contentActivity } from '$lib/stores/contentActivity.svelte';
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { goto } from '$app/navigation';
    import { Download, LoaderCircle, X, CheckCircle2, AlertCircle, Check, Clock3 } from "lucide-svelte";
    import { listenDownloadProgress } from "$lib/api";
    import { elapsedTime, installationProgressPercent, installationStep } from '$lib/utils/installProgress';
    import { button } from "$lib/components/ui/button";
    import { onMount } from 'svelte';
    let transferPercent = $state(-1);
    const task = $derived(modpackInstallation.state);
    onMount(() => {
        let disposed = false;
        let unlisten: (() => void) | undefined;
        void modpackInstallation.initialize().catch(() => {});
        void listenDownloadProgress(progress => {
            if (!task.busy || task.phase !== 'package' || task.percent >= 0 || !['downloading', 'done'].includes(progress.phase)) return;
            transferPercent = progress.totalBytes > 0
                ? Math.min(100, Math.round(progress.bytesDownloaded / progress.totalBytes * 100))
                : progress.total > 0 ? Math.min(100, Math.round(progress.completed / progress.total * 100)) : -1;
        }).then(stop => { if (disposed) stop(); else unlisten = stop; })
            .catch(error => console.error("modpack download progress listener failed", error));
        return () => { disposed = true; unlisten?.(); };
    });
    let now = $state(Date.now());
    const step = $derived(installationStep(task.phase, !!task.profileId && !task.busy && !task.error));
    const labels = $derived(['package', 'files', 'validation', 'ready'].map(key => uiText(`performanceUi.${key}`)));
    const expanded = $derived(modpackInstallation.panelExpanded);
    const percent = $derived(installationProgressPercent(task.phase, task.percent, transferPercent));
    const activeStepProgress = $derived(Math.min(100, Math.max(0, (percent - step * 25) * 4)));
    function segmentProgress(index: number) {
        return index < step ? 100 : index === step ? activeStepProgress : 0;
    }
    $effect(() => {
        task.startedAt;
        transferPercent = -1;
    });
    $effect(() => {
        if (!task.busy) return;
        now = Date.now();
        const timer = setInterval(() => now = Date.now(), 1000);
        return () => clearInterval(timer);
    });
</script>
{#if !contentActivity.updateProfileId && (task.item || modpackInstallation.jobs.length)}
    <aside aria-label={uiText('workshop.queue')} data-installation-status data-expanded={expanded} class="max-h-[min(70vh,34rem)] w-full overflow-y-auto rounded-3xl border border-brand-400/30 bg-bg-elevated text-fg shadow-elevated">
        <button type="button" class="flex w-full items-center gap-3 p-4 text-left" onclick={() => modpackInstallation.togglePanel()} aria-expanded={expanded}>
            {#if task.item?.iconUrl}<img src={task.item.iconUrl} alt="" class="h-10 w-10 rounded-xl object-contain" />{:else}<Download class="h-6 w-6 text-brand-400" />{/if}
            <span class="min-w-0 flex-1"><span class="block text-[10px] uppercase tracking-wider text-fg-muted">{!task.item ? uiText('workshop.queue') : task.paused ? uiText('workshop.paused') : task.busy ? uiText('downloadsDesign.title') : task.error ? uiText('downloadsDesign.failed') : uiText('downloadsDesign.complete')}</span><span class="mt-1 block truncate text-sm font-semibold">{task.item ? task.name : uiText('workshop.queueCount',{count:modpackInstallation.jobs.length})}</span></span><span class="min-w-[3.5rem] rounded-lg border border-brand-400/25 bg-brand-500/10 px-2 py-1 text-right text-sm font-bold tabular-nums text-brand-400">{`${percent}%`}</span>
        </button>
        {#if expanded && task.item}
            <ol class="installation-timeline grid grid-cols-4 gap-1 border-t border-border px-4 pb-3 pt-4" aria-label={uiText('performanceUi.stages')}>
                {#each labels as label, index}
                    <li class="installation-step flex flex-col items-center gap-2 text-center" data-active={index === step && task.busy} aria-current={index === step ? 'step' : undefined}>
                        <span class="flex h-7 w-7 items-center justify-center rounded-full border text-[11px] font-semibold {index < step ? 'border-success/25 bg-success/10 text-success' : index === step ? 'border-brand-400/40 bg-brand-500/15 text-brand-400' : 'border-border bg-bg-subtle text-fg-subtle'}">
                            {#if index < step}<Check class="h-3.5 w-3.5" />{:else}{index + 1}{/if}
                        </span>
                        <span class="text-[10px] font-medium {index <= step ? 'text-fg' : 'text-fg-subtle'}">{label}</span>
                    </li>
                {/each}
            </ol>
            {#if task.startedAt > 0}
                <p class="flex items-center justify-center gap-1.5 pb-3 text-[10px] tabular-nums text-fg-muted"><Clock3 class="h-3 w-3" />{uiText('performanceUi.elapsed', {time: elapsedTime(task.startedAt, task.finishedAt || now)})}</p>
            {/if}
        {/if}
        {#if expanded && task.item}<div class="space-y-3 border-t border-border px-4 pb-4 pt-3"><p role="status" class="flex items-start gap-2 text-xs leading-relaxed text-fg-muted">{#if task.busy}<LoaderCircle class="mt-0.5 h-3.5 w-3.5 shrink-0 animate-spin motion-reduce:animate-none" />{:else if task.error}<AlertCircle class="h-4 w-4 shrink-0 text-danger" />{:else}<CheckCircle2 class="h-4 w-4 shrink-0 text-success" />{/if}{task.error || task.text}</p>{#if task.busy}<p class="text-[11px] text-fg-subtle">{uiText('downloadsDesign.navigate')}</p><button type="button" class={button({variant:'ghostDanger',size:'sm',block:true})} disabled={task.cancelling} onclick={() => modpackInstallation.cancel()}><X class="h-3.5 w-3.5" />{task.cancelling ? uiText('downloadsDesign.cancelling') : uiText('common.cancel')}</button>{:else}<div class='flex gap-2'>{#if task.profileId}<button type='button' class={button({variant:'primary',size:'sm',class:'flex-1'})} onclick={() => { void goto(`/instances/${task.profileId}`); modpackInstallation.dismiss(); }}>{uiText('downloadsDesign.open')}</button>{/if}<button type='button' class={button({variant:'secondary',size:'sm',class:'flex-1'})} onclick={() => modpackInstallation.dismiss()}>{uiText('common.close')}</button></div>{/if}</div>{/if}
        {#if expanded && modpackInstallation.jobs.length}<div class="max-h-56 space-y-2 overflow-y-auto border-t border-border p-3">
            {#if task.busy}<button type="button" class={button({variant:'secondary',size:'sm',block:true})} disabled={task.cancelling} onclick={() => modpackInstallation.pause()}>{uiText('workshop.pause')}</button>{/if}
            {#each modpackInstallation.jobs as job (job.id)}<div class="rounded-xl border border-border bg-bg-subtle p-3"><p class="truncate text-xs font-semibold">{job.name}</p><p class="mt-1 text-[11px] text-fg-muted">{job.status === 'running' ? uiText('workshop.running') : job.status === 'paused' ? uiText('workshop.paused') : job.status === 'failed' ? uiText('workshop.failed') : uiText('workshop.queued')}</p>{#if job.error}<p class="mt-1 line-clamp-2 text-[11px] text-danger">{job.error}</p>{/if}{#if job.status !== 'running'}<div class="mt-2 flex gap-2"><button type="button" class={button({variant:'ghostBrand',size:'sm'})} onclick={() => modpackInstallation.resume(job.id)}>{uiText('workshop.resume')}</button><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={() => modpackInstallation.prioritize(job.id)}>{uiText('workshop.prioritize')}</button><button type="button" class={button({variant:'ghostDanger',size:'sm'})} onclick={() => modpackInstallation.remove(job.id)}>{uiText('workshop.remove')}</button></div>{/if}</div>{/each}
        </div>{/if}
        <div class="px-4 pb-3 pt-1.5">
            <div class="mb-1.5 grid grid-cols-4 gap-1">
                {#each labels as label, index}
                    <span class="truncate text-center text-[9px] font-medium {index <= step ? 'text-brand-300' : 'text-fg-subtle'}">{label}<span class="ml-1 text-fg-subtle">25%</span></span>
                {/each}
            </div>
            <div role="progressbar" aria-label={uiText('downloadsDesign.title')} aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent} aria-valuetext={`${task.text} (${percent}%)`} class="installation-track grid h-2.5 grid-cols-4 gap-1 overflow-hidden rounded-full p-0.5">
                {#each labels as _, index}
                    <span class="installation-segment relative overflow-hidden rounded-full" aria-hidden="true">
                        <span class="installation-progress absolute inset-y-0 left-0 rounded-full bg-gradient-to-r from-brand-600 via-brand-400 to-brand-300 transition-[width] duration-300" style:width={`${segmentProgress(index)}%`}></span>
                    </span>
                {/each}
            </div>
        </div>
    </aside>
{/if}

<style>
    .installation-timeline { position: relative; }
    .installation-track { background: rgb(var(--bg-subtle)); box-shadow: inset 0 1px 3px rgb(var(--shadow-color) / .2), 0 0 0 1px rgb(var(--brand-400) / .18); }
    .installation-segment { background: rgb(var(--border) / .35); }
    .installation-progress { box-shadow: 0 0 12px rgb(var(--brand-400) / .55); }
    .installation-timeline::before { content: ''; position: absolute; top: 29px; left: 15%; right: 15%; height: 1px; background: rgb(var(--border)); }
    .installation-step { position: relative; }
    .installation-step > span:first-child { box-shadow: 0 0 0 5px rgb(var(--bg-elevated)); }
    :global(html:not(.no-ui-motion):not(.no-anim)) .installation-step[data-active="true"] > span:first-child { animation: step-entry 340ms cubic-bezier(.16,1,.3,1); }
    @keyframes step-entry { from { transform: scale(.82); } to { transform: scale(1); } }
    @media (prefers-reduced-motion: reduce) { :global(html:not(.force-ui-motion)) .installation-progress, :global(html:not(.force-ui-motion)) .installation-step > span:first-child { animation: none !important; } }
</style>
