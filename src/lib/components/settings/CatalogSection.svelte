<script lang="ts">
    import { settings, type AppSettings } from '$lib/stores/settings.svelte';
    import { schedulePersist } from '$lib/stores/persistence.svelte';
    import { versionsList } from '$lib/api/launch';
    import { settingsSetConcurrentDownloads } from '$lib/api/settings';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { RefreshCw } from 'lucide-svelte';
    let busy = $state(false);
    let message = $state('');
    let error = $state('');
    function update(value: Partial<AppSettings>) { settings.patch(value); schedulePersist(); }
    async function refresh() {
        if (busy) return;
        busy = true; error = '';
        try { const manifest = await versionsList(); message = `${t('extendedSettings.latestRelease')}: ${manifest.latestRelease} · ${t('extendedSettings.snapshot')}: ${manifest.latestSnapshot}`; }
        catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
    async function downloads(value: number) {
        try { await settingsSetConcurrentDownloads(value); update({concurrentDownloads: value}); error = ''; }
        catch (cause) { error = String(cause); }
    }
</script>

<section class="surface-glass space-y-6 p-6">
    <header><h2 class="text-2xl font-bold">{t('extendedSettings.catalog')}</h2><p class="mt-2 text-sm leading-6 text-fg-muted">{t('extendedSettings.catalogHelp')}</p></header>
    <div class="grid gap-5 sm:grid-cols-2">
        <label class="space-y-2 text-sm"><span class="block">{t('settings.concurrentDownloads')}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.concurrentDownloads ?? 24} onchange={event => void downloads(Number(event.currentTarget.value))}>{#each [2,4,8,16,24] as count}<option value={count}>{count}</option>{/each}</select><small class="block leading-5 text-fg-muted">{t('settings.concurrentDownloadsDesc')}</small></label>
        <label class="space-y-2 text-sm"><span class="block">{t('settings.catalog.pageSize')}</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.catalogPageSize || 36} onchange={event => update({catalogPageSize: Number(event.currentTarget.value) as 24|36|48})}>{#each [24,36,48] as count}<option value={count}>{count}</option>{/each}</select></label>
        <label class="flex items-center justify-between gap-3 rounded-xl border border-border p-4 text-sm"><span>{t('settings.catalog.compact')}</span><input type="checkbox" class="h-5 w-5 accent-brand-500" checked={settings.value.catalogCompactCards === true} onchange={event => update({catalogCompactCards:event.currentTarget.checked})} /></label>
        <label class="flex items-center justify-between gap-3 rounded-xl border border-border p-4 text-sm"><span>{t('extendedSettings.preload')}</span><input type="checkbox" class="h-5 w-5 accent-brand-500" checked={settings.value.preloadContentIcons !== false} onchange={event => update({preloadContentIcons:event.currentTarget.checked})} /></label>
    </div>
    <div class="space-y-3 rounded-2xl border border-border bg-bg-subtle p-5"><h3 class="font-semibold">{t('extendedSettings.minecraftCatalog')}</h3><p class="text-xs leading-6 text-fg-muted">{t('extendedSettings.versionsHelp')}</p><button type="button" class={button({variant:'secondary'})} disabled={busy} onclick={() => void refresh()}><RefreshCw class="h-4 w-4 {busy ? 'animate-spin' : ''}" />{t('extendedSettings.refreshCatalog')}</button>{#if message}<p role="status" class="text-xs text-fg-muted">{message}</p>{/if}</div>
    {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
</section>
