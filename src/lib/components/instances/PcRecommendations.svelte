<script lang="ts">
    import { onMount } from 'svelte';
    import { pcRecommendations, launcherResourceSample, type PcRecommendations } from '$lib/api/studio';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { Cpu, RefreshCw, ArrowUpRight } from 'lucide-svelte';
    let data=$state<PcRecommendations|null>(null);let busy=$state(false);let error=$state('');
    let usage=$state<Awaited<ReturnType<typeof launcherResourceSample>>|null>(null);
    async function measure(){if(busy)return;busy=true;try{usage=await launcherResourceSample();}catch(cause){error=String(cause);}finally{busy=false;}}
    async function load(){if(busy)return;busy=true;error='';try{data=await pcRecommendations();}catch(cause){error=String(cause);}finally{busy=false;}}
    onMount(()=>{void load();});
</script>
<section class="space-y-4 rounded-3xl border border-border bg-bg-elevated p-5">
    <div class="flex items-start justify-between gap-3"><div><h2 class="flex gap-2 text-lg font-semibold"><Cpu class="h-5 w-5 text-brand-400" />{t('studio.pc')}</h2><p class="mt-2 text-xs leading-relaxed text-fg-muted">{t('studio.pcHelp')}</p></div><button type="button" aria-label={t('workshop.refresh')} disabled={busy} class={button({variant:'ghost',size:'icon'})} onclick={()=>void load()}><RefreshCw class="h-4 w-4" /></button></div>
    {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
    <div class="flex flex-wrap items-center gap-3 rounded-xl border border-border bg-bg-subtle p-3"><button type="button" disabled={busy} class={button({variant:'secondary',size:'sm'})} onclick={()=>void measure()}>{t('studio.measureLauncher')}</button>{#if usage}<p role="status" class="text-xs tabular-nums text-fg-muted">{usage.cpuPercent.toFixed(1)}% CPU · {usage.ramMb} MB · {usage.processes} {t('studio.processes')}</p>{/if}<p class="text-[11px] leading-relaxed text-fg-muted">{t('studio.backgroundPolicy')}</p></div>
    {#if data}<p class="text-xs text-fg-muted">{t('studio.memoryNow')}: {(data.availableRamMb/1024).toFixed(1)} / {(data.totalRamMb/1024).toFixed(1)} GB</p><div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">{#each data.items as item}<a href={`/instances/${item.profileId}`} class="min-w-0 rounded-2xl border border-border bg-bg-subtle p-4 transition-colors hover:border-brand-400/50"><div class="flex gap-2"><h3 class="min-w-0 flex-1 truncate font-semibold">{item.name}</h3><ArrowUpRight class="h-4 w-4 text-brand-400" /></div><p class="mt-3 text-xs font-medium {item.fitsNow?'text-success':'text-warning'}">{item.fitsNow?t('studio.fits'):t('studio.tight')}</p><div class="mt-4 grid grid-cols-2 gap-2 text-xs"><div><p class="text-fg-muted">{t('workshop.peakRam')}</p><p class="mt-1 font-semibold">{(item.peakRamMb/1024).toFixed(1)} GB</p></div><div><p class="text-fg-muted">{t('studio.ramSuggestion')}</p><p class="mt-1 font-semibold">{(item.suggestedRamMb/1024).toFixed(1)} GB</p></div></div><p class="mt-4 text-[11px] text-fg-muted">{item.samples} {t('studio.measuredSessions')} · {item.preparationSeconds?.toFixed(1) || '—'} s</p></a>{/each}</div>{#if !data.items.length}<p class="rounded-2xl border border-dashed border-border p-5 text-sm leading-relaxed text-fg-muted">{t('studio.noMeasurements')}</p>{/if}{/if}
</section>
