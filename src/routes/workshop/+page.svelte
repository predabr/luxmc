<script lang="ts">
    import { onMount } from 'svelte';
    import DependencyMap from '$lib/components/instances/DependencyMap.svelte';
    import PcRecommendations from '$lib/components/instances/PcRecommendations.svelte';
    import { Activity, Palette, ShieldCheck, HardDrive, ArrowLeft, Download, Upload, RefreshCw } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import { button } from '$lib/components/ui/button';
    import { profiles } from '$lib/stores/profiles.svelte';
    import { themeStore as theme } from '$lib/stores/theme.svelte';
    import { settings } from '$lib/stores/settings.svelte';
    import { settingsSet, doctorInstanceReadiness, doctorRepairAll, instanceListWorldBackups, type InstanceReadiness, type WorldBackupEntry } from '$lib/api';
    import { themeFileChoose, themeFileDestination, supportFileDestination, exportSupportReport, validateModpack, performanceHistory, supportReport, themeExport, themeImport, restoreWorld, type PerformanceEntry, type SupportReport } from '$lib/api/experience';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    let history = $state<PerformanceEntry[]>([]);
    let selected = $state(profiles.activeId || '');
    $effect(() => { if (!profiles.list.some(profile => profile.id === selected)) selected=(profiles.list.some(profile => profile.id === profiles.activeId) ? profiles.activeId : null) || profiles.list[0]?.id || ''; });
    let readiness = $state<InstanceReadiness | null>(null);
    let report = $state<SupportReport | null>(null);
    let backups = $state<WorldBackupEntry[]>([]);
    let busy = $state(false);
    let error = $state('');
    let success = $state('');
    let pendingRestore = $state<WorldBackupEntry | null>(null);
    const summaries = $derived(profiles.list.map(profile => {
        const entries = history.filter(entry => entry.profileId === profile.id);
        const prep = entries.filter(entry => entry.metrics.kind === 'preparation');
        const sessions = entries.filter(entry => entry.metrics.kind === 'session');
        return {profile, launches:prep.length, average:prep.length ? prep.reduce((sum,entry) => sum + (entry.metrics.seconds || 0),0)/prep.length : null, peak:sessions.length ? Math.max(...sessions.map(entry => entry.metrics.peakRamMb || 0)) : null};
    }));
    async function action(work: () => Promise<void>) { if (busy) return; busy=true; error=''; success=''; try { await work(); } catch (cause) { error=String(cause); } finally { busy=false; } }
    async function refresh() { history = await performanceHistory(); }
    onMount(() => { void action(refresh); });
    async function check() { const id = selected; const value = await validateModpack(id); if (id === selected) readiness=value; }
    async function loadBackups() { const id = selected; const value = await instanceListWorldBackups(id); if (id === selected) backups=value; }
    async function exportTheme() { const path = await themeFileDestination(); if (path) { await themeExport(path); success=t('workshop.themeSaved'); } }
    async function importTheme() { const path = await themeFileChoose(); if (typeof path === 'string') { const value = await themeImport(path); settings.patch(value); if(value.theme) theme.setTheme(value.theme,false); if(value.accentTheme) theme.setAccent(value.accentTheme,false); if(value.customBackground) theme.setBackground(value.customBackground,false); if(value.customWallpaperUrl) theme.setCustomWallpaper(value.customWallpaperUrl,value.customWallpaperType,undefined,false); else { theme.clearCustomWallpaper(false); theme.setBackground(value.customBackground || "obsidian",false); } success=t('workshop.themeApplied'); } }
    async function updateBackup(enabled: boolean,limit?: number) { const value={autoBackup:enabled,backupLimitGb:Math.min(100,Math.max(1,limit || settings.value.backupLimitGb || 5))}; await settingsSet(value); settings.patch(value); }
    async function diagnose() { const id=selected; const result=await supportReport(id); if (id===selected) report=result; }
    async function exportReport() { if (!report) return; const snapshot=$state.snapshot(report); const path=await supportFileDestination(); if(path) await exportSupportReport(path,snapshot); }

</script>
<svelte:head><title>{t('workshop.title')} · Luxmc</title></svelte:head>
<div class="mx-auto w-full max-w-6xl space-y-6 pb-10">
    <a href="/settings" class={button({variant:'ghost',size:'sm'})}><ArrowLeft class="h-4 w-4" />{t('nav.settings')}</a>
    <header class="workshop-hero rounded-3xl border border-border bg-bg-elevated p-7"><p class="page-eyebrow">Luxmc / {t('workshop.eyebrow')}</p><h1 class="mt-3 text-3xl font-semibold tracking-tight text-fg">{t('workshop.title')}</h1><p class="mt-3 max-w-2xl text-sm leading-relaxed text-fg-muted">{t('workshop.subtitle')}</p></header>
    <PcRecommendations />
    {#if error}<p role="alert" class="rounded-2xl border border-danger/25 bg-danger/10 p-4 text-sm text-danger">{error}</p>{/if}
    {#if success}<p role="status" class="rounded-2xl border border-success/25 bg-success/10 p-4 text-sm text-success">{success}</p>{/if}
    <section class="workshop-card space-y-5"><div class="flex items-start justify-between gap-4"><div><h2 class="flex items-center gap-3 text-lg font-semibold"><Activity class="h-5 w-5 text-brand-400" />{t('workshop.performance')}</h2><p class="mt-2 text-xs leading-relaxed text-fg-muted">{t('workshop.measurement')}</p></div><button disabled={busy} type="button" class={button({variant:'ghost',size:'icon'})} aria-label={t('workshop.refresh')} onclick={() => action(refresh)}><RefreshCw class="h-4 w-4" /></button></div>
        <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">{#each summaries as summary (summary.profile.id)}<a href={`/instances/${summary.profile.id}`} class="rounded-2xl border border-border bg-bg-subtle p-4 transition-colors hover:border-brand-400/40"><h3 class="truncate text-sm font-semibold">{summary.profile.name}</h3><p class="mt-1 text-[11px] text-fg-muted">{summary.profile.mcVersion} · {summary.profile.loader}</p><div class="mt-5 grid grid-cols-2 gap-3"><div><p class="text-[10px] text-fg-muted">{t('workshop.preparation')}</p><p class="mt-1 text-xl font-semibold tabular-nums">{summary.average === null ? '—' : `${summary.average.toFixed(1)} s`}</p></div><div><p class="text-[10px] text-fg-muted">{t('workshop.peakRam')}</p><p class="mt-1 text-xl font-semibold tabular-nums">{summary.peak === null ? '—' : `${(summary.peak/1024).toFixed(1)} GB`}</p></div></div><p class="mt-4 text-[10px] text-fg-muted">{summary.launches} {t('workshop.records')}</p></a>{/each}</div>
        {#if history.length}<details class="text-xs"><summary class="cursor-pointer text-brand-400">{t('workshop.sessions')}</summary><div class="mt-3 max-h-60 overflow-auto"><table class="w-full text-left"><thead><tr class="border-b border-border text-fg-muted"><th class="p-2">{t('workshop.instance')}</th><th class="p-2">{t('workshop.date')}</th><th class="p-2">{t('workshop.measure')}</th></tr></thead><tbody>{#each history.slice(0,100) as entry}<tr class="border-b border-border/50"><td class="p-2">{profiles.list.find(p => p.id===entry.profileId)?.name || '—'}</td><td class="p-2">{new Date(entry.timestamp).toLocaleString(settings.value.language)}</td><td class="p-2 tabular-nums">{entry.metrics.kind!=='session' ? `${entry.metrics.seconds?.toFixed(1)} s · ${entry.metrics.phases?.map(phase=>phase.name+': '+phase.seconds.toFixed(1)+'s').join(' · ') || entry.metrics.kind}` : `${entry.metrics.peakRamMb} MB · ${Math.round((entry.metrics.durationSeconds || 0)/60)} min`}</td></tr>{/each}</tbody></table></div></details>{/if}
    </section>
    <div class="grid gap-5 lg:grid-cols-2"><section class="workshop-card"><h2 class="flex items-center gap-3 text-lg font-semibold"><Palette class="h-5 w-5 text-brand-400" />{t('workshop.themes')}</h2><p class="my-4 text-sm leading-relaxed text-fg-muted">{t('workshop.themeHelp')}</p><div class="flex flex-wrap gap-3"><button disabled={busy} type="button" class={button({variant:'secondary'})} onclick={() => action(exportTheme)}><Upload class="h-4 w-4" />{t('workshop.exportTheme')}</button><button disabled={busy} type="button" class={button({variant:'primary'})} onclick={() => action(importTheme)}><Download class="h-4 w-4" />{t('workshop.importTheme')}</button></div></section>
    <section class="workshop-card"><h2 class="flex items-center gap-3 text-lg font-semibold"><HardDrive class="h-5 w-5 text-brand-400" />{t('workshop.backups')}</h2><p class="my-4 text-sm leading-relaxed text-fg-muted">{t('workshop.backupHelp')}</p><label class="flex items-center gap-3 text-sm"><input type="checkbox" checked={settings.value.autoBackup === true} disabled={busy} onchange={event => action(() => updateBackup(event.currentTarget.checked))} />{t('workshop.autoBackup')}</label><label class="mt-4 flex items-center justify-between gap-3 text-xs text-fg-muted">{t('workshop.backupLimit')}<input type="number" min="1" max="100" value={settings.value.backupLimitGb || 5} disabled={busy} class="w-20 rounded-xl border border-border bg-bg-subtle p-2 text-fg" onchange={event => action(() => updateBackup(settings.value.autoBackup===true,Number(event.currentTarget.value)))} /></label></section></div>
    <section class="workshop-card space-y-5"><h2 class="flex items-center gap-3 text-lg font-semibold"><ShieldCheck class="h-5 w-5 text-brand-400" />{t('workshop.health')}</h2><div class="flex flex-wrap items-center gap-3"><select bind:value={selected} disabled={busy} onchange={() => {readiness=null;report=null;backups=[];pendingRestore=null;}} aria-label={t('workshop.instance')} class="min-w-48 rounded-xl border border-border bg-bg-subtle px-3 py-3 text-sm">{#each profiles.list as profile}<option value={profile.id}>{profile.name}</option>{/each}</select><button type="button" disabled={busy || !selected} class={button({variant:'primary'})} onclick={() => action(check)}>{t('workshop.validate')}</button><button type="button" disabled={busy || !selected} class={button({variant:'secondary'})} onclick={() => action(diagnose)}>{t('workshop.support')}</button><button type="button" disabled={busy || !selected} class={button({variant:'secondary'})} onclick={() => action(loadBackups)}>{t('workshop.showBackups')}</button></div>
    {#if busy}<p role="status" class="text-sm text-fg-muted">{t('common.loading')}</p>{/if}
    {#if readiness}<div class="rounded-2xl border border-border bg-bg-subtle p-4"><p class="font-semibold">{readiness.ready ? t('workshop.ready') : t('workshop.needsRepair')}</p>{#each [...readiness.blockers,...readiness.warnings] as message}<p class="mt-2 text-xs leading-relaxed text-fg-muted">{message}</p>{/each}{#if readiness.repairable}<button type="button" disabled={busy} class={button({variant:'secondary',class:'mt-4'})} onclick={() => action(async () => { await doctorRepairAll(selected); await check(); })}>{t('workshop.repair')}</button>{/if}</div>{/if}
    {#if report}<div class="space-y-3"><p class="text-xs leading-relaxed text-fg-muted">{t('workshop.privacy')}</p><p class="text-sm font-semibold">{report.diagnosis.title}</p><p class="text-sm text-fg-muted">{report.diagnosis.solution}</p><pre class="max-h-60 overflow-auto rounded-2xl border border-border bg-bg-subtle p-4 text-[11px]">{JSON.stringify(report,null,2)}</pre><button type="button" class={button({variant:'secondary'})} disabled={busy} onclick={() => action(exportReport)}><Download class="h-4 w-4" />{t('workshop.exportReport')}</button></div>{/if}
    {#each backups as backup}<div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl border border-border p-4"><div><p class="text-sm font-semibold">{backup.worldName}</p><p class="mt-1 text-xs text-fg-muted">{backup.createdAt} · {(backup.sizeBytes/1048576).toFixed(1)} MB</p></div><button type="button" disabled={busy} class={button({variant:'secondary',size:'sm'})} onclick={() => pendingRestore=backup}>{t('workshop.restore')}</button></div>{/each}
    {#if pendingRestore}<div role="alert" class="rounded-2xl border border-warning/30 bg-warning/10 p-4"><p class="text-sm">{t('workshop.restoreConfirm',{name:pendingRestore.worldName})}</p><div class="mt-3 flex gap-3"><button type="button" disabled={busy} class={button({variant:'primary',size:'sm'})} onclick={() => action(async () => { await restoreWorld(selected,pendingRestore!.fileName); pendingRestore=null; success=t('workshop.restored'); })}>{t('workshop.restore')}</button><button type="button" disabled={busy} class={button({variant:'ghost',size:'sm'})} onclick={() => pendingRestore=null}>{t('common.cancel')}</button></div></div>{/if}
    </section>
</div>
<div class="mx-auto mt-6 w-full max-w-6xl pb-10"><DependencyMap profileId={selected} /></div>
<style>
    .workshop-card { border: 1px solid rgb(var(--border)); border-radius: 1.5rem; background: rgb(var(--bg-elevated)); padding: 1.5rem; }
    .workshop-hero { background-image: radial-gradient(ellipse at 90% 0%, rgb(var(--brand-500) / .14), transparent 65%); }
</style>
