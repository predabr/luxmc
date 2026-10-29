<script lang="ts">
    import { onMount } from 'svelte';
    import { save } from '@tauri-apps/plugin-dialog';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    import { serverManage, type ServerBackup } from '$lib/api/localServers';
    let { id, running }: { id: string; running: boolean } = $props();
    let items = $state<ServerBackup[]>([]); let busy = $state(false); let error = $state(''); let confirm = $state(''); let ready = $state(false); let disposed = false;
    async function load() { const next = await serverManage<ServerBackup[]>({ action: 'backups', id }); if (!disposed) { items = next; ready = true; } }
    async function act(action: string, backup?: string) { if (busy) return; busy = true; error = ''; try { await serverManage({ action, id, backup }); if (!disposed) { confirm = ''; await load(); } } catch (cause) { if (!disposed) error = String(cause); } finally { if (!disposed) busy = false; } }
    async function exportCopy(backup: string) { const path = await save({ defaultPath: `Luxmc-backup-${backup}.zip`, filters: [{ name: 'ZIP', extensions: ['zip'] }] }); if (!path) return; busy = true; error = ''; try { await serverManage({ action: 'export_backup', id, backup, path }); } catch (cause) { error = String(cause); } finally { busy = false; } }
    onMount(() => { void load().catch(cause => { if (!disposed) error = String(cause); }); return () => { disposed = true; }; });
</script>

<section class="space-y-5 p-5 sm:p-6">
    <div class="flex flex-wrap items-start justify-between gap-4"><div class="max-w-xl"><h3 class="font-semibold">{t('serverPanel.backups')}</h3><p class="mt-2 text-sm leading-6 text-fg-muted">{t('serverPanel.backupHelp')}</p></div><button type="button" class={button({ variant: 'primary' })} disabled={busy || running || items.length >= 10} onclick={() => act('backup')}>{busy ? t('localServer.loading') : t('serverPanel.createBackup')}</button></div>
    {#if error}<p role="alert" class="break-words rounded-xl border border-danger/30 p-3 text-sm text-danger">{error}</p>{/if}
    {#if running}<p class="rounded-xl bg-warning/10 p-4 text-sm text-warning">{t('serverPanel.stopForBackup')}</p>{/if}
    <p class="text-xs text-fg-muted">{items.length} / 10 · {t('serverPanel.noAutoDelete')}</p>
    {#if ready && !items.length}<p class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-fg-muted">{t('serverPanel.noBackups')}</p>{/if}
    {#each items as item (item.id)}<article class="space-y-3 rounded-2xl border border-border bg-bg-subtle p-4"><div class="flex flex-wrap items-center justify-between gap-4"><div><h4 class="text-sm font-semibold">{new Date(item.createdAt).toLocaleString()}</h4><p class="mt-2 text-xs text-fg-muted">{(item.bytes / 1048576).toFixed(1)} MiB · {item.files} {t('serverPanel.files')}</p></div><div class="flex flex-wrap gap-2"><button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={busy} onclick={() => exportCopy(item.id)}>{t('serverPanel.export')}</button><button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={busy || running} onclick={() => confirm = `restore:${item.id}`}>{t('serverPanel.restore')}</button><button type="button" class={button({ variant: 'ghost', size: 'sm' })} disabled={busy || items.length <= 1} onclick={() => confirm = `remove:${item.id}`}>{t('serverPanel.remove')}</button></div></div>{#if confirm.endsWith(item.id)}<div class="space-y-3 border-t border-border pt-3"><p class="text-sm text-warning">{t(confirm.startsWith('restore:') ? 'serverPanel.restoreConfirm' : 'serverPanel.removeConfirm')}</p><div class="flex gap-2"><button type="button" class={button({ variant: 'primary', size: 'sm' })} disabled={busy} onclick={() => act(confirm.startsWith('restore:') ? 'restore_backup' : 'remove_backup', item.id)}>{t('serverPanel.confirm')}</button><button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={() => confirm = ''}>{t('localServer.cancel')}</button></div></div>{/if}</article>{/each}
</section>
