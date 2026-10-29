<script lang="ts">
    import { serverMessage } from '$lib/utils/serverMessages';
    import { onMount } from 'svelte';
    import { SlidersHorizontal, Gauge, ShieldCheck, Image, Settings2, Save } from 'lucide-svelte';
    import { open } from '@tauri-apps/plugin-dialog';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    import { serverSettings, saveServerSettings, serverManage, serverManagedComponents, type ManagedSkinSupport, type ServerSettings, type ServerLoader } from '$lib/api/localServers';
    let { id, running, pro }: { id: string; running: boolean; pro: boolean } = $props();
    let value = $state<ServerSettings | null>(null);
    let managed = $state<ManagedSkinSupport|null>(null);
    let loader = $state<ServerLoader>({ kind: 'vanilla', version: '' });
    let versions = $state<{ id: string; stable: boolean }[]>([]);
    let busy = $state(false);
    let error = $state('');
    let saved = $state(false);
    let signature = $state('');
    const dirty = $derived(Boolean(value && JSON.stringify(value) !== signature));
    let disposed = false;
    onMount(() => { void Promise.all([serverSettings(id), serverManage<ServerLoader>({ action: 'loader', id }),serverManagedComponents(id)]).then(([settings, current, support]) => { if (!disposed) { value = {networkCompressionThreshold:256,entityBroadcastRangePercentage:100,maxTickTime:60000,opPermissionLevel:4,hideOnlinePlayers:false,...settings}; signature = JSON.stringify(value); loader = current; managed=support; } }).catch(cause => { if (!disposed) error = serverMessage(cause); }); return () => { disposed = true; }; });
    async function act(action: () => Promise<unknown>) { if (busy) return; busy = true; error = ''; saved = false; try { await action(); if (!disposed) saved = true; } catch (cause) { if (!disposed) error = serverMessage(cause); } finally { if (!disposed) busy = false; } }
    async function loaders() { loader.version = ''; versions = []; if (loader.kind === 'vanilla') return; await act(async () => { const next = await serverManage<{ id: string; stable: boolean }[]>({ action: 'loader_versions', id, kind: loader.kind }); if (!disposed) { versions = next; loader.version = next[0]?.id || ''; } }); saved = false; }
    async function icon() { const path = await open({ multiple: false, filters: [{ name: 'PNG / JPEG / WebP', extensions: ['png', 'jpg', 'jpeg', 'webp'] }] }); if (typeof path === 'string') await act(() => serverManage({ action: 'set_icon', id, path })); }
</script>

<div class="server-settings space-y-5 p-5 sm:p-6" data-server-settings>
    <header><p class="text-xs font-semibold uppercase tracking-widest text-brand-400">Luxmc / {t('serverPanel.settings')}</p><h3 class="mt-2 text-xl font-semibold">{t('serverSettingsDesign.title')}</h3><p class="mt-2 max-w-2xl text-sm leading-6 text-fg-muted">{t('serverPanel.restartHelp')}</p></header>
    {#if error}<p role="alert" class="break-words rounded-xl border border-danger/30 bg-danger/5 p-3 text-sm text-danger">{error}</p>{/if}
    {#if saved}<p role="status" class="text-sm text-success">{t(running ? 'serverPanel.savedRestart' : 'localServer.saved')}</p>{/if}
    {#if value}
    <form class="space-y-5" onsubmit={event => {event.preventDefault();void act(async()=>{if(value){value=await saveServerSettings(id,value);signature=JSON.stringify(value);}});}}>
        <fieldset disabled={busy || !pro} class="space-y-5 disabled:opacity-70">
            <section class="settings-section"><h4><SlidersHorizontal class="h-4 w-4" />{t('creator.world')}</h4><div class="settings-grid">
                <label class="sm:col-span-2"><span>{t('serverPanel.motd')}</span><textarea class="server-field" rows="2" maxlength="256" bind:value={value.motd}></textarea></label>
                <label><span>{t('serverPanel.difficulty')}</span><select class="server-field" bind:value={value.difficulty}>{#each ['peaceful','easy','normal','hard'] as difficulty}<option value={difficulty}>{t(`serverPanel.${difficulty}`)}</option>{/each}</select></label>
                <label><span>{t('serverPanel.gamemode')}</span><select class="server-field" bind:value={value.gamemode}>{#each ['survival','creative','adventure','spectator'] as mode}<option value={mode}>{t(`serverPanel.${mode}`)}</option>{/each}</select></label>
                <label class="sm:col-span-2"><span>{t('creator.seed')}</span><input class="server-field" maxlength="128" bind:value={value.seed} /><small>{t('creator.seedHelp')}</small></label>
                <label class="settings-toggle"><input type="checkbox" bind:checked={value.hardcore} /><span>{t('creator.hardcore')}</span></label><label class="settings-toggle"><input type="checkbox" bind:checked={value.generateStructures} /><span>{t('creator.generateStructures')}</span></label>
                <label class="settings-toggle"><input type="checkbox" bind:checked={value.forceGamemode} /><span>{t('serverPanel.forceGamemode')}</span></label><label class="settings-toggle"><input type="checkbox" bind:checked={value.pvp} /><span>PvP</span></label>
            </div></section>
            <section class="settings-section"><h4><Gauge class="h-4 w-4" />{t('serverSettingsDesign.performance')}</h4><div class="settings-grid">
                <label class="settings-toggle sm:col-span-2"><input type="checkbox" bind:checked={value.economyMode} /><span>{t('serverEconomy.title')}<small>{t('serverEconomy.help')}</small></span></label>
                <div class="flex flex-wrap gap-2 sm:col-span-2"><button type="button" class={button({variant:'secondary',size:'sm'})} onclick={()=>{if(value){value.viewDistance=6;value.simulationDistance=4;value.economyMode=true;}}}>{t('serverEconomy.apply')}</button><button type="button" class={button({variant:'secondary',size:'sm'})} onclick={()=>{if(value){value.viewDistance=4;value.simulationDistance=2;value.entityBroadcastRangePercentage=75;value.economyMode=true;}}}>{t('serverSettingsDesign.extraEconomy')}</button></div>
                <label><span>{t('serverPanel.view')}</span><input class="server-field" type="number" min="2" max="32" required bind:value={value.viewDistance} /></label><label><span>{t('serverPanel.simulation')}</span><input class="server-field" type="number" min="2" max="32" required bind:value={value.simulationDistance} /></label>
                <label><span>{t('serverPanel.autosave')}</span><input class="server-field" type="number" min="0" max="60" required bind:value={value.autosaveMinutes} /></label><label><span>{t('serverSettingsDesign.entities')}</span><input class="server-field" type="number" min="10" max="1000" aria-label={t('serverSettingsDesign.entities')} required bind:value={value.entityBroadcastRangePercentage} /><small>{t('serverSettingsDesign.entitiesHelp')}</small></label>
                <label><span>{t('serverSettingsDesign.compression')}</span><input class="server-field" type="number" min="-1" max="1048576" aria-label={t('serverSettingsDesign.compression')} required bind:value={value.networkCompressionThreshold} /><small>{t('serverSettingsDesign.compressionHelp')}</small></label><label><span>{t('serverSettingsDesign.watchdog')}</span><input class="server-field" type="number" min="-1" max="600000" step="1" aria-label={t('serverSettingsDesign.watchdog')} required bind:value={value.maxTickTime} /><small>{t('serverSettingsDesign.watchdogHelp')}</small></label>
            </div></section>
            <section class="settings-section"><h4><ShieldCheck class="h-4 w-4" />{t('serverSettingsDesign.access')}</h4><div class="settings-grid">
                <label><span>{t('localServer.port')} · {t('serverPanel.autoPort')}</span><input class="server-field" type="number" min="0" max="65535" required bind:value={value.port} /></label><label><span>{t('serverSettingsDesign.operator')}</span><select class="server-field" bind:value={value.opPermissionLevel}>{#each [0,1,2,3,4] as level}<option value={level}>{level}</option>{/each}</select></label>
                <label><span>{t('packs.spawnProtection')}</span><input class="server-field" type="number" min="0" max="64" required bind:value={value.spawnProtection} /></label><label><span>{t('packs.idleTimeout')}</span><input class="server-field" type="number" min="0" max="120" required bind:value={value.idleTimeout} /></label>
                <label class="settings-toggle"><input type="checkbox" bind:checked={value.allowFlight} /><span>{t('packs.allowFlight')}</span></label><label class="settings-toggle"><input type="checkbox" bind:checked={value.commandBlocks} /><span>{t('packs.commandBlocks')}</span></label>
                <label class="settings-toggle sm:col-span-2"><input type="checkbox" bind:checked={value.whitelist} /><span>{t('serverPanel.whitelist')}<small>{t('serverPanel.whitelistHelp')}</small></span></label><label class="settings-toggle sm:col-span-2"><input type="checkbox" bind:checked={value.hideOnlinePlayers} /><span>{t('serverSettingsDesign.hidePlayers')}</span></label>
            </div></section>
            <section class="settings-section"><h4><Image class="h-4 w-4" />{t('serverPanel.packUrl')}</h4>
                {#if value.resourcePackFile}<p class="text-sm leading-6 text-fg-muted">{t('packs.localManaged')} <strong class="text-fg">{value.resourcePackFile}</strong></p>{:else}<details><summary class="cursor-pointer text-sm font-medium">{t('packs.advancedUrl')}</summary><div class="settings-grid mt-4"><p class="text-xs leading-6 text-fg-muted sm:col-span-2">{t('serverPanel.packUrlHelp')}</p><label class="sm:col-span-2"><span>{t('serverPanel.packUrl')}</span><input class="server-field" type="url" placeholder="https://" bind:value={value.resourcePackUrl} /></label><label class="sm:col-span-2"><span>SHA-1</span><input class="server-field font-mono" maxlength="40" bind:value={value.resourcePackSha1} /></label><label class="settings-toggle sm:col-span-2"><input type="checkbox" bind:checked={value.requireResourcePack} /><span>{t('serverPanel.requirePack')}</span></label></div></details>{/if}
            </section>
        </fieldset>
        <section class="settings-section"><h4><Settings2 class="h-4 w-4" />{t('serverPanel.loader')}</h4><p class="mb-4 text-xs leading-6 text-fg-muted">{t('serverPanel.loaderHelp')}</p><div class="settings-grid"><label><span>{t('serverPanel.loader')}</span><select class="server-field" disabled={busy || running || !pro} bind:value={loader.kind} onchange={loaders}>{#each ['vanilla','fabric','quilt','forge','neoforge'] as kind}<option value={kind}>{kind === 'neoforge' ? 'NeoForge' : kind.charAt(0).toUpperCase()+kind.slice(1)}</option>{/each}</select></label>{#if loader.kind !== 'vanilla'}<label><span>{t('localServer.version')}</span><select class="server-field" disabled={busy || running || !pro} bind:value={loader.version}>{#if loader.version && !versions.some(version=>version.id===loader.version)}<option value={loader.version}>{loader.version}</option>{/if}{#each versions as version}<option value={version.id}>{version.id}</option>{/each}</select></label>{/if}</div><div class="mt-4 flex flex-wrap gap-2"><button type="button" class={button({variant:'secondary'})} disabled={busy || running || !pro || (loader.kind !== 'vanilla' && !loader.version)} onclick={()=>act(()=>serverManage({action:'set_loader',id,loader}))}>{t('serverPanel.configureLoader')}</button><button type="button" class={button({variant:'secondary'})} disabled={busy || running || !pro} onclick={icon}>{t('serverPanel.serverIcon')}</button></div></section>
        <section class="settings-section" data-managed-skin-support><h4><ShieldCheck class="h-4 w-4" />{t('managedSkins.title')}</h4><p class="text-sm font-medium">Luxmc Skin Bridge <span class="ml-2 text-xs text-brand-400">{t('managedSkins.badge')}</span></p><p class="mt-3 text-xs leading-6 text-fg-muted">{t('managedSkins.serverHelp')}</p>{#if managed}<p class="mt-3 text-xs text-fg-muted">{t(managed.valid ? 'managedSkins.ready' : 'managedSkins.repair')}</p>{/if}</section>
        <footer class="settings-save flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-border bg-bg-subtle p-4" data-server-settings-save><p class="text-xs leading-6 text-fg-muted">{t(dirty ? 'serverSettingsDesign.unsaved' : 'serverSettingsDesign.upToDate')}</p><button type="submit" class={button({variant:'primary'})} disabled={busy || !pro || !dirty}><Save class="h-4 w-4" />{t('serverPanel.saveSettings')}</button></footer>
    </form>
    {:else}<p class="text-sm text-fg-muted">{t('localServer.loading')}</p>{/if}
</div>

<style>
    .settings-section { border:1px solid rgb(var(--border)); border-radius:16px; padding:20px; background:rgb(var(--bg-subtle)/.65); }
    .settings-section h4 { display:flex; align-items:center; gap:10px; margin-bottom:18px; font-size:14px; font-weight:600; }
    .settings-section h4 :global(svg) { color:rgb(var(--brand-400)); }
    .settings-grid { display:grid; gap:20px; }
    .settings-grid label>span { display:block; margin-bottom:8px; font-size:12px; color:rgb(var(--fg-muted)); }
    .settings-grid small { display:block; margin-top:7px; font-size:11px; line-height:1.6; color:rgb(var(--fg-muted)); }
    .settings-toggle { display:flex; align-items:flex-start; gap:12px; border:1px solid rgb(var(--border)); padding:14px; border-radius:12px; }
    .settings-toggle input { margin-top:3px; accent-color:rgb(var(--brand-400)); }
    .settings-grid .settings-toggle>span { margin:0; color:rgb(var(--fg)); }
    .server-field { display:block; width:100%; min-height:44px; border:1px solid rgb(var(--border)); border-radius:10px; background:rgb(var(--bg-elevated)); color:rgb(var(--fg)); padding:10px 12px; }
    .server-field:focus-visible { outline:2px solid rgb(var(--brand-400)); outline-offset:2px; }
    .settings-save { position:static; }
    @media(min-width:640px){.settings-grid{grid-template-columns:repeat(2,minmax(0,1fr));}}
</style>
