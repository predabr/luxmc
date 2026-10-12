<script lang="ts">
    import { modpackUpdates } from '$lib/stores/modpackUpdates.svelte';
    import { modpackInstallation } from '$lib/stores/modpackInstallation.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { goto } from '$app/navigation';
    import { LoaderCircle, CheckCircle2, AlertCircle, ArrowUpRight, X } from 'lucide-svelte';
    import { button } from '$lib/components/ui/button';
    const task = $derived(modpackUpdates.task);
</script>

{#if task && !modpackInstallation.state.busy}
    <aside aria-label={t('packFollowup.updating')} data-modpack-update-status class="fixed bottom-5 right-5 z-[149] w-[min(420px,calc(100vw-40px))] overflow-hidden rounded-3xl border border-border bg-bg-elevated text-fg shadow-elevated">
        <div class="flex items-start gap-3 p-4">
            {#if task.busy}<LoaderCircle class="mt-1 h-5 w-5 shrink-0 animate-spin text-brand-400" />{:else if task.error}<AlertCircle class="mt-1 h-5 w-5 shrink-0 text-danger" />{:else}<CheckCircle2 class="mt-1 h-5 w-5 shrink-0 text-success" />{/if}
            <div class="min-w-0 flex-1"><p class="text-[10px] uppercase tracking-wider text-fg-muted">{task.busy ? t('packFollowup.updating') : task.complete ? t('packFollowup.updated') : t('downloadsDesign.failed')}</p><p class="mt-1 truncate text-sm font-semibold">{task.name}</p><p role="status" class="mt-2 text-xs leading-relaxed text-fg-muted">{task.error || task.text}</p>{#if task.busy}<p class="mt-2 text-[11px] text-fg-subtle">{t('downloadsDesign.navigate')}</p>{/if}</div>
            {#if !task.busy}<button type="button" class={button({variant:'ghost',size:'icon'})} aria-label={t('common.close')} onclick={() => modpackUpdates.dismiss()}><X class="h-4 w-4" /></button>{/if}
        </div>
        <div class="flex items-center justify-between gap-3 border-t border-border px-4 py-3"><button type="button" class={button({variant:'ghostBrand',size:'sm'})} onclick={() => goto(`/instances/${task.profileId}`)}>{t('downloadsDesign.open')}<ArrowUpRight class="h-4 w-4" /></button><span class="text-xs tabular-nums text-fg-muted">{task.percent}%</span></div>
        <div role="progressbar" aria-label={t('packFollowup.updating')} aria-valuemin="0" aria-valuemax="100" aria-valuenow={task.percent} aria-valuetext={task.text} class="h-1.5 overflow-hidden bg-bg-subtle"><div class="h-full bg-brand-400 transition-[width] duration-200" style:width={`${task.percent}%`}></div></div>
    </aside>
{/if}
