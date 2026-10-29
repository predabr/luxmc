<script lang="ts">
    import { serverMessage } from '$lib/utils/serverMessages';
    import { onMount } from 'svelte';
    import { X, ArrowLeft, ArrowRight, Check, Search, Server, Package, Blocks, LoaderCircle, ShieldCheck, Cpu, Globe } from 'lucide-svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from './serverButton';
    import { versionsList, loadersVersions } from '$lib/api/launch';
    import { modsSearch } from '$lib/api/mods';
    import type { ModSearchResultItem } from '$lib/api/types';
    import { createLocalServer, serverManage, serverSettings, saveServerSettings, type LocalServerConfig, type ServerSettings, type ServerCatalogVersion } from '$lib/api/localServers';

    let { onclose, oncreated }: { onclose: () => void; oncreated: (server: LocalServerConfig) => void } = $props();
    let dialog: HTMLDialogElement;
    let step = $state(0); let name = $state(''); let software = $state('vanilla'); let version = $state(''); let loaderVersion = $state('');
    let versions = $state<{id:string;versionType:string}[]>([]); let loaderChoices = $state<{id:string;stable:boolean}[]>([]);
    let source = $state('modrinth'); let query = $state(''); let results = $state<ModSearchResultItem[]>([]); let pack = $state<ModSearchResultItem | null>(null);
    let packVersions = $state<ServerCatalogVersion[]>([]); let packVersion = $state(''); let busy = $state(false); let pending = $state(false); let error = $state(''); let phase = $state(''); let eula = $state(false);
    let ram = $state(2048); let players = $state(10); let created = $state<LocalServerConfig | null>(null); let disposed = false; let request = 0;
    let rules = $state<ServerSettings>({ motd: 'Luxmc', difficulty:'normal', gamemode:'survival', forceGamemode:true, pvp:true, whitelist:false, viewDistance:6, simulationDistance:4, port:0, autosaveMinutes:5, resourcePackUrl:'', resourcePackSha1:'', requireResourcePack:false, seed:'', hardcore:false, generateStructures:true,spawnProtection:16,allowFlight:false,commandBlocks:false,idleTimeout:0,economyMode:true });
    const chosenPack = $derived(packVersions.find(item => item.id === packVersion));
    const valid = $derived(step === 0 ? Boolean(name.trim() && version && (software === 'modpack' ? pack && chosenPack : software === 'vanilla' || loaderVersion)) : step === 2 ? Number.isInteger(ram) && ram >= 512 && ram <= 8192 && Number.isInteger(players) && players >= 1 && players <= 20 : true);
    onMount(() => {
        dialog.showModal();
        void versionsList().then(result => { if (!disposed) { versions = result.versions.filter(item => item.versionType === 'release'); version = result.latestRelease; } }).catch(cause => { if (!disposed) error = serverMessage(cause); });
        return () => { disposed = true; request++; };
    });
    async function loadLoaders() {
        const revision = ++request; loaderChoices = []; loaderVersion = ''; error = '';
        if (['vanilla','modpack'].includes(software) || !version) return;
        pending = true;
        try { const result = await loadersVersions(software, version); if (!disposed && revision === request) { loaderChoices = result.versions; loaderVersion = result.versions[0]?.id || ''; if (!loaderVersion) error = t('creator.noLoader'); } }
        catch (cause) { if (!disposed && revision === request) error = serverMessage(cause); }
        finally { if (!disposed && revision === request) pending = false; }
    }
    async function search() {
        if (pending) return; pending = true; error = ''; const revision = ++request;
        try { const next = await modsSearch(query, '', 24, 0, 'modpack', 'downloads', '', '', source); if (!disposed && revision === request) results = next; }
        catch (cause) { if (!disposed && revision === request) error = serverMessage(cause); }
        finally { if (!disposed && revision === request) pending = false; }
    }
    async function choose(item: ModSearchResultItem) {
        if (pending) return; pending = true; error = ''; pack = item; packVersions = []; packVersion = ''; version = ''; const revision = ++request;
        try {
            const next = await serverManage<ServerCatalogVersion[]>({ action:'catalog_versions', project:item.sourceId || item.slug, source:item.source });
            if (!disposed && revision === request) { packVersions = next.filter(item => item.gameVersions?.length); packVersion = packVersions[0]?.id || ''; version = packVersions[0]?.gameVersions[0] || ''; if (!packVersion) error = t('creator.noServerPack'); }
        } catch (cause) { if (!disposed && revision === request) error = serverMessage(cause); }
        finally { if (!disposed && revision === request) pending = false; }
    }
    function changeSoftware(value: string) { software = value; pack = null; packVersion = ''; packVersions = []; error = ''; version = versions[0]?.id || ''; void loadLoaders(); if(value === 'modpack') { results=[]; void search(); } }
    async function prepare() {
        if (busy || !eula || !valid) return;
        busy = true; error = '';
        try {
            phase = t('creator.reserving');
            const server = created || await createLocalServer({name,mcVersion:version,ramMb:ram,maxPlayers:players,eulaAccepted:eula}); created = server;
            phase = t('creator.applying');
            const defaults = await serverSettings(server.id);
            await saveServerSettings(server.id, {...defaults,...rules,motd:rules.motd === 'Luxmc' ? name : rules.motd});
            if (software === 'modpack' && pack) {
                phase = t('creator.downloading');
                await serverManage({action:'install_catalog_pack',id:server.id,project:pack.sourceId || pack.slug,version:packVersion,source:pack.source});
            } else if (software !== 'vanilla') {
                await serverManage({action:'set_loader',id:server.id,loader:{kind:software,version:loaderVersion}});
            }
            oncreated(server);
        } catch (cause) { if (!disposed) error = `${serverMessage(cause)}${created ? ` · ${t('creator.retrySame')}` : ''}`; }
        finally { if (!disposed) { busy = false; phase = ''; } }
    }
</script>

<dialog bind:this={dialog} class="creator text-fg" aria-labelledby="creator-title" oncancel={event => { event.preventDefault(); if (!busy) onclose(); }}>
    <header class="creator-header"><div><p class="text-xs font-semibold uppercase tracking-widest text-brand-400">Luxmc / {t('creator.dedicated')}</p><h2 id="creator-title" class="mt-2 text-2xl font-semibold tracking-tight">{t('localServer.create')}</h2><p class="mt-2 text-sm text-fg-muted">{t('creator.independent')}</p></div><button type="button" class={button({variant:'ghost',size:'icon'})} disabled={busy} aria-label={t('localServer.cancel')} onclick={onclose}><X class="h-5 w-5" /></button></header>
    <ol class="creator-steps" aria-label={t('creator.steps')}>{#each ['software','world','resources','review'] as label,index}<li aria-current={step === index ? 'step' : undefined} class:complete={index < step}><span>{#if index < step}<Check class="h-4 w-4" />{:else}{index + 1}{/if}</span><strong>{t(`creator.${label}`)}</strong></li>{/each}</ol>
    <div class="creator-content custom-scrollbar">
        {#if error}<div role="alert" class="mb-5 rounded-xl border border-danger/30 bg-danger/5 p-4 text-sm leading-6 break-words">{error}</div>{/if}
        {#if busy}<div role="status" class="creator-progress"><LoaderCircle class="h-8 w-8 animate-spin text-brand-400" /><h3 class="text-lg font-semibold">{phase}</h3><p class="max-w-md text-sm leading-6 text-fg-muted">{t('creator.preparingHelp')}</p></div>
        {:else if step === 0}
            <label class="creator-label"><span>{t('localServer.name')}</span><input class="server-field" maxlength="64" bind:value={name} disabled={Boolean(created)} placeholder={t('creator.nameHint')} /></label>
            <div class="creator-software mt-5">{#each [{id:'vanilla',icon:Server},{id:'fabric',icon:Blocks},{id:'quilt',icon:Blocks},{id:'forge',icon:Blocks},{id:'neoforge',icon:Blocks},{id:'modpack',icon:Package}] as item}<button type="button" class="creator-choice" aria-pressed={software === item.id} disabled={Boolean(created) || pending} onclick={() => changeSoftware(item.id)}><item.icon class="h-5 w-5" /><span>{item.id === 'modpack' ? 'Modpack' : item.id === 'neoforge' ? 'NeoForge' : item.id.charAt(0).toUpperCase()+item.id.slice(1)}</span>{#if software === item.id}<Check class="ml-auto h-4 w-4" />{/if}</button>{/each}</div>
            {#if software === 'modpack'}
                <form class="creator-search mt-5" onsubmit={event => {event.preventDefault();void search();}}><input class="server-field min-w-0 flex-1" aria-label={t('serverPanel.search')} placeholder={t('serverPanel.search')} bind:value={query} disabled={Boolean(created)} /><select class="server-field" aria-label={t('serverPanel.provider')} bind:value={source} disabled={Boolean(created) || pending}><option value="modrinth">Modrinth</option><option value="curseforge">CurseForge</option></select><button class={button({variant:'secondary'})} disabled={pending || Boolean(created)}><Search class="h-4 w-4" />{t('serverPanel.search')}</button></form>
                <p class="my-3 text-xs leading-6 text-fg-muted">{t('creator.catalogHelp')}</p>
                {#if pack}<div class="creator-selected"><Package class="h-5 w-5 shrink-0 text-brand-400" /><div class="min-w-0 flex-1"><p class="font-semibold">{pack.title}</p><p class="text-xs text-fg-muted">{pack.author || pack.source}</p></div>{#if !created}<button type="button" class={button({variant:'ghost',size:'sm'})} disabled={pending} onclick={() => {pack=null;packVersion='';}}>{t('creator.change')}</button>{/if}</div>{/if}
                {#if !pack}<div class="creator-results">{#each results as result (`${result.source}:${result.sourceId || result.slug}`)}<button type="button" class="creator-result" disabled={pending || Boolean(created)} onclick={() => choose(result)}>{#if result.iconUrl}<img src={result.iconUrl} alt="" loading="lazy" />{:else}<Package class="h-10 w-10 shrink-0" />{/if}<span class="min-w-0"><strong class="block truncate">{result.title}</strong><small class="mt-1 block line-clamp-2 text-fg-muted">{result.description}</small></span><ArrowRight class="ml-auto h-4 w-4 shrink-0" /></button>{/each}</div>{/if}
                {#if chosenPack}<label class="creator-label mt-4"><span>{t('creator.packVersion')}</span><select class="server-field" bind:value={packVersion} disabled={Boolean(created)} onchange={() => version = chosenPack?.gameVersions[0] || ''}>{#each packVersions as item}<option value={item.id}>{item.name}</option>{/each}</select></label><label class="creator-label mt-4"><span>Minecraft</span><select class="server-field" bind:value={version} disabled={Boolean(created)}>{#each chosenPack.gameVersions as game}<option value={game}>{game}</option>{/each}</select></label>{/if}
            {:else}<div class="mt-5 grid gap-4 sm:grid-cols-2"><label class="creator-label"><span>Minecraft</span><select class="server-field" bind:value={version} disabled={Boolean(created) || pending} onchange={loadLoaders}>{#each versions as item}<option value={item.id}>{item.id}</option>{/each}</select></label>{#if software !== 'vanilla'}<label class="creator-label"><span>{t('serverPanel.loader')}</span><select class="server-field" bind:value={loaderVersion} disabled={pending || Boolean(created)}>{#each loaderChoices as item}<option value={item.id}>{item.id}{item.stable ? ' ✓' : ''}</option>{/each}</select></label>{/if}</div>{/if}
            {#if pending}<p role="status" class="mt-4 flex items-center gap-2 text-sm text-fg-muted"><LoaderCircle class="h-4 w-4 animate-spin" />{t('localServer.loading')}</p>{/if}
        {:else if step === 1}
            <div class="creator-notice"><Globe class="h-5 w-5 shrink-0 text-brand-400" /><p>{t('creator.newWorld')}</p></div>
            <div class="mt-5 grid gap-5 sm:grid-cols-2"><label class="creator-label"><span>{t('serverPanel.gamemode')}</span><select class="server-field" bind:value={rules.gamemode}>{#each ['survival','creative','adventure','spectator'] as mode}<option value={mode}>{t(`serverPanel.${mode}`)}</option>{/each}</select></label><label class="creator-label"><span>{t('serverPanel.difficulty')}</span><select class="server-field" bind:value={rules.difficulty}>{#each ['peaceful','easy','normal','hard'] as difficulty}<option value={difficulty}>{t(`serverPanel.${difficulty}`)}</option>{/each}</select></label><label class="creator-label sm:col-span-2"><span>{t('creator.seed')}</span><input class="server-field" maxlength="128" bind:value={rules.seed} placeholder={t('creator.randomSeed')} /></label></div>
            <div class="mt-5 grid gap-3 sm:grid-cols-2">{#each ['pvp','hardcore','generateStructures','whitelist'] as key}<label class="creator-toggle"><input type="checkbox" checked={Boolean(rules[key as keyof ServerSettings])} onchange={event => rules = {...rules,[key]:event.currentTarget.checked}} /><span>{key === 'pvp' ? 'PvP' : t(`creator.${key}`)}</span></label>{/each}</div>
            <details class="mt-5 rounded-xl border border-border p-4"><summary class="cursor-pointer text-sm font-semibold">{t('packs.moreRules')}</summary><div class="mt-4 grid gap-4 sm:grid-cols-2"><label class="creator-label"><span>{t('packs.spawnProtection')}</span><input class="server-field" type="number" min="0" max="64" bind:value={rules.spawnProtection} /></label><label class="creator-label"><span>{t('packs.idleTimeout')}</span><input class="server-field" type="number" min="0" max="120" bind:value={rules.idleTimeout} /></label><label class="creator-toggle"><input type="checkbox" bind:checked={rules.allowFlight} />{t('packs.allowFlight')}</label><label class="creator-toggle"><input type="checkbox" bind:checked={rules.commandBlocks} />{t('packs.commandBlocks')}</label></div></details>
        {:else if step === 2}
            <div class="creator-notice"><Cpu class="h-5 w-5 shrink-0 text-brand-400" /><p>{t('creator.resourceHelp')}</p></div>
            <div class="mt-5 grid gap-5 sm:grid-cols-2"><label class="creator-label"><span>{t('localServer.memory')} · MiB</span><input class="server-field" type="number" min="512" max="8192" step="256" bind:value={ram} disabled={Boolean(created)} /></label><label class="creator-label"><span>{t('localServer.players')}</span><input class="server-field" type="number" min="1" max="20" bind:value={players} disabled={Boolean(created)} /></label></div>
            <div class="mt-5 flex flex-wrap gap-2">{#each ['economy','balanced'] as preset}<button type="button" class={button({variant:'secondary',size:'sm'})} onclick={() => { rules.viewDistance=preset === 'economy'?6:8;rules.simulationDistance=preset === 'economy'?4:6; }}>{t(`creator.${preset}`)}</button>{/each}</div>
            <div class="mt-4 grid gap-5 sm:grid-cols-2"><label class="creator-label"><span>{t('serverPanel.view')}</span><input class="server-field" type="number" min="2" max="32" bind:value={rules.viewDistance} /></label><label class="creator-label"><span>{t('serverPanel.simulation')}</span><input class="server-field" type="number" min="2" max="32" bind:value={rules.simulationDistance} /></label></div>
        {:else}
            <div class="creator-notice"><ShieldCheck class="h-5 w-5 shrink-0 text-success" /><p>{t('creator.reviewHelp')}</p></div>
            <dl class="creator-summary">{#each [{label:t('localServer.name'),value:name},{label:t('creator.software'),value:pack?.title || software},{label:'Minecraft',value:version},{label:t('creator.packVersion'),value:chosenPack?.name || loaderVersion || 'Vanilla'},{label:t('serverPanel.gamemode'),value:t(`serverPanel.${rules.gamemode}`)},{label:t('localServer.memory'),value:`${ram} MiB`},{label:t('localServer.players'),value:String(players)}] as row}<div><dt>{row.label}</dt><dd>{row.value}</dd></div>{/each}</dl>
            <p class="my-4 text-sm leading-6 text-fg-muted">{t('creator.localEntry')}</p>
            <label class="creator-toggle"><input type="checkbox" bind:checked={eula} /><span>{t('localServer.eula')} <a class="text-brand-400 underline" href="https://www.minecraft.net/eula" target="_blank" rel="noreferrer">Minecraft EULA</a></span></label>
        {/if}
    </div>
    <footer class="creator-footer"><button type="button" class={button({variant:'secondary'})} disabled={busy} onclick={() => step > 0 ? step-- : onclose()}><ArrowLeft class="h-4 w-4" />{t(step > 0 ? 'creator.back' : 'localServer.cancel')}</button><span class="text-xs text-fg-muted">{step + 1} / 4</span>{#if step < 3}<button type="button" class={button({variant:'primary'})} disabled={!valid || pending || busy} onclick={() => step++}>{t('creator.next')}<ArrowRight class="h-4 w-4" /></button>{:else}<button type="button" class={button({variant:'primary'})} disabled={!eula || busy} onclick={prepare}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Check class="h-4 w-4" />{/if}{t(created ? 'localServer.retry' : 'creator.createPrepare')}</button>{/if}</footer>
</dialog>

<style>
    .creator{margin:auto;width:min(860px,calc(100vw - 32px));max-height:min(900px,94dvh);border:1px solid rgb(var(--border-strong)/.6);border-radius:24px;background:rgb(var(--bg-elevated));box-shadow:0 24px 90px rgb(var(--bg-overlay)/.5);padding:0;overflow:hidden;display:flex;flex-direction:column;animation:creator-enter .22s ease-out}.creator:not([open]){display:none}.creator::backdrop{background:rgb(var(--bg-overlay)/.82)}.creator-header{display:flex;justify-content:space-between;gap:16px;padding:26px 28px 20px}.creator-steps{display:flex;gap:12px;padding:0 28px 22px;border-bottom:1px solid rgb(var(--border))}.creator-steps li{display:flex;align-items:center;gap:8px;flex:1;font-size:12px;color:rgb(var(--fg-muted))}.creator-steps li>span{display:grid;place-items:center;width:28px;height:28px;border-radius:9px;background:rgb(var(--fg)/.05)}.creator-steps li[aria-current],.creator-steps .complete{color:rgb(var(--brand-400))}.creator-steps li[aria-current]>span{background:rgb(var(--brand-500)/.16)}.creator-content{min-height:220px;overflow:auto;padding:24px 28px;flex:1}.creator-footer{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:18px 28px;border-top:1px solid rgb(var(--border));background:rgb(var(--bg-subtle)/.7)}.creator-label{display:flex;flex-direction:column;gap:8px;font-size:13px;font-weight:500}.server-field{width:100%;min-height:44px;padding:11px 13px;border:1px solid rgb(var(--border));border-radius:11px;background:rgb(var(--bg-subtle));color:rgb(var(--fg));font-weight:400}.server-field:focus-visible{outline:2px solid rgb(var(--brand-400));outline-offset:2px}.creator-software{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:10px}.creator-choice{display:flex;align-items:center;gap:10px;padding:16px;border:1px solid rgb(var(--border));border-radius:13px;background:rgb(var(--fg)/.025);text-align:left;font-size:13px;transition:transform .16s,border-color .16s,background .16s}.creator-choice:hover{transform:translateY(-2px);border-color:rgb(var(--brand-400)/.6)}.creator-choice[aria-pressed=true]{background:rgb(var(--brand-500)/.12);border-color:rgb(var(--brand-400)/.7);color:rgb(var(--brand-400))}.creator-notice,.creator-selected{display:flex;align-items:center;gap:12px;padding:16px;border:1px solid rgb(var(--border));background:rgb(var(--brand-500)/.045);border-radius:13px;font-size:13px;line-height:1.7}.creator-search{display:grid;grid-template-columns:minmax(0,1fr) 145px auto;gap:8px}.creator-results{display:grid;gap:8px}.creator-result{display:flex;align-items:center;gap:14px;padding:12px;border:1px solid rgb(var(--border));border-radius:12px;text-align:left;transition:background .16s}.creator-result:hover{background:rgb(var(--fg)/.045)}.creator-result img{width:48px;height:48px;object-fit:contain;border-radius:10px}.creator-toggle{display:flex;align-items:center;gap:12px;padding:14px;border:1px solid rgb(var(--border));border-radius:12px;font-size:13px;line-height:1.7}.creator-summary{margin-top:20px}.creator-summary>div{display:flex;justify-content:space-between;gap:20px;padding:11px 0;border-bottom:1px solid rgb(var(--border));font-size:13px}.creator-summary dt{color:rgb(var(--fg-muted))}.creator-summary dd{text-align:right;overflow-wrap:anywhere}.creator-progress{display:flex;min-height:270px;align-items:center;justify-content:center;flex-direction:column;gap:18px;text-align:center}@keyframes creator-enter{from{opacity:0;transform:translateY(12px) scale(.985)}to{opacity:1;transform:none}}@media(max-width:600px){.creator-search{grid-template-columns:minmax(0,1fr) auto}.creator-search>input{grid-column:1/-1}.creator-header,.creator-content{padding:18px}.creator-steps{padding:0 18px 16px;gap:6px}.creator-steps strong{display:none}.creator-software{grid-template-columns:repeat(2,minmax(0,1fr))}.creator-footer{padding:14px 18px}}@media(prefers-reduced-motion:reduce){.creator{animation:none}.creator-choice,.creator-result{transition:none;transform:none}}
</style>
