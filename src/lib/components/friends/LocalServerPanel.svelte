<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { Server, Play, Square, Terminal, FolderOpen, Share2, Plus, ArrowLeft, Pencil, Send, Copy, LoaderCircle, Save, Users, Settings2, Package, Archive, LayoutDashboard, X, Globe, Wifi, Trash2 } from 'lucide-svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    import { localServerList, createLocalServer, startLocalServer, stopLocalServer, sendServerCommand, openLocalServerFolder, updateLocalServer, shareLocalServer, shareExistingServer, serverManage, type LocalServerConfig, type LocalServerOverview, type ServerConsoleLine } from '$lib/api/localServers';
    import { type TunnelStatus } from '$lib/api/tunnel';
    import { versionsList } from '$lib/api/launch';
    import { membershipState } from '$lib/stores/membership.svelte';
    import { toast } from '$lib/stores/toasts.svelte';
    import ServerCreator from './ServerCreator.svelte';
    import ServerConsolePanel from './ServerConsolePanel.svelte';
    import { serverConnectionDiagnostic, serverDeletionPreview } from '$lib/api/localServers';
    import ServerSettingsPanel from './ServerSettingsPanel.svelte';
    import ServerPlayersPanel from './ServerPlayersPanel.svelte';
    import ServerContentPanel from './ServerContentPanel.svelte';

    let pendingSetup = $state<{source:string;project:string;version:string}|null>(null);
    let sending = $state(false);
    let deleteOpen = $state(false); let deleteName = $state(''); let deleteDialog = $state<HTMLDialogElement>(); let creatorOpen = $state(false); let deletion = $state<{files:number;bytes:number}|null>(null); let checking = $state(false); let verifiedAddress = $state<string|null>(null);
    let overview = $state<LocalServerOverview>({ servers: [], active: null, warnings: [] });
    let tunnel = $state<TunnelStatus | null>(null);
    let lines = $state<ServerConsoleLine[]>([]);
    let versions = $state<{ id: string; versionType: string }[]>([]);
    let selected = $state(''); let tab = $state('overview');
    let name = $state(''); let version = $state(''); let ram = $state(2048); let players = $state(20); let eula = $state(false); let snapshots = $state(false); let editing = $state('');
    let formDialog = $state<HTMLDialogElement>(); let formOpen = $state(false); let busy = $state(false);  let sharing = $state(false); let ready = $state(false); let error = $state(''); let versionError = $state('');  let externalPort = $state(25565); let external = $state(false);
     let cursor = 0; let runtimeId = ''; let disposed = false; let operation = 0; let contentRevision = $state(0);
    const config = $derived(overview.servers.find(server => server.id === selected));
    const serverState = $derived(overview.active?.id === selected ? overview.active : null);
    const connectionKey = $derived(`${selected}:${serverState?.generation || ''}:${serverState?.phase || ''}:${serverState?.port || ''}`);
    const running = $derived(Boolean(serverState && !['stopped', 'failed'].includes(serverState.phase)));
    const anotherRunning = $derived(Boolean(overview.active && overview.active.id !== selected && !['stopped', 'failed'].includes(overview.active.phase)));
    const pro = $derived(Boolean(membershipState.ready && membershipState.current?.plan === 'pro' && (membershipState.current.expiresAt ?? 0) > Date.now() / 1000));
    const choices = $derived(versions.filter(item => snapshots || item.versionType === 'release'));
    const shared = $derived(tunnel?.mode === 'host' && tunnel.hostedServer?.localServerId.startsWith(`${selected}:`) ? tunnel : null);
    const tabs = [{ id: 'overview', icon: LayoutDashboard }, { id: 'contents', icon: Package }, { id: 'players', icon: Users }, { id: 'console', icon: Terminal }, { id: 'settings', icon: Settings2 }];

    $effect(() => { if (!formDialog) return; if (formOpen && !formDialog.open) formDialog.showModal(); else if (!formOpen && formDialog.open) formDialog.close(); });
    $effect(() => { if (!deleteDialog) return; if (deleteOpen && !deleteDialog.open) deleteDialog.showModal(); else if (!deleteOpen && deleteDialog.open) deleteDialog.close(); });
    $effect(() => { const id = creatorOpen ? '' : selected; const revision = contentRevision; let cancelled = false; pendingSetup = null; if (id && revision >= 0) void serverManage<typeof pendingSetup>({action:'catalog_pending',id}).then(value => { if (!cancelled && !disposed) pendingSetup = value; }).catch(() => {}); return () => { cancelled = true; }; });
    async function resumeSetup() { if (!pendingSetup || !config) return; const request = {...pendingSetup}; const id = config.id; await act(async () => { await serverManage({action:'install_catalog_pack',id,...request}); pendingSetup = null; }); }
    async function removeServer() { if (!config || deleteName !== config.name) return; const id = config.id; await act(async () => { if (shared) { await serverManage({ action: 'stop_sharing' }); tunnel = null; } await serverManage({ action: 'delete_server', id, name: deleteName, deleteBackups: true }); selected = ''; deleteOpen = false; }); }
    function message(cause: unknown) { return String(cause).replace(/^(?:(?:Error|invalid state|invalid input|serialization error):\s*)+/i, ''); }
    async function refresh() {
        const revision = operation;
        const next = await localServerList(cursor);
        if (disposed || revision !== operation) return;
        const key = next.active ? `${next.active.id}:${next.active.generation}` : '';
        if (runtimeId !== key || (next.active && next.active.cursor < cursor)) { const previousCursor = cursor; lines = []; cursor = 0; if (previousCursor && next.active) { const complete = await localServerList(0); if (disposed || revision !== operation) return; next.active = complete.active; } }
        runtimeId = next.active ? `${next.active.id}:${next.active.generation}` : '';
        if (next.active) { if (cursor === 0 && next.active.cursor && !next.active.console.length) { const complete = await localServerList(0); if (disposed || revision !== operation) return; next.active = complete.active; } if (next.active) { lines = [...lines, ...next.active.console.filter(line => line.sequence > cursor)].slice(-500); cursor = next.active.cursor; } }
        overview = next;
        if (!selected || !next.servers.some(server => server.id === selected)) selected = next.active?.id || next.servers[0]?.id || '';
        ready = true; await tick();
    }
    onMount(() => {
        let timer: ReturnType<typeof setTimeout>; let roomTimer: ReturnType<typeof setTimeout>;
        async function poll() { if (!busy || overview.active?.phase === 'stopping') { try { await refresh(); } catch (cause) { if (!disposed) { error = message(cause); ready = true; } } } if (!disposed) timer = setTimeout(poll, document.hidden ? 6000 : 1500); }
        async function room() { if (!sharing && !busy) { try { const next = await serverManage<TunnelStatus | null>({ action: 'sharing_status' }); if (!disposed) tunnel = next; } catch {} } if (!disposed) roomTimer = setTimeout(room, document.hidden ? 6000 : 2500); }
        void poll(); void room(); void loadVersions();

        return () => { disposed = true; clearTimeout(timer); clearTimeout(roomTimer); };
    });
    async function loadVersions() { versionError = ''; try { const result = await versionsList(); if (!disposed) { versions = result.versions; if (!version) version = result.latestRelease; } } catch (cause) { if (!disposed) versionError = message(cause); } }
    async function act(action: () => Promise<unknown>) { if (busy) return; busy = true; error = ''; operation++; try { await action(); if (!disposed) await refresh(); } catch (cause) { if (!disposed) error = message(cause); } finally { if (!disposed) { busy = false; contentRevision++; } } }
    function edit(server?: LocalServerConfig) { if (!server) { creatorOpen = true; return; } editing = server.id; name = server.name; version = server.mcVersion; ram = server.ramMb; players = server.maxPlayers; eula = server.eulaAccepted; formOpen = true; }
    async function save() { await act(async () => { const saved = await updateLocalServer(editing,name,ram,players); selected = saved.id; formOpen = false; toast(t('localServer.saved'),'success'); }); }
    async function start() { if (!config) return; const id = config.id; await act(async () => { lines = []; cursor = 0; overview.active = await startLocalServer(id); }); }
    async function stop() { if (!serverState) return; const id = serverState.id; serverState.phase = 'stopping'; await act(() => stopLocalServer(id)); }
    async function share() { if (busy || sharing || !serverState) return; const id = serverState.id; sharing = true; error = ''; try { const next = await shareLocalServer(id); if (!disposed) { tunnel = next; tab = 'overview'; } } catch (cause) { if (!disposed) error = message(cause); } finally { if (!disposed) sharing = false; } }
    async function copy(value: string) { try { await navigator.clipboard.writeText(value); toast(t('localServer.copied'), 'success'); } catch (cause) { error = message(cause); } }
    async function send(text: string): Promise<boolean> { if (sending || !serverState || !['running','unreachable'].includes(serverState.phase)) return false; const id = serverState.id; sending = true; error = ''; try { await sendServerCommand(id,text); if (!disposed) await refresh(); return true; } catch (cause) { if (!disposed) error = message(cause); return false; } finally { sending = false; } }
    $effect(() => { const [id, , phase, port] = connectionKey.split(':'); verifiedAddress = null; let cancelled = false; if (id && port && ['running','unreachable'].includes(phase || '')) void serverConnectionDiagnostic(id).then(result => { if (!cancelled && !disposed) verifiedAddress = result.address; }).catch(() => {}); return () => { cancelled = true; }; });
    async function verifyConnection(copyAddress = false) { if (checking || !config) return; const id = config.id; checking = true; error = ''; try { const result = await serverConnectionDiagnostic(id); if (disposed || selected !== id) return; verifiedAddress = result.address; if (result.address && copyAddress) await copy(result.address); else if (!result.minecraftResponding) error = t('consoleTools.connectionFailed'); } catch (cause) { if (!disposed) error = message(cause); } finally { checking = false; } }
    async function confirmDelete() { if (!config) return; deleteName = ''; deletion = null; deleteOpen = true; const id = config.id; try { const result = await serverDeletionPreview(id); if (deleteOpen && selected === id) deletion = result; } catch (cause) { error = message(cause); } }

</script>

{#snippet externalPanel()}<section class="m-5 space-y-3 rounded-2xl border border-warning/25 p-4"><h3 class="font-semibold">{t('localServer.externalTitle')}</h3><p class="text-sm leading-6 text-fg-muted">{t('serverPanel.externalHelp')}</p><form class="flex flex-wrap gap-3" onsubmit={event => { event.preventDefault(); void act(async () => { tunnel = await shareExistingServer(externalPort); }); }}><label><span class="sr-only">{t('localServer.port')}</span><input class="server-field" type="number" min="1" max="65535" required aria-label={t('localServer.port')} bind:value={externalPort} /></label><button type="submit" class={button({ variant: 'secondary' })} disabled={busy || !pro}>{t('localServer.share')}</button></form>{#if tunnel?.hostedServer?.localServerId === `external:${externalPort}`}<div class="space-y-3 border-t border-border pt-3"><p class="text-sm leading-6 text-fg-muted">{t('serverPanel.integrationHelp')}</p><div class="flex flex-wrap gap-3"><button type="button" class={button({ variant: 'primary', size: 'sm' })} onclick={() => copy(tunnel!.invitation || tunnel!.roomCode || '')}>{t('serverPanel.copyCode')}</button><button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={busy} onclick={() => act(async () => { await serverManage({ action: 'stop_sharing' }); tunnel = null; })}>{t('serverPanel.stopSharing')}</button></div></div>{/if}</section>{/snippet}

<svelte:head><title>{t('localServer.title')} · Luxmc</title></svelte:head>
<div class="server-workspace mx-auto w-full max-w-screen-2xl space-y-4" data-local-server>
    <header class="flex flex-wrap items-center justify-between gap-4"><div class="flex items-center gap-3"><a class={button({ variant: 'ghost', size: 'icon' })} href="/hosting" aria-label={t('localServer.back')}><ArrowLeft class="h-4 w-4" /></a><div><p class="text-xs font-semibold uppercase tracking-widest text-brand-400">Luxmc Pro</p><h1 class="mt-1 text-2xl font-semibold tracking-tight">{t('hosting.title')}</h1></div></div><div class="flex flex-wrap gap-2"><a class={button({ variant: 'secondary' })} href="/hosting"><Wifi class="h-4 w-4" />{t('serverPanel.lan')}</a><button type="button" class={button({ variant: 'primary' })} disabled={busy || !pro} onclick={() => edit()}><Plus class="h-4 w-4" />{t('localServer.create')}</button></div></header>
    <p class="text-xs leading-5 text-fg-muted">{t('hosting.localMachine')}</p>

    {#if busy}<p role="status" class="flex items-center gap-3 rounded-xl border border-brand-400/25 bg-brand-500/5 px-4 py-3 text-sm"><LoaderCircle class="h-4 w-4 animate-spin" />{t(serverState?.phase === 'stopping' ? 'localServer.phase.stopping' : 'serverPanel.preparingContents')}</p>{/if}
    {#if !pro}<p class="rounded-xl border border-border bg-bg-elevated px-4 py-3 text-sm leading-6 text-fg-muted">{t('serverPanel.proHelp')}</p>{/if}
    {#if error}<div role="alert" class="flex items-start justify-between gap-3 rounded-xl border border-danger/30 bg-bg-elevated p-3 text-sm text-danger"><p class="min-w-0 break-words">{error}</p><button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={t('localServer.cancel')} onclick={() => error = ''}><X class="h-4 w-4" /></button></div>{/if}
    {#each overview.warnings as warning}<p role="alert" class="rounded-xl border border-warning/30 bg-bg-elevated p-3 text-xs text-warning">{warning}</p>{/each}
    <div class="server-shell overflow-hidden rounded-3xl border border-border bg-bg-elevated">
        <aside class="server-list custom-scrollbar overflow-auto border-border p-3" aria-label={t('localServer.yourServers')}>
            <p class="px-2 pb-3 pt-2 text-xs font-semibold uppercase tracking-wide text-fg-muted">{t('localServer.yourServers')}</p>
            {#if !ready}<p class="p-3 text-sm text-fg-muted">{t('localServer.loading')}</p>{/if}
            <div class="hosting-server-choices custom-scrollbar">            {#each overview.servers as item (item.id)}<button type="button" class="server-item mb-2 w-full rounded-2xl border p-4 text-left {selected === item.id ? 'border-brand-400/40 bg-brand-500/10' : 'border-border bg-bg-subtle'}" aria-pressed={selected === item.id} onclick={() => { selected = item.id; error = ''; }}><span class="flex items-center gap-2 text-sm font-semibold">{#if overview.icons?.[item.id]}<img src={overview.icons[item.id]} alt="" class="h-8 w-8 shrink-0 rounded-lg object-contain" />{:else}<Server class="h-4 w-4 shrink-0 text-brand-400" />{/if}<span class="truncate">{item.name}</span>{#if overview.active?.id === item.id && !['stopped', 'failed'].includes(overview.active.phase)}<span class="ml-auto h-2 w-2 shrink-0 rounded-full bg-success"></span>{/if}</span><span class="mt-3 block text-xs text-fg-muted">Minecraft {item.mcVersion}</span><span class="mt-1 block text-xs text-fg-muted">{(item.ramMb / 1024).toFixed(1)} GiB · {item.maxPlayers} {t('serverPanel.slots')}</span></button>{/each}
</div>
            {#if ready && !overview.servers.length}<p class="p-3 text-sm leading-6 text-fg-muted">{t('localServer.empty')}</p>{/if}
            <button type="button" class={button({ variant: 'ghost', size: 'sm', class: 'mt-2 w-full' })} onclick={() => { external = !external; tab = 'overview'; }}>{t('serverPanel.external')}</button>
            <div class="hosting-nav-label">{t("hosting.manage")}</div>
                <nav class="hosting-section-nav" aria-label={t('serverPanel.navigation')}>{#each tabs as item}<button type="button" class="server-tab flex min-w-0 items-center gap-3 px-3 py-3 text-sm font-medium {tab === item.id ? 'border-brand-400 text-brand-400' : 'border-transparent text-fg-muted'}" aria-current={tab === item.id ? 'page' : undefined} onclick={() => tab = item.id}><item.icon class="h-4 w-4 shrink-0" />{t(`serverPanel.${item.id}`)}</button>{/each}</nav>

        </aside>
        <section class="flex min-h-0 min-w-0 flex-col" aria-label={config?.name || t('localServer.title')}>
            {#if config}
                <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border px-5 py-4"><div class="min-w-0"><h2 class="truncate text-lg font-semibold">{config.name}</h2><p role="status" class="mt-1 text-xs text-fg-muted">{serverState ? t(`localServer.phase.${serverState.phase}`) : t('localServer.idle')}</p></div><div class="flex gap-2"><button type="button" class={button({ variant: 'secondary', size: 'sm' })} data-server-open-files aria-label={t('localServer.folder')} onclick={() => act(() => openLocalServerFolder(config!.id))}><FolderOpen class="h-4 w-4" /><span>{t("serverPanel.files")}</span></button><button type="button" class={button({ variant: 'ghost', size: 'icon' })} disabled={busy || running || !pro} aria-label={t('localServer.edit')} onclick={() => edit(config)}><Pencil class="h-4 w-4" /></button><button type="button" aria-label={t("hosting.delete")} class={button({variant:"ghostDanger",size:"sm"})} disabled={busy || running} onclick={confirmDelete}><Trash2 class="h-4 w-4" /><span>{t("hosting.deleteShort")}</span></button>{#if running}<button type="button" class={button({ variant: 'secondary' })} disabled={busy || serverState?.phase === 'stopping'} onclick={stop}><Square class="h-4 w-4" />{t('localServer.stop')}</button>{:else}<button type="button" class={button({ variant: 'primary' })} disabled={busy || anotherRunning || !pro} onclick={start}><Play class="h-4 w-4" />{t('localServer.start')}</button>{/if}</div></div>

                <div class="server-content custom-scrollbar min-h-0 flex-1 overflow-auto" data-server-content>
                    {#key `${selected}:${tab}:${tab === 'contents' ? contentRevision : 0}`}
                    {#if tab === 'overview'}<div class="space-y-5 p-5 sm:p-6">
                        {#if pendingSetup}<div class="rounded-xl border border-warning/30 bg-warning/5 p-4"><p class="text-sm leading-6">{t('creator.pendingHelp')}</p><button type="button" class={button({variant:'primary',size:'sm',class:'mt-3'})} disabled={busy || !pro || running} onclick={resumeSetup}>{t('creator.resume')}</button></div>{/if}
                        {#if serverState?.error}<p role="alert" class="rounded-xl border border-danger/30 p-4 text-sm leading-6 text-danger">{message(serverState.error)}</p>{/if}
                        <div class="grid gap-3 sm:grid-cols-3">{#each [{ label: 'online', value: `${serverState?.playersOnline ?? 0} / ${config.maxPlayers}` }, { label: 'memory', value: `${(config.ramMb / 1024).toFixed(1)} GiB` }, { label: 'version', value: config.mcVersion }] as metric}<div class="rounded-2xl border border-border bg-bg-subtle p-4"><p class="text-xs text-fg-muted">{t(`serverPanel.${metric.label}`)}</p><p class="mt-2 text-lg font-semibold tabular-nums">{metric.value}</p></div>{/each}</div>
                        <section class="hosting-local-entry rounded-2xl border border-brand-400/30 bg-brand-500/5 p-5"><div class="flex items-center gap-3"><Play class="h-5 w-5 text-brand-400" /><h3 class="font-semibold">{t('consoleTools.connection')}</h3></div><p class="mt-3 text-sm leading-7 text-fg-muted">{t('creator.ownerEntry')}</p><div class="mt-4 flex flex-wrap gap-3"><button type="button" class={button({variant:'primary'})} disabled={checking || !verifiedAddress} onclick={() => verifyConnection(true)}><Copy class="h-4 w-4" /><span class="font-mono">{verifiedAddress || t('consoleTools.notVerified')}</span></button><button type="button" class={button({variant:'secondary'})} disabled={checking || !running} onclick={() => verifyConnection()}>{#if checking}<LoaderCircle class="h-4 w-4 animate-spin" />{/if}{t('consoleTools.verify')}</button></div><p class="mt-3 text-xs leading-6 text-fg-muted">{t('consoleTools.addressHelp')}</p></section>
                        <section class="hosting-network" data-shared-server>
                            <div class="flex flex-wrap items-center justify-between gap-4"><div class="flex items-center gap-3"><div class="hosting-network-icon"><Wifi class="h-5 w-5" /></div><div><h3 class="font-semibold">{t('dedicatedP2P.title')}</h3><p class="mt-1 text-xs text-fg-muted">{t('hosting.noExternalAccount')}</p></div></div>
                            {#if shared}<button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={busy} onclick={() => act(async () => { await serverManage({ action: 'stop_sharing' }); tunnel = null; })}>{t('serverPanel.stopSharing')}</button>{:else}<button type="button" class={button({ variant: 'primary' })} disabled={busy || sharing || serverState?.phase !== 'running' || !pro} onclick={share}>{#if sharing}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Share2 class="h-4 w-4" />{/if}{t(sharing ? 'localServer.sharing' : 'localServer.share')}</button>{/if}</div>
                            <p class="mt-4 text-sm leading-6 text-fg-muted">{t('dedicatedP2P.help')}</p><div class="mt-4 border-t border-border pt-4">{#if shared}<div class="flex flex-wrap items-center gap-2"><button type="button" class={button({ variant: 'secondary', size: 'sm' })} onclick={() => copy(shared!.invitation || shared!.roomCode || '')}><Copy class="h-4 w-4" />{t('serverPanel.copyCode')}</button><span class="text-xs text-fg-muted">{shared.members.length} / {shared.maxPlayers} {t('serverPanel.tunnelMembers')}</span><a href="/hosting" class={button({variant:'primary',size:'sm'})}>{t('dedicatedP2P.manageRoom')}</a></div>{:else}<p class="text-sm leading-6 text-fg-muted">{t('dedicatedP2P.steps')}</p>{/if}</div>
                        </section>
                        <section class="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-border px-4 py-3"><div><p class="text-xs font-medium text-fg-muted">{t('serverPanel.lastSave')}</p><p class="mt-1 text-sm">{serverState?.lastSavedAt ? new Date(serverState.lastSavedAt).toLocaleString() : t('serverPanel.unconfirmedSave')}</p></div><button type="button" class={button({ variant: 'secondary', size: 'sm' })} disabled={sending || serverState?.phase !== 'running'} onclick={() => send('save-all flush')}><Save class="h-4 w-4" />{t('localServer.saveWorld')}</button></section>

                        <p class="text-xs leading-6 text-fg-muted">{t('serverPanel.workerHelp')}</p>
                    </div>
                    {:else if tab === 'settings'}<ServerSettingsPanel id={selected} {running} {pro} />
                    {:else if tab === 'players'}<ServerPlayersPanel id={selected} {serverState} {pro} />
                    {:else if tab === 'contents'}<ServerContentPanel id={selected} mcVersion={config.mcVersion} {running} {pro} />
                    {:else}<ServerConsolePanel id={selected} lines={serverState ? lines : []} available={Boolean(serverState && ['running','unreachable'].includes(serverState.phase))} {send} />{/if}
                    {/key}
                    {#if external && tab === 'overview'}{@render externalPanel()}{/if}
                </div>
            {:else}{#if external}{@render externalPanel()}{:else}<div class="flex min-h-64 flex-1 flex-col items-center justify-center gap-4 p-8 text-center"><Server class="h-12 w-12 text-brand-400" /><h2 class="text-xl font-semibold">{t('localServer.empty')}</h2><p class="max-w-md text-sm leading-7 text-fg-muted">{t('serverPanel.emptyHelp')}</p><button type="button" class={button({ variant: 'primary' })} disabled={busy || !pro} onclick={() => edit()}>{t('localServer.create')}</button></div>{/if}{/if}
        </section>
    </div>
</div>

<dialog bind:this={formDialog} onclose={() => formOpen = false} oncancel={() => formOpen = false} aria-labelledby="server-editor-title" class="server-editor custom-scrollbar max-h-[90dvh] w-[calc(100%-2rem)] max-w-xl overflow-auto rounded-3xl border border-border bg-bg-elevated p-6 text-fg shadow-elevated"><div class="flex items-center justify-between gap-3"><h2 id="server-editor-title" class="text-xl font-semibold">{t(editing ? 'localServer.edit' : 'localServer.create')}</h2><button type="button" class={button({ variant: 'ghost', size: 'icon' })} aria-label={t('localServer.cancel')} disabled={busy} onclick={() => formOpen = false}><X class="h-4 w-4" /></button></div><form class="mt-5 space-y-5" aria-label={t(editing ? 'localServer.edit' : 'localServer.create')} onsubmit={event => { event.preventDefault(); void save(); }}>

        <label class="block space-y-2"><span class="text-sm text-fg-muted">{t('localServer.name')}</span><input class="server-field" maxlength="64" required bind:value={name} /></label><label class="block space-y-2"><span class="text-sm text-fg-muted">{t('localServer.version')}</span><select class="server-field" required disabled={Boolean(editing)} bind:value={version}>{#if version && !choices.some(item => item.id === version)}<option value={version}>{version}</option>{/if}{#each choices as item}<option value={item.id}>{item.id}</option>{/each}</select></label>{#if versionError}<p role="alert" class="text-sm text-danger">{versionError}</p><button type="button" class={button({ variant: 'ghost', size: 'sm' })} onclick={loadVersions}>{t('localServer.retry')}</button>{/if}{#if !editing}<label class="flex items-center gap-3 text-xs text-fg-muted"><input type="checkbox" bind:checked={snapshots} />{t('localServer.snapshots')}</label>{/if}
        <div class="grid grid-cols-2 gap-4"><label class="space-y-2"><span class="text-sm text-fg-muted">{t('localServer.memory')}</span><input class="server-field" type="number" min="512" max="8192" step="256" required bind:value={ram} /></label><label class="space-y-2"><span class="text-sm text-fg-muted">{t('localServer.players')}</span><input class="server-field" type="number" min="1" max="20" required bind:value={players} /></label></div><p class="text-xs leading-6 text-fg-muted">{t('serverPanel.ramHelp')}</p>
        {#if !editing}<label class="flex items-start gap-3 text-sm leading-6"><input class="mt-1" type="checkbox" required bind:checked={eula} /><span>{t('localServer.eula')} <a href="https://www.minecraft.net/eula" target="_blank" rel="noreferrer" class="text-brand-400 underline">Minecraft EULA</a></span></label>{/if}
        <div class="flex justify-end gap-3 border-t border-border pt-4"><button type="button" class={button({ variant: 'secondary' })} disabled={busy} onclick={() => formOpen = false}>{t('localServer.cancel')}</button><button type="submit" class={button({ variant: 'primary' })} disabled={busy || !pro || !version || !name.trim() || (!editing && !eula) || ram > 8192}>{busy ? t('localServer.loading') : t(editing ? 'localServer.save' : 'localServer.create')}</button></div>
    </form></dialog>

<dialog bind:this={deleteDialog} onclose={() => deleteOpen = false} oncancel={() => deleteOpen = false} aria-labelledby="delete-server-title" class="server-editor w-[calc(100%-2rem)] max-w-md rounded-2xl border border-border bg-bg-elevated p-6 text-fg shadow-elevated"><h2 id="delete-server-title" class="text-xl font-semibold">{t('hosting.delete')}</h2><p class="mt-3 text-sm leading-7 text-fg-muted">{t('consoleTools.deleteAll')}</p><p class="mt-3 font-semibold">{config?.name}</p>{#if deletion}<p class="mt-2 text-xs text-fg-muted">{deletion.files} {t('serverPanel.files')} · {(deletion.bytes / (1024 * 1024)).toFixed(1)} MiB</p>{/if}<form class="mt-4 space-y-4" onsubmit={event => { event.preventDefault(); void removeServer(); }}><label class="block space-y-2"><span class="text-xs text-fg-muted">{t('hosting.confirmName')}</span><input class="server-field" required bind:value={deleteName} autocomplete="off" /></label><div class="flex justify-end gap-2"><button type="button" class={button({variant:'secondary'})} onclick={() => deleteOpen = false}>{t('localServer.cancel')}</button><button type="submit" class={button({variant:'danger'})} disabled={busy || !config || deleteName !== config.name}>{t('hosting.delete')}</button></div></form></dialog>

{#if creatorOpen}<ServerCreator onclose={() => creatorOpen = false} oncreated={server => { selected = server.id; tab = 'overview'; creatorOpen = false; void refresh(); toast(t('creator.prepared'),'success'); }} />{/if}

<style>
    .server-editor { margin:auto; } .server-editor::backdrop { background:rgb(var(--bg-overlay)/.8); }
    :global(.server-action) { min-height:42px; border-radius:9px; font-weight:600; letter-spacing:.01em; box-shadow:inset 0 1px 0 rgb(var(--fg)/.05); }
    :global(.server-action.launcher-button--primary) { background:rgb(var(--brand-action)); border-color:rgb(var(--brand-action)); color:rgb(var(--brand-foreground)); box-shadow:0 3px 10px rgb(var(--brand-500)/.18); }
    :global(.server-action.launcher-button--secondary) { background:rgb(var(--bg-elevated)); border-color:rgb(var(--border-strong)/.65); }
    :global(.server-action:hover:not(:disabled)) { transform:translateY(-1px); border-color:rgb(var(--brand-400)/.6); box-shadow:0 4px 14px rgb(var(--bg-overlay)/.2); }
    :global(.server-action:active:not(:disabled)) { transform:translateY(0); }
    .server-tab { border-radius:9px; margin:2px 0; border:1px solid transparent; justify-content:flex-start; } .server-tab[aria-current="page"] { background:rgb(var(--brand-500)/.1); }
    @media(prefers-reduced-motion:reduce) { :global(.server-action) { transform:none !important; transition:none; } }

    .server-shell { background:rgb(var(--bg-elevated)/.97); display:grid; grid-template-columns:240px minmax(0,1fr); height:calc(100dvh - 170px); min-height:360px; max-height:900px; }
    .server-list { border-right-width:1px; display:flex; flex-direction:column; gap:8px; background:rgb(var(--bg-subtle)); }
    .server-field { width:100%; min-height:44px; border:1px solid rgb(var(--border)); border-radius:12px; background:rgb(var(--bg-subtle)); color:rgb(var(--fg)); padding:10px 12px; }
    .server-field:focus-visible,.server-item:focus-visible,.server-tab:focus-visible { outline:2px solid rgb(var(--brand-400)); outline-offset:2px; }
    .server-item,.server-tab { transition:background-color 140ms ease,border-color 140ms ease,color 140ms ease; }
    .server-item:hover { background:rgb(var(--brand-500)/.08); }
    .server-content { overscroll-behavior:contain; }
    .server-content > :global(*) { animation:server-section-enter .22s ease-out; }
    :global(.server-action:not(:disabled):hover) { transform:translateY(-1px); box-shadow:0 5px 16px rgb(var(--brand-500)/.12),inset 0 1px 0 rgb(var(--fg)/.07); }
    :global(.server-action:not(:disabled):active) { transform:scale(.98); }
    .server-tab:hover { transform:translateX(2px); }
    @keyframes server-section-enter { from { opacity:0; transform:translateY(6px); } to { opacity:1; transform:translateY(0); } }

    @media(max-width:1100px) { .server-shell { grid-template-columns:190px minmax(0,1fr); } }
    @media(max-width:760px) { .server-shell { grid-template-columns:minmax(0,1fr); grid-template-rows:auto minmax(0,1fr); height:calc(100dvh - 190px); } .server-list { display:flex; gap:8px; align-items:center; border-right:0; border-bottom:1px solid rgb(var(--border)); max-height:112px; padding:8px; } .server-list>p { display:none; } .server-item { min-width:160px; margin:0; padding:10px; } .server-item>span:nth-child(3) { display:none; } }
    @media(prefers-reduced-motion:reduce) { .server-item,.server-tab,:global(.server-action) { transition:none; transform:none!important; } .server-content > :global(*) { animation:none; } }
    .hosting-server-choices { max-height:25dvh; overflow:auto; flex-shrink:0; }
    .hosting-section-nav { display:grid; gap:3px; }
    .hosting-nav-label { margin:12px 8px 0; font-size:10px; font-weight:700; letter-spacing:.12em; text-transform:uppercase; color:rgb(var(--fg-muted)); }
    .server-item { border-radius:10px; padding:12px; } .server-item>span:nth-child(2) { margin-top:6px; } .server-item>span:nth-child(3) { display:none; }
    .hosting-network,.hosting-recovery { border:1px solid rgb(var(--border)); background:rgb(var(--bg-subtle)); border-radius:12px; padding:20px; }
    .hosting-network-icon { padding:10px; border-radius:10px; background:rgb(var(--brand-500)/.12); color:rgb(var(--brand-400)); }
    @media(max-width:760px) { .hosting-section-nav { display:flex; min-width:460px; } .hosting-nav-label,.hosting-server-choices { display:none; } .server-list { flex-direction:row; overflow-x:auto; } .server-list>button { display:none; } }
</style>
