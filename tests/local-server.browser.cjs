const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/local-server-reform';
    fs.mkdirSync(output, { recursive: true });
    try {
        for (const [width, height] of (process.env.LUXMC_TEST_LIGHT === '1' ? [[1440,900]] : [[1440, 900], [1366, 768], [960, 600]])) {
            console.log(`Checking ${width}x${height}`);
            const page = await browser.newPage({ viewport: { width, height }, reducedMotion: 'reduce' });
            page.setDefaultTimeout(15000); page.setDefaultNavigationTimeout(30000);
            const errors = []; page.on('pageerror', error => { errors.push(error.message); console.error('Browser runtime:', error.stack); });
            await setupLauncherDemo(page);
            await page.addInitScript(light => {
                window.serverCalls = []; window.localServerConfigs = []; window.localServerActive = null; window.localServerRoom = null; window.serverBackups = []; window.deletedServers = []; window.failCatalogOnce = true; window.catalogPending=null; window.serverContents=[{kind:'mod',name:'server-content.jar',bytes:4096,staged:false,remove:false}];
                window.serverSettings = { motd: 'Meu servidor', difficulty: 'normal', gamemode: 'survival', pvp: true, whitelist: false, viewDistance: 8, simulationDistance: 6, port: 0, autosaveMinutes: 5, resourcePackUrl: '', resourcePackSha1: '', requireResourcePack: false,resourcePackFile:'',spawnProtection:16,allowFlight:false,commandBlocks:false,idleTimeout:0 };
                const original = window.electronAPI.invoke;
                const invoke = async (command, args = {}) => {
                    window.serverCalls.push({ command, args });
                    if(light && command === 'plugin:store|get' && args.key === 'app') {const [settings,found]=await original(command,args);return [{...settings,theme:'default-light',customBackground:'none',liveWallpaper:false},found];}
                    if(light && command === 'lux_account_sync') {const value=await original(command,args);return value?{...value,preferences:{...value.preferences,theme:'default-light'}}:value;}
                    if (command === 'membership_request' && args.request?.action === 'preferences') return { ok: true, revision: 2 };
                    if (command === 'membership_request') return { identity: { id: '00000000-0000-4000-8000-000000000001', username: 'LuxPlayer', owner: true }, membership: { plan: 'pro', expiresAt: Math.floor(Date.now() / 1000) + 86400, revision: 1, collections: 24, presets: 20 }, settings: { preferences: {}, presets: [], revision: 1 } };
                    if (command === 'tunnel_status') return null;
                    if (command === 'stop_session') { window.localServerRoom = null; return null; }
                    if (command === 'mods_search' && args.contentType === 'modpack') return [{source:'modrinth',sourceId:'server-pack',slug:'server-pack',title:'Pack de servidor',description:'Fixture de instalação independente',author:'Autor',iconUrl:null}];
                    if (command === 'profiles_list') return [];
                    if (command === 'plugin:dialog|open') return 'C:\\fixtures\\pack.zip';
                    if (command === 'versions_list') return { versions: [{ id: '1.21.1', versionType: 'release' }], latestRelease: '1.21.1', latestSnapshot: '26w01a' };
                    if (command === 'local_server_list') return { servers: structuredClone(window.localServerConfigs), active: window.localServerActive ? { ...structuredClone(window.localServerActive), console: window.localServerActive.console.filter(line => line.sequence > (args.after || 0)) } : null, warnings: [] };
                    if (command === 'local_server_create') { const server = { ...args.input, id: crypto.randomUUID() }; window.localServerConfigs.push(server); return structuredClone(server); }
                    if (command === 'local_server_update') { const server = window.localServerConfigs.find(item => item.id === args.id); Object.assign(server, { name: args.name, ramMb: args.ramMb, maxPlayers: args.maxPlayers }); return structuredClone(server); }
                    if (command === 'local_server_start') { window.localServerActive = { id: args.id, generation: crypto.randomUUID(), phase: 'running', port: 34567, pid: 123, maxPlayers: 20, playersOnline: 1, error: null, exitCode: null, console: [{ sequence: 1, stream: 'stdout', text: 'Done (0.1s)! Ready' }], cursor: 1, lastSavedAt: null, forcedStop: false, historyComplete: true, players: [{ uuid: '00000000-0000-4000-8000-000000000002', name: 'Friend', firstJoinedAt: new Date().toISOString(), lastJoinedAt: new Date().toISOString(), lastLeftAt: null, visits: 2, online: true }] }; return structuredClone(window.localServerActive); }
                    if (command === 'local_server_stop') { window.localServerActive.phase = 'stopped'; window.localServerActive.pid = null; window.localServerActive.playersOnline = 0; window.localServerActive.players.forEach(player => player.online = false); window.localServerRoom = null; return null; }
                    if (command === 'local_server_console') { const server = window.localServerActive; server.cursor++; server.console.push({ sequence: server.cursor, stream: 'stdout', text: 'Command received by fixture' }); return null; }
                    if (command === 'local_server_share') { window.localServerRoom = { mode: 'host', invitation: '1234567890123456', roomCode: '1234567890123456', localAddress: '127.0.0.1:34567', expiresAt: 9999999999, transport: 'QUIC', maxPlayers: 20, roomLocked: false, worldReady: true, worlds: [], hostedServer: { localServerId: `${args.id}:${window.localServerActive.generation}`, name: 'Servidor da turma', mcVersion: '1.21.1' }, members: [] }; return structuredClone(window.localServerRoom); }
                    if (command === 'local_server_open_folder') return null;
                    if (command === 'local_server_manage') {
                        const request = args.request;
                        if (request.action === 'connection_diagnostic') return {minecraftResponding:window.localServerActive?.phase==='running',address:window.localServerActive?.phase==='running'?'127.0.0.1:34567':null,phase:window.localServerActive?.phase||'stopped',tcpReachable:window.localServerActive?.phase==='running'};
                        if (request.action === 'console_text') return {text:(window.localServerActive?.console||[]).map(line=>line.text).join('\n'),truncated:false};
                        if (request.action === 'deletion_preview') return {files:47,bytes:1048576};
                        if (request.action === 'catalog_pending') return window.catalogPending;
                        if (request.action === 'public_tunnel') return { linked: false, address: '', agentRunning: false };
                        if (request.action === 'sharing_status') return structuredClone(window.localServerRoom);
                        if (request.action === 'stop_sharing') { window.localServerRoom = null; return null; }
                        if (request.action === 'sharing_lock') {window.localServerRoom.roomLocked=request.locked;return null;}
                        if (request.action === 'sharing_invitation') return structuredClone(window.localServerRoom);
                        if (request.action === 'managed_components') return {name:'Luxmc Skin Bridge',required:true,valid:true,clientComponentRequired:true};
                        if (request.action === 'settings') return structuredClone(window.serverSettings);
                        if (request.action === 'save_settings') { window.serverSettings = request.settings; return structuredClone(request.settings); }
                        if (request.action === 'loader') return { kind: 'vanilla', version: '' };
                        if (request.action === 'players') return { players: window.localServerActive?.players || [], banned: [], whitelist: [], operators: [] };
                        if (request.action === 'contents') return structuredClone(window.serverContents);
                        if (request.action === 'content_details') return request.names.map(name=>({name,title:name.includes('server-content')?'Mod de teste':null,icon:'/mods.png'}));
                        if (request.action === 'toggle_content') {const item=window.serverContents.find(item=>item.name===request.name);item.name=request.enabled?item.name.replace(/\.disabled$/,''):`${item.name}.disabled`;return null;}
                        if (request.action === 'remove_content') {const item=window.serverContents.find(item=>item.name===request.name);window.serverContents.push({...item,staged:true,remove:true});return null;}
                        if (request.action === 'import_content') {window.serverContents.push({kind:request.kind,name:'pack.zip',bytes:2048,staged:true,remove:false});return null;}
                        if (request.action === 'apply_content') {const removed=window.serverContents.filter(item=>item.staged&&item.remove).map(item=>item.name);window.serverContents=window.serverContents.filter(item=>!removed.includes(item.name));window.serverContents.forEach(item=>item.staged=false);return {applied:1,preserved:[]};}
                        if (request.action === 'use_resource_pack') {window.serverSettings.resourcePackFile=request.file;window.serverSettings.requireResourcePack=request.required;return null;}
                        if (request.action === 'instance_report') return {prepared:2,excluded:['mods/client-visual.jar'],localCopy:true};
                        if (request.action === 'browse_files') return {path:request.path,entries:request.path === '' ? [{name:'world',directory:true,bytes:0},{name:'config',directory:true,bytes:0},{name:'server.properties',directory:false,bytes:200}] : request.path === 'world' ? [{name:'level.dat',directory:false,bytes:1024}] : []};
                        if (request.action === 'catalog_versions') return [{id:'pack-version-1',name:'Pack 1.0',gameVersions:['1.21.1'],loaders:['neoforge']}];
                        if (request.action === 'install_catalog_pack') { if(window.failCatalogOnce) {window.failCatalogOnce=false;window.catalogPending={source:request.source,project:request.project,version:request.version};throw new Error('Fixture: rede indisponível');} window.catalogPending=null;return {prepared:2,excluded:1}; }
                        if (request.action === 'deleted_servers') return structuredClone(window.deletedServers);
                        if (request.action === 'delete_server') { const item = window.localServerConfigs.find(item => item.id === request.id); if (item.name !== request.name) throw new Error('Confirm name'); if(request.deleteBackups!==true) throw new Error('Delete must include owned backups'); window.localServerConfigs = window.localServerConfigs.filter(item => item.id !== request.id); return null; }
                        if (request.action === 'restore_server') { const item = window.deletedServers.find(item => item.id === request.id); window.localServerConfigs.push(item); window.deletedServers = window.deletedServers.filter(item => item.id !== request.id); return null; }
                        if (request.action === 'backups') return structuredClone(window.serverBackups);
                        if (request.action === 'backup') { const backup = { id: crypto.randomUUID(), createdAt: new Date().toISOString(), bytes: 1000, files: 2 }; window.serverBackups.push(backup); return backup; }
                        return null;
                    }
                    return original(command, args);
                };
                window.electronAPI.invoke = invoke; window.__TAURI_INTERNALS__.invoke = invoke;
            }, process.env.LUXMC_TEST_LIGHT === '1');
            const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';
            await page.goto(`${base}/hosting/server`);
            const create = page.getByRole('button', { name: 'Criar servidor', exact: true }).first();
            await create.waitFor({timeout:45000}).catch(async cause=>{console.error((await page.locator('body').innerText()).slice(-3000));await page.screenshot({path:`${output}/startup-failure-${width}.png`});throw cause;}); await page.waitForFunction(() => Array.from(document.querySelectorAll('button')).some(button => button.textContent.trim() === 'Criar servidor' && !button.disabled)).catch(async cause => { await page.screenshot({ path: `${output}/failure-${width}.png` }); console.error(await page.evaluate(() => ({ text: document.body.innerText.slice(-1500), membershipCalls: window.serverCalls?.filter(call => call.command === 'membership_request').length }))); throw cause; }); await create.click();
            const form = page.getByRole('dialog', { name: 'Criar servidor', exact: true });
            await form.getByLabel('Nome do servidor', { exact: true }).fill('Servidor da turma');
            await page.keyboard.press('Escape'); await form.waitFor({ state: 'hidden' }); await create.click();
            await form.getByLabel('Nome do servidor', { exact: true }).fill('Servidor da turma');
            await form.getByRole('button',{name:'Continuar',exact:true}).click();
            await form.getByRole('combobox',{name:/Modo de jogo/}).selectOption('creative');
            await form.getByLabel('Seed (opcional)',{exact:true}).fill('luxmc-world');
            await form.getByRole('button',{name:'Continuar',exact:true}).click();
            const memory = form.getByLabel(/RAM do servidor/);
            assert.equal(await memory.getAttribute('max'), '8192'); await memory.fill('8192');
            await form.getByRole('button',{name:'Continuar',exact:true}).click();
            assert.equal(await form.getByRole('button', { name: 'Criar e preparar', exact: true }).isDisabled(), true);
            assert.equal(await page.evaluate(() => window.serverCalls.some(call => call.command === 'local_server_create')), false);
            await form.getByRole('checkbox', { name: /Li e aceito a EULA/ }).check();
            await form.getByRole('button', { name: 'Criar e preparar', exact: true }).click(); await form.waitFor({ state: 'hidden' });
            assert.equal(await page.evaluate(() => window.localServerConfigs[0].ramMb), 8192);
            assert.equal(await page.evaluate(() => window.serverSettings.gamemode), 'creative');
            assert.equal(await page.evaluate(() => window.serverSettings.seed), 'luxmc-world');
            assert.equal(await page.evaluate(() => window.serverCalls.some(call => call.args?.request?.action === 'import_instance')),false);
            await page.getByRole('button', { name: 'Iniciar', exact: true }).click(); await page.getByText('Servidor pronto para jogar', { exact: true }).waitFor();
            await page.getByRole('button', {name:'127.0.0.1:34567',exact:true}).waitFor(); assert.equal(await page.getByText('localhost:34567',{exact:true}).count(),0);
            assert.equal(await page.getByText('Retomar preparação',{exact:true}).count(),0);
            await page.getByRole('button', { name: 'Compartilhar por P2P', exact: true }).click(); await page.getByRole('button', { name: 'Copiar código de conexão', exact: true }).waitFor();
            assert.equal(await page.getByRole('button', { name: 'Criar minha sala', exact: true }).count(), 0);
            assert.equal(await page.getByRole('spinbutton', { name: 'Porta local do servidor', exact: true }).count(), 0);
            await page.getByRole('link',{name:'Ver sala e participantes',exact:true}).click(); await page.getByText('Sua sala está conectada',{exact:true}).waitFor(); assert.equal(await page.getByRole('button',{name:'Criar minha sala',exact:true}).count(),0); await page.getByRole('button',{name:'Fechar novas entradas',exact:true}).click(); await page.waitForFunction(()=>window.localServerRoom.roomLocked); await page.getByRole('button',{name:'Permitir entradas',exact:true}).click(); await page.waitForFunction(()=>!window.localServerRoom.roomLocked); await page.getByRole('button',{name:'Parar compartilhamento',exact:true}).click(); await page.getByRole('button',{name:'Criar minha sala',exact:true}).waitFor(); assert.equal(await page.evaluate(()=>window.localServerActive.phase),'running'); await page.getByRole('link',{name:/Hospedar servidor local/}).first().click(); await page.getByRole('button',{name:'Compartilhar por P2P',exact:true}).click(); await page.getByRole('button',{name:'Copiar código de conexão',exact:true}).waitFor();
            await page.getByRole('button', { name: 'Console', exact: true }).click(); await page.getByRole('log').getByText('Done (0.1s)! Ready', { exact: true }).waitFor();
            await page.getByRole('textbox', { name: 'Comando do servidor', exact: true }).fill('list'); await page.getByRole('button', { name: 'Enviar comando', exact: true }).click(); await page.getByRole('log').getByText('Command received by fixture', { exact: true }).waitFor();
            await page.getByRole('textbox',{name:'Comando do servidor',exact:true}).press('ArrowUp'); assert.equal(await page.getByRole('textbox',{name:'Comando do servidor',exact:true}).inputValue(),'list'); await page.getByRole('textbox',{name:'Comando do servidor',exact:true}).press('ArrowDown'); assert.equal(await page.getByRole('textbox',{name:'Comando do servidor',exact:true}).inputValue(),''); await page.getByRole('button',{name:'Consultar jogadores',exact:true}).click(); await page.getByRole('button',{name:'Copiar console',exact:true}).waitFor(); assert.equal(await page.getByRole('log').evaluate(element=>getComputedStyle(element).userSelect),'text'); await page.screenshot({path:`${output}/console-${width}.png`});
            await page.getByRole('button', { name: 'Jogadores', exact: true }).click(); await page.getByText('Friend', { exact: true }).waitFor(); assert.equal(await page.locator('img[alt="Avatar de Friend"]').count(),1); assert.equal(await page.getByText(/2 entradas/).count(),0); await page.getByRole('button', { name: 'Banir', exact: true }).click();
            assert.equal(await page.evaluate(() => window.serverCalls.some(call => call.command === 'local_server_manage' && call.args.request.action === 'moderate')), false);
            await page.getByRole('button', { name: 'Confirmar', exact: true }).click(); await page.waitForFunction(() => window.serverCalls.some(call => call.command === 'local_server_manage' && call.args.request.action === 'moderate' && call.args.request.command === 'ban'));
            await page.getByRole('button', { name: 'Configurações', exact: true }).click(); const motd = page.getByLabel('Descrição no Multijogador (MOTD)', { exact: true }); await motd.waitFor(); await page.locator('[data-managed-skin-support]').waitFor(); assert.equal(await page.locator('[data-managed-skin-support]').getByText('Obrigatório',{exact:true}).count(),1); assert.equal(await page.locator('[data-server-settings-save]').evaluate(element=>getComputedStyle(element).position),'static'); await page.getByLabel('Compressão de rede (bytes)',{exact:true}).fill('512'); await page.getByLabel('Alcance das entidades (%)',{exact:true}).fill('75'); assert.equal(await motd.isDisabled(), false); await motd.fill('Alterado durante execução'); await page.getByRole('button', { name: 'Guardar configurações', exact: true }).click(); await page.waitForFunction(() => window.serverSettings.motd === 'Alterado durante execução'); assert.equal(await page.evaluate(()=>window.serverSettings.networkCompressionThreshold),512); assert.equal(await page.evaluate(()=>window.serverSettings.entityBroadcastRangePercentage),75); await page.locator('[data-server-settings-save]').scrollIntoViewIfNeeded(); await page.screenshot({path:`${output}/settings-bottom-${width}.png`});
            await page.getByRole('button', { name: 'Parar e salvar', exact: true }).click(); await page.getByText('Servidor parado', { exact: true }).waitFor(); await motd.fill('Servidor aberto'); await page.getByRole('button', { name: 'Guardar configurações', exact: true }).click(); await page.waitForFunction(() => window.serverSettings.motd === 'Servidor aberto');
            assert.equal(await page.getByRole('checkbox', { name: /Restringir acesso por whitelist/ }).isChecked(), false);
            assert.equal(await page.getByRole('button',{name:'Backups',exact:true}).count(),0);assert.equal(await page.getByRole('button',{name:'Alterar porta',exact:true}).count(),0);
            await page.getByLabel(/Distância de visão/).fill('12'); await page.getByRole('button',{name:'Guardar configurações',exact:true}).click(); await page.waitForFunction(()=>window.serverSettings.viewDistance===12);
            await page.getByRole('button',{name:'Usar alcance leve: visão 6 · simulação 4',exact:true}).click();await page.getByRole('button',{name:'Guardar configurações',exact:true}).click();await page.waitForFunction(()=>window.serverSettings.viewDistance===6&&window.serverSettings.simulationDistance===4&&window.serverSettings.economyMode===true);
            await page.getByRole('button', { name: 'Conteúdos', exact: true }).click();
            const fileAction=page.locator('[data-server-open-files]');const box=await fileAction.boundingBox();assert.ok(box.width>=80);
            await page.getByRole('heading',{name:'Mod de teste',exact:true}).waitFor();
            await page.getByRole('button',{name:'Desativar',exact:true}).click();await page.getByText('server-content.jar.disabled',{exact:true}).waitFor();
            await page.getByRole('button',{name:'Ativar',exact:true}).click();await page.getByText('server-content.jar',{exact:true}).waitFor();
            const iconRequests=await page.evaluate(()=>window.serverCalls.filter(call=>call.args?.request?.action==='content_details').length);await page.waitForTimeout(500);assert.ok((await page.evaluate(()=>window.serverCalls.filter(call=>call.args?.request?.action==='content_details').length))-iconRequests<3,'Cover requests must settle without an update loop');
            await page.screenshot({path:`${output}/mod-controls-${width}.png`});
            assert.equal(await page.getByRole('button', { name: 'Modpack', exact: true }).count(),0); await page.getByRole('button', { name: 'Datapacks', exact: true }).click(); await page.getByRole('button', { name: 'Importar arquivo', exact: true }).waitFor();
            assert.equal(await page.getByRole('button', {name:'Vincular minha conta Playit',exact:true}).count(),0);
            await page.getByRole('button',{name:'Arquivos',exact:true}).click(); await page.getByText('server.properties',{exact:true}).waitFor();
            await page.getByRole('button',{name:'Mundo',exact:true}).click(); await page.getByText('level.dat',{exact:true}).waitFor();
            await page.getByRole('button',{name:'Texturas',exact:true}).click();
            await page.getByRole('button',{name:'Importar ZIP e preparar',exact:true}).click();
            await page.getByText('Selecionado para distribuição',{exact:false}).waitFor();
            assert.equal(await page.evaluate(()=>window.serverSettings.resourcePackFile),'pack.zip');
            assert.equal(await page.evaluate(()=>window.serverCalls.some(call=>call.args?.request?.action==='use_resource_pack')),true);
            await page.screenshot({path:`${output}/resource-packs-${width}.png`});
            const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth + 2); assert.equal(overflow, false);
            await page.screenshot({ path: `${output}/contents-${width}.png` });
            await page.getByRole('button', { name: 'Visão geral', exact: true }).click(); await page.locator('[data-server-content]').evaluate(element => element.scrollTop = 0); await page.screenshot({ path: `${output}/overview-${width}.png` });
            await page.getByRole('button',{name:'Excluir servidor',exact:true}).click(); const deletion = page.getByRole('dialog',{name:'Excluir servidor',exact:true});
            await deletion.getByLabel('Digite o nome do servidor para confirmar',{exact:true}).fill('wrong'); assert.equal(await deletion.getByRole('button',{name:'Excluir servidor',exact:true}).isDisabled(),true);
            await deletion.getByLabel('Digite o nome do servidor para confirmar',{exact:true}).fill('Servidor da turma'); await deletion.getByRole('button',{name:'Excluir servidor',exact:true}).click();
            await deletion.waitFor({state:'hidden'}); assert.equal(await page.evaluate(()=>window.localServerConfigs.length),0);
            assert.equal(await page.getByRole('button',{name:'Restaurar servidor',exact:true}).count(),0);
            await create.click(); const catalog = page.getByRole('dialog',{name:'Criar servidor',exact:true});
            await catalog.getByLabel('Nome do servidor',{exact:true}).fill('Servidor do catálogo');
            await catalog.getByRole('button',{name:'Modpack',exact:true}).click();
            await page.waitForFunction(()=>window.serverCalls.some(call=>call.command==='mods_search'&&call.args.contentType==='modpack'));
            await catalog.getByRole('button',{name:/Pack de servidor/}).click();
            await catalog.getByRole('combobox',{name:/Minecraft/}).waitFor();
            await page.screenshot({path:`${output}/creator-catalog-${width}.png`});
            for (let next=0;next<3;next++) await catalog.getByRole('button',{name:'Continuar',exact:true}).click();
            await catalog.getByRole('checkbox',{name:/Li e aceito a EULA/}).check();
            await catalog.getByRole('button',{name:'Criar e preparar',exact:true}).click();
            await catalog.getByRole('alert').filter({hasText:'rede indisponível'}).waitFor();
            assert.equal(await page.evaluate(()=>window.localServerConfigs.length),1);
            await catalog.getByRole('button',{name:'Tentar novamente',exact:true}).click(); await catalog.waitFor({state:'hidden'}).catch(async cause => { console.error(await catalog.innerText()); console.error(await page.evaluate(()=>window.serverCalls.slice(-12))); await page.screenshot({path:`${output}/catalog-failure-${width}.png`}); throw cause; });
            assert.equal(await page.evaluate(()=>window.serverCalls.filter(call=>call.args?.request?.action==='install_catalog_pack').length),2); assert.equal(await page.getByRole('button',{name:'Retomar preparação',exact:true}).count(),0);
            assert.equal(await page.evaluate(()=>window.serverCalls.some(call=>call.args?.request?.action==='import_instance')),false);
            assert.equal(await page.evaluate(()=>window.serverCalls.filter(call=>call.command==='local_server_create').length),2);
            assert.deepEqual(errors, []); console.log(`PASS viewport ${width}`); await page.close(); console.log(`Closed viewport ${width}`);
        }
        console.log('PASS: independent creator, explicit EULA, 8 GiB limit, retry without duplicates, unified tabs, P2P distinction, console, moderation, settings, low resource options and permanent deletion. Browser fixtures only.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
