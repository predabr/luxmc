<script lang="ts">
    import { onMount } from 'svelte';
    import { Globe, ExternalLink, Copy, Link2, LoaderCircle, Square } from 'lucide-svelte';
    import { api } from '$lib/api/client';
    import { serverManage } from '$lib/api/localServers';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    let { id, running, pro, port }: { id: string; running: boolean; pro: boolean; port?: number | null } = $props();
    type Status = { linked: boolean; claimUrl?: string | null; address: string; agentRunning: boolean };
    let status = $state<Status>({ linked: false, address: '', agentRunning: false });
    let busy = $state(false); let error = $state(''); let copied = $state(false); let disposed = false;
    async function load(operation = 'status') { const next = await serverManage<Status>({ action: 'public_tunnel', id, operation }); if (!disposed) status = next; }
    onMount(() => { let timer: ReturnType<typeof setTimeout>; async function poll() { if (!busy) { try { await load(); } catch (cause) { if (!disposed) error = String(cause); } } if (!disposed) timer = setTimeout(poll, 5000); } void poll(); return () => { disposed = true; clearTimeout(timer); }; });
    async function act(operation: string) { if (busy) return; busy = true; error = ''; try { await load(operation); if (operation === 'claim' && status.claimUrl) await external(status.claimUrl); } catch (cause) { if (!disposed) error = String(cause); } finally { if (!disposed) busy = false; } }
    async function external(url: string) { if (window.electronAPI) await api.invoke('open_url', { url }); else { const { openUrl } = await import('@tauri-apps/plugin-opener'); await openUrl(url); } }
    async function copy() { try { await navigator.clipboard.writeText(status.address); copied = true; } catch (cause) { error = String(cause); } }
</script>

<section class="space-y-4 rounded-2xl border border-brand-400/25 bg-bg-elevated p-5" data-public-server>
    <div class="flex flex-wrap items-center justify-between gap-3"><h3 class="flex items-center gap-3 font-semibold"><Globe class="h-5 w-5 text-brand-400" />{t('serverPanel.publicAddress')}</h3><span class="rounded-full border border-border px-3 py-1 text-xs text-fg-muted">playit.gg</span></div>
    <p class="text-sm leading-6 text-fg-muted">{t('serverPanel.playitHelp')}</p>
    {#if port}<p class="rounded-xl border border-border bg-bg-subtle p-3 text-xs leading-6 text-fg-muted">{t('serverPanel.playitTarget')} <strong class="font-mono text-fg">127.0.0.1:{port}</strong></p>{/if}
    {#if error}<p role="alert" class="break-words rounded-xl border border-danger/25 bg-danger/5 p-3 text-sm text-danger">{error}</p>{/if}
    {#if status.address}<div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-bg-subtle p-4"><div><p class="break-all font-mono text-sm font-semibold">{status.address}</p><p class="mt-2 text-xs text-fg-muted">{t(status.agentRunning && running ? 'serverPanel.publicAgentRunning' : 'serverPanel.publicAgentStopped')}</p></div><button type="button" class={button({ variant: 'secondary' })} onclick={copy}><Copy class="h-4 w-4" />{t(copied ? 'localServer.copied' : 'serverPanel.copyAddress')}</button></div>{/if}
    <div class="flex flex-wrap gap-3">
        {#if !status.linked}<button type="button" class={button({ variant: 'primary' })} disabled={busy || !pro} onclick={() => act('claim')}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Link2 class="h-4 w-4" />{/if}{t('serverPanel.linkPlayit')}</button>{/if}
        {#if status.claimUrl}<button type="button" class={button({ variant: 'secondary' })} onclick={() => external(status.claimUrl!)}><ExternalLink class="h-4 w-4" />{t('serverPanel.finishPlayit')}</button>{/if}
        {#if status.linked && !status.address}<button type="button" class={button({ variant: 'secondary' })} onclick={() => external('https://playit.gg/account/setup/new-tunnel')}><ExternalLink class="h-4 w-4" />{t('serverPanel.createPlayitTunnel')}</button>{/if}
        <button type="button" class={button({ variant: status.linked ? 'primary' : 'secondary' })} disabled={busy || !running || !pro || (!status.linked && !status.claimUrl)} onclick={() => act('connect')}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Globe class="h-4 w-4" />{/if}{t('serverPanel.connectPlayit')}</button>
        {#if status.agentRunning}<button type="button" class={button({ variant: 'secondary' })} disabled={busy} onclick={() => act('stop')}><Square class="h-4 w-4" />{t('serverPanel.stopSharing')}</button>{/if}
        <button type="button" class={button({ variant: 'ghost' })} onclick={() => external('https://playit.gg/account/tunnels')}><ExternalLink class="h-4 w-4" />{t('serverPanel.playitDashboard')}</button>
    </div>
</section>
