<script lang="ts">
    import { modpackInstallation } from "$lib/stores/modpackInstallation.svelte";
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { goto } from '$app/navigation';
    import { Download, LoaderCircle, X, CheckCircle2, AlertCircle } from "lucide-svelte";
    import { button } from "$lib/components/ui/button";
    import { onMount } from 'svelte';
    onMount(() => { void modpackInstallation.initialize().catch(() => {}); });
    let expanded = $state(true);
    const task = $derived(modpackInstallation.state);
</script>
{#if task.item || modpackInstallation.jobs.length}
    <aside aria-label={uiText('workshop.queue')} class="fixed bottom-5 right-5 z-[150] w-[min(400px,calc(100vw-40px))] overflow-hidden rounded-2xl border border-border bg-bg-elevated text-fg shadow-elevated">
        <button type="button" class="flex w-full items-center gap-3 p-4 text-left" onclick={() => expanded = !expanded} aria-expanded={expanded}>
            {#if task.item?.iconUrl}<img src={task.item.iconUrl} alt="" class="h-10 w-10 rounded-xl object-contain" />{:else}<Download class="h-6 w-6 text-brand-400" />{/if}
            <span class="min-w-0 flex-1"><span class="block text-[10px] uppercase tracking-wider text-fg-muted">{!task.item ? uiText('workshop.queue') : task.paused ? uiText('workshop.paused') : task.busy ? uiText('downloadsDesign.title') : task.error ? uiText('downloadsDesign.failed') : uiText('downloadsDesign.complete')}</span><span class="mt-1 block truncate text-sm font-semibold">{task.item ? task.name : uiText('workshop.queueCount',{count:modpackInstallation.jobs.length})}</span></span><span class="text-xs tabular-nums text-brand-400">{task.percent < 0 ? '…' : `${Math.max(0, task.percent).toFixed(0)}%`}</span>
        </button>
        {#if expanded && task.item}<div class="space-y-3 border-t border-border px-4 pb-4 pt-3"><p role="status" class="flex items-start gap-2 text-xs leading-relaxed text-fg-muted">{#if task.busy}<LoaderCircle class="mt-0.5 h-3.5 w-3.5 shrink-0 animate-spin motion-reduce:animate-none" />{:else if task.error}<AlertCircle class="h-4 w-4 shrink-0 text-danger" />{:else}<CheckCircle2 class="h-4 w-4 shrink-0 text-success" />{/if}{task.error || task.text}</p>{#if task.busy}<p class="text-[11px] text-fg-subtle">{uiText('downloadsDesign.navigate')}</p><button type="button" class={button({variant:'ghostDanger',size:'sm',block:true})} disabled={task.cancelling} onclick={() => modpackInstallation.cancel()}><X class="h-3.5 w-3.5" />{task.cancelling ? uiText('downloadsDesign.cancelling') : uiText('common.cancel')}</button>{:else}<div class='flex gap-2'>{#if task.profileId}<button type='button' class={button({variant:'primary',size:'sm',class:'flex-1'})} onclick={() => { void goto(`/instances/${task.profileId}`); modpackInstallation.dismiss(); }}>{uiText('downloadsDesign.open')}</button>{/if}<button type='button' class={button({variant:'secondary',size:'sm',class:'flex-1'})} onclick={() => modpackInstallation.dismiss()}>{uiText('common.close')}</button></div>{/if}</div>{/if}
        {#if expanded && modpackInstallation.jobs.length}<div class="max-h-56 space-y-2 overflow-y-auto border-t border-border p-3">
            {#if task.busy}<button type="button" class={button({variant:'secondary',size:'sm',block:true})} disabled={task.cancelling} onclick={() => modpackInstallation.pause()}>{uiText('workshop.pause')}</button>{/if}
            {#each modpackInstallation.jobs as job (job.id)}<div class="rounded-xl border border-border bg-bg-subtle p-3"><p class="truncate text-xs font-semibold">{job.name}</p><p class="mt-1 text-[11px] text-fg-muted">{job.status === 'running' ? uiText('workshop.running') : job.status === 'paused' ? uiText('workshop.paused') : job.status === 'failed' ? uiText('workshop.failed') : uiText('workshop.queued')}</p>{#if job.error}<p class="mt-1 line-clamp-2 text-[11px] text-danger">{job.error}</p>{/if}{#if job.status !== 'running'}<div class="mt-2 flex gap-2"><button type="button" class={button({variant:'ghostBrand',size:'sm'})} onclick={() => modpackInstallation.resume(job.id)}>{uiText('workshop.resume')}</button><button type="button" class={button({variant:'ghost',size:'sm'})} onclick={() => modpackInstallation.prioritize(job.id)}>{uiText('workshop.prioritize')}</button><button type="button" class={button({variant:'ghostDanger',size:'sm'})} onclick={() => modpackInstallation.remove(job.id)}>{uiText('workshop.remove')}</button></div>{/if}</div>{/each}
        </div>{/if}
        <div role="progressbar" aria-label={uiText('downloadsDesign.title')} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.max(0,task.percent)} class="h-1.5 bg-bg-subtle"><div class="h-full bg-brand-500 transition-[width] duration-200" style:width={`${Math.min(100,Math.max(0,task.percent))}%`}></div></div>
    </aside>
{/if}
