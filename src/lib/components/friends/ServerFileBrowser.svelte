<script lang="ts">
    import { onMount } from 'svelte';
    import { Folder, File, ChevronRight, ArrowUp, FolderOpen, RefreshCw } from 'lucide-svelte';
    import { serverManage, openLocalServerFolder } from '$lib/api/localServers';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    let { id, initialPath = '' }: { id: string; initialPath?: string } = $props();
    type Entry = { name: string; directory: boolean; bytes: number };
    let path = $state(''); let entries = $state<Entry[]>([]); let busy = $state(false); let error = $state(''); let disposed = false;
    async function load(next = path) {
        if (busy) return; busy = true; error = '';
        try { const result = await serverManage<{ path: string; entries: Entry[] } | null>({ action: 'browse_files', id, path: next }); if (!disposed) { entries = result?.entries || []; path = result?.path ?? next; } }
        catch (cause) { if (!disposed) error = String(cause).replace(/^(?:(?:Error|invalid state|invalid input):\s*)+/i, ''); }
        finally { if (!disposed) busy = false; }
    }
    const size = (bytes: number) => bytes >= 1048576 ? `${(bytes / 1048576).toFixed(1)} MiB` : `${(bytes / 1024).toFixed(1)} KiB`;
    onMount(() => { void load(initialPath); return () => { disposed = true; }; });
</script>

<section class="overflow-hidden rounded-xl border border-border bg-bg-subtle">
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3">
        <div class="flex min-w-0 items-center gap-2 text-sm"><FolderOpen class="h-4 w-4 shrink-0 text-brand-400" /><span class="truncate font-mono">/{path}</span></div>
        <div class="flex gap-2"><button type="button" class={button({ variant: 'ghost', size: 'sm' })} disabled={busy || !path} onclick={() => load(path.split('/').slice(0, -1).join('/'))}><ArrowUp class="h-4 w-4" />{t('hosting.parentFolder')}</button><button type="button" class={button({ variant: 'ghost', size: 'icon' })} disabled={busy} aria-label={t('localServer.retry')} onclick={() => load()}><RefreshCw class="h-4 w-4 {busy ? 'animate-spin' : ''}" /></button></div>
    </div>
    {#if error}<p role="alert" class="px-4 py-3 text-sm text-danger">{error}</p>{/if}
    {#if !busy && !entries.length && !error}<p class="px-4 py-8 text-center text-sm text-fg-muted">{t('hosting.emptyFolder')}</p>{/if}
    <div class="divide-y divide-border">{#each entries as item (item.name)}
        {#if item.directory}<button type="button" class="flex w-full items-center gap-3 px-4 py-3 text-left text-sm transition-colors hover:bg-fg/5 focus-visible:outline focus-visible:outline-brand-400" disabled={busy} onclick={() => load(path ? `${path}/${item.name}` : item.name)}><Folder class="h-4 w-4 shrink-0 text-brand-400" /><span class="min-w-0 flex-1 truncate">{item.name}</span><span class="text-xs text-fg-muted">{t('hosting.folder')}</span><ChevronRight class="h-4 w-4 text-fg-muted" /></button>
        {:else}<div class="flex items-center gap-3 px-4 py-3 text-sm"><File class="h-4 w-4 shrink-0 text-fg-muted" /><span class="min-w-0 flex-1 break-all">{item.name}</span><span class="shrink-0 text-xs tabular-nums text-fg-muted">{size(item.bytes)}</span></div>{/if}
    {/each}</div>
    <div class="border-t border-border p-3"><button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={() => openLocalServerFolder(id)}><FolderOpen class="h-4 w-4" />{t('localServer.folder')}</button></div>
</section>
