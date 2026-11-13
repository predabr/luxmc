const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/local-server-reform';
    fs.mkdirSync(output, { recursive: true });
    try {
        const width = 1440, height = 900;
        console.log(`Checking ${width}x${height}`);
        const page = await browser.newPage({ viewport: { width, height }, reducedMotion: 'reduce' });
        page.setDefaultTimeout(15000); page.setDefaultNavigationTimeout(30000);
        const errors = [];
        page.on('pageerror', (error) => console.log('PAGEERROR', error.message.slice(0, 160)));
        page.on('console', (m) => { if (m.text().includes('DBG')) console.log('PAGE', m.text()); });
        page.on('crash', () => console.log('PAGE CRASH')); page.on('pageerror', error => { errors.push(error.message); console.error('Browser runtime:', error.stack); });
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.serverCalls = []; window.publicEnabled = false; window.localServerConfigs = []; window.localServerActive = null; window.localServerRoom = null; window.serverBackups = []; window.deletedServers = []; window.serverContents = [{ kind: 'mod', name: 'server-content.jar', bytes: 4096, staged: false, remove: false }];
            window.serverSettings = { motd: 'Meu servidor', difficulty: 'normal', gamemode: 'survival', pvp: true, whitelist: false, viewDistance: 8, simulationDistance: 6, port: 0, autosaveMinutes: 5, resourcePackUrl: '', resourcePackSha1: '', requireResourcePack: false, resourcePackFile: '', spawnProtection: 16, allowFlight: false, commandBlocks: false, idleTimeout: 0 };
            const original = window.electronAPI.invoke;
            const invoke = async (command, args = {}) => {
                window.serverCalls.push({ command, args });
                if (command === 'membership_request' && args.request?.action === 'preferences') return { ok: true, revision: 2 };
                if (command === 'membership_request') return { identity: { id: '00000000-0000-4000-8000-000000000001', username: 'LuxPlayer', owner: true }, membership: { plan: 'free', expiresAt: 0, revision: 1, collections: 0, presets: 0 }, settings: { preferences: {}, presets: [], revision: 1 } };
                if (command === 'tunnel_status') return null;
                if (command === 'stop_session') { window.localServerRoom = null; return null; }
                if (command === 'profiles_list') return [];
                if (command === 'versions_list') return { versions: [{ id: '1.21.1', versionType: 'release' }], latestRelease: '1.21.1', latestSnapshot: '26w01a' };
                if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64', launcherVersion: '4.0.1', gpuVendor: 'test', gpuRenderer: 'test', gpuSupportsZink: false, kernelVersion: 'test' };
                if (command === 'java_scan') return { runtimes: [{ major: 21, installed: true, path: 'C:\\java\\21', versionString: '21.0.3', isSystem: false }] };
                if (command === 'system_resource_sample') return { cpuPercent: 12, usedMemoryMb: 4096, totalMemoryMb: 8192, sampleMilliseconds: 100 };
                if (command === 'network_throughput_sample') return { receivedBytesPerSecond: 1024, transmittedBytesPerSecond: 512, sampleMilliseconds: 100 };
                if (command === 'local_server_list') {
                    const state = window.localServerActive ? { ...structuredClone(window.localServerActive), console: window.localServerActive.console.filter(line => line.sequence > (args.after || 0)) } : null;
                    return { servers: structuredClone(window.localServerConfigs), active: state, states: state ? [structuredClone(state)] : [], warnings: [] };
                }
                if (command === 'local_server_telemetry') {
                    const active = window.localServerActive;
                    const online = !!active && active.phase === 'running';
                    return {
                        serverId: args.id, phase: online ? 'running' : 'stopped', online,
                        pid: online ? active.pid : null, uptimeSeconds: online ? 120 : null, port: online ? active.port : null,
                        configuredRamMb: 2048, playersOnline: online ? active.playersOnline : 0, maxPlayers: active?.maxPlayers ?? 20,
                        processMemoryMb: online ? 900 : null, processCpuPercent: online ? 12.5 : null,
                        heapUsedMb: online ? 512 : null, heapCommittedMb: online ? 1024 : null, heapMaxMb: 2048, heapSource: online ? 'jcmd' : null,
                        tps: null, mspt: null,
                        systemCpuPercent: 12, systemUsedMemoryMb: 4096, systemTotalMemoryMb: 8192, systemAvailableMemoryMb: 4096,
                        detail: online ? 'Medições reais do processo Java (PID 123) via jcmd local; heap máximo = RAM configurada.' : 'Servidor offline — sem métricas de processo.'
                    };
                }
                if (command === 'local_server_create') { const server = { ...args.input, id: crypto.randomUUID() }; window.localServerConfigs.push(server); return structuredClone(server); }
                if (command === 'local_server_update') { const server = window.localServerConfigs.find(item => item.id === args.id); Object.assign(server, { name: args.name, ramMb: args.ramMb, maxPlayers: args.maxPlayers }); return structuredClone(server); }
                if (command === 'local_server_start') { window.localServerActive = { id: args.id, generation: crypto.randomUUID(), phase: 'running', port: 34567, pid: 123, maxPlayers: 20, playersOnline: 1, error: null, exitCode: null, console: [{ sequence: 1, stream: 'stdout', text: 'Done (0.1s)! Ready' }], cursor: 1, lastSavedAt: null, forcedStop: false, historyComplete: true, players: [{ uuid: '00000000-0000-4000-8000-000000000002', name: 'Friend', firstJoinedAt: new Date().toISOString(), lastJoinedAt: new Date().toISOString(), lastLeftAt: null, visits: 2, online: true }] }; return structuredClone(window.localServerActive); }
                if (command === 'local_server_stop') { window.localServerActive.phase = 'stopped'; window.localServerActive.pid = null; window.localServerActive.playersOnline = 0; window.localServerActive.players.forEach(player => player.online = false); return null; }
                if (command === 'local_server_console') { const server = window.localServerActive; server.cursor++; server.console.push({ sequence: server.cursor, stream: 'stdout', text: 'Command received by fixture' }); return null; }
                if (command === 'local_server_open_folder') return null;
                if (command === 'local_server_manage') {
                    const request = args.request;
                    if (request.action === 'connection_diagnostic') {
                        const running = window.localServerActive?.phase === 'running';
                        const labels = [
                            ['installation', 'Integridade da instalação', 'approved', 'server.jar verificado: o SHA-1 confere com o pacote oficial.'],
                            ['java', 'Runtime Java', 'approved', 'Java 21 disponível: OpenJDK Runtime Environment'],
                            ['process', 'Processo do servidor', running ? 'approved' : 'unverified', running ? 'Processo Java em execução (PID 123), fase running.' : 'Servidor parado — nenhum processo em execução para verificar.'],
                            ['startup', 'Conclusão da inicialização', running ? 'approved' : 'unverified', running ? 'Inicialização concluída: o servidor registrou o sinal de prontidão e responde ao ping de status.' : 'Servidor parado — a inicialização não foi verificada.'],
                            ['port', 'Disponibilidade da porta local', 'approved', 'A porta 34567 está em uso pelo próprio processo do servidor.'],
                            ['listening', 'Socket de escuta real', running ? 'approved' : 'unverified', running ? 'Socket de escuta ativo em 0.0.0.0:34567.' : 'Servidor offline — sem socket de escuta para verificar.'],
                            ['localhost', 'Conexão local (127.0.0.1) e Server List Ping', running ? 'approved' : 'unverified', running ? 'Server List Ping válida em 127.0.0.1:34567 — 1.21.1 (1 / 20 jogadores, 3 ms).' : 'Servidor offline — o teste local não foi executado.'],
                            ['lan', 'Endereços da rede local', 'approved', 'Use estes endereços em outros aparelhos da mesma rede: 192.168.1.50:34567.'],
                            ['firewall', 'Firewall do Windows', 'attention', 'Nenhuma regra de entrada conhecida para a porta 34567 ou para o Java. O Luxmc não altera o firewall automaticamente.'],
                            ['public', 'Endereço público', 'approved', 'Endereço público deste computador: 203.0.113.10 (visível apenas nesta tela).'],
                            ['router', 'Encaminhamento no roteador', 'unverified', 'Para jogadores de outra rede, encaminhe TCP 34567 no roteador até 192.168.1.50. Nada disso foi verificado.'],
                            ['external', 'Conectividade externa', 'unverified', 'Requer uma rede externa. O Luxmc não afirma conectividade externa sem esse teste.']
                        ];
                        const badge = { approved: 'Aprovado', attention: 'Atenção', failed: 'Falhou', unverified: 'Não verificado' };
                        return {
                            minecraftResponding: running,
                            address: running ? '127.0.0.1:34567' : null,
                            phase: window.localServerActive?.phase || 'stopped',
                            tcpReachable: running,
                            port: 34567,
                            publicAddress: '203.0.113.10',
                            lanAddresses: ['192.168.1.50:34567'],
                            generatedAt: new Date().toISOString(),
                            ping: running ? { online: true, version: '1.21.1', playersOnline: 1, playersMax: 20, motd: 'Meu servidor', latencyMs: 3 } : null,
                            checks: labels.map(([id, label, status, detail]) => ({ id, label, status, statusLabel: badge[status], detail }))
                        };
                    }
                    if (request.action === 'console_text') return { text: (window.localServerActive?.console || []).map(line => line.text).join('\n'), truncated: false };
                    if (request.action === 'deletion_preview') return { files: 47, bytes: 1048576 };
                    if (request.action === 'settings') return structuredClone(window.serverSettings);
                    if (request.action === 'save_settings') { window.serverSettings = request.settings; return structuredClone(request.settings); }
                    if (request.action === 'loader') return { kind: 'vanilla', version: '' };
                    if (request.action === 'public_access') { if (request.operation === 'enable') window.publicEnabled = true; if (request.operation === 'disable') window.publicEnabled = false; return { enabled: window.publicEnabled === true, configured: true, whitelist: false, port: 34567, externalIp: window.publicEnabled ? '203.0.113.9' : null, address: window.publicEnabled ? 'oi.joinmc.link' : null, likelyBehindCgnat: false, renewalError: null }; }
                    if (request.action === 'players') return { players: window.localServerActive?.players || [], banned: [], whitelist: [], operators: [{ uuid: '00000000-0000-4000-8000-000000000001', name: 'LuxPlayer', level: 4 }] };
                    if (request.action === 'moderate') { window.serverCalls.push({ command: 'moderate_done', args: request }); return null; }
                    if (request.action === 'contents') return structuredClone(window.serverContents);
                    if (request.action === 'backups') return structuredClone(window.serverBackups);
                    if (request.action === 'backup') { const backup = { id: crypto.randomUUID(), createdAt: new Date().toISOString(), bytes: 1000, files: 2 }; window.serverBackups.push(backup); return backup; }
                    if (request.action === 'restore_backup') { const index = window.serverBackups.findIndex(item => item.id === request.backup); if (index >= 0) window.serverBackups.splice(index, 1); return null; }
                    if (request.action === 'remove_backup') { const index = window.serverBackups.findIndex(item => item.id === request.backup); if (index >= 0) window.serverBackups.splice(index, 1); return null; }
                    if (request.action === 'delete_server') { const item = window.localServerConfigs.find(item => item.id === request.id); if (item.name !== request.name) throw new Error('Confirm name'); if (request.deleteBackups !== true) throw new Error('Delete must include owned backups'); window.localServerConfigs = window.localServerConfigs.filter(item => item.id !== request.id); return null; }
                    return null;
                }
                return original(command, args);
            };
            window.electronAPI.invoke = invoke; window.__TAURI_INTERNALS__.invoke = invoke;
        });

        const focusNode = async (node) => {
            await node.evaluate((n) => n.scrollIntoView({ block: 'center', inline: 'center' }));
            await page.waitForTimeout(120);
        };
        const openNode = async (node) => {
            await focusNode(node);
            if (await node.locator('.node-x').count() === 0) return;
            if (await node.evaluate((n) => n.className.includes('open'))) return;
            await node.locator('.node-x').click();
            await node.evaluate((n) => n.className.includes('open')).then(async (open) => { if (!open) await node.locator('.node-x').click(); });
        };

        await page.goto('http://127.0.0.1:1420/hosting/server');

        // Empty canvas with a single real action.
        await page.getByText('f(x) → servidor', { exact: true }).waitFor({ timeout: 45000 });
        assert.equal(await page.locator('.node').count(), 0);
        assert.equal(await page.getByText('LUXMC CORE', { exact: false }).count(), 0);

        // Wizard opens; EULA starts unchecked and creation stays disabled.
        await page.getByRole('button', { name: 'Criar servidor', exact: true }).first().click();
        const form = page.locator('dialog.wizard-dialog');
        await form.getByLabel('Nome do servidor', { exact: true }).fill('Servidor da turma');

		// Step 1 → 2: identity.
        await form.getByRole('button', { name: 'Continuar', exact: true }).click();
        // Step 2 → 3: software (Vanilla preselected).
        await form.getByRole('button', { name: 'Continuar', exact: true }).click();
        // Step 3 → 4: Java compatibility is displayed, then continue.
        await form.getByText(/requer/).waitFor();
        await form.getByRole('button', { name: 'Continuar', exact: true }).click();
        // Step 4: world configuration.
        await form.getByLabel(/Semente do mundo/).fill('luxmc-world');
        await form.getByRole('combobox', { name: /Modo de jogo/ }).click();
        await form.getByRole('option', { name: 'Criativo', exact: true }).click();
        await form.getByRole('button', { name: 'Continuar', exact: true }).click();
        // Step 5: game rules catalog, summary, EULA gate.
        await form.getByLabel('Buscar regra do jogo').fill('keepInventory');
        await form.getByText('keepInventory', { exact: true }).first().waitFor();
        await form.getByRole('button', { name: 'Criar servidor', exact: true }).waitFor();
        assert.equal(await form.getByRole('button', { name: 'Criar servidor', exact: true }).isDisabled(), true);
        const eula = form.getByRole('checkbox', { name: /Li e concordo com o contrato/ });
        await eula.waitFor();
        assert.equal(await eula.isChecked(), false);
        await form.getByLabel('Buscar regra do jogo').fill('');
        await eula.check();
        await form.getByRole('button', { name: 'Criar servidor', exact: true }).click();

        // Real installation timeline finishing, then the dashboard appears.
        await form.getByText('Instalação concluída', { exact: true }).waitFor({ timeout: 20000 }).catch(async cause => {
            console.error('INSTALL BODY:', (await form.innerText().catch(() => '(dialog gone)')).slice(-1200));
            console.error('CALLS:', await page.evaluate(() => window.serverCalls.map(call => call.command).join(',')));
            await page.screenshot({ path: output + '/install-failure.png' });
            throw cause;
        });
        assert.equal(await page.evaluate(() => window.localServerConfigs[0].eulaAccepted), true);
        assert.equal(await page.evaluate(() => window.serverSettings.seed), 'luxmc-world');
        assert.equal(await page.evaluate(() => window.serverSettings.gamemode), 'creative');
        await form.waitFor({ state: 'hidden' });
        const hub = page.locator('.node[aria-label="Módulo Servidor"]');
        await hub.getByRole('heading', { name: 'Servidor da turma' }).waitFor({ timeout: 20000 });
        await page.getByText('Online', { exact: true }).first().waitFor();
        assert.equal(await page.locator('.wire').count() > 0, true);
        await page.screenshot({ path: `${output}/overview.png` });

        // Rede & Conexão: endereços copiáveis, online-mode e 12 verificações reais.
        const net = page.locator('.node[aria-label="Módulo Rede & Conexão"]');
        await openNode(net);
        await net.getByText('127.0.0.1:34567', { exact: true }).waitFor({ timeout: 20000 });
        await net.getByText('192.168.1.50:34567', { exact: true }).waitFor();
        assert.equal(await net.locator('.copy').count(), 3);
        await net.getByRole('button', { name: 'Testar conexão', exact: true }).click();
        await net.getByRole('button', { name: /Ver as 12 verificações/ }).click();
        assert.equal(await net.locator('.check').count(), 12);
        assert.ok((await net.locator('.check.ok').count()) > 0);
        await page.screenshot({ path: `${output}/diagnostic.png` });

        // Online-mode alterna para offline e grava em server.properties.
        await net.getByRole('button', { name: /Pirata \/ Offline/ }).click();
        await page.waitForFunction(() => window.serverSettings.onlineMode === false);
        await net.getByText(/Qualquer pessoa com o endereço entra/).waitFor();
        await net.getByRole('button', { name: /Originais/ }).click();
        await page.waitForFunction(() => window.serverSettings.onlineMode === true);

        // UPnP liga e desliga com estado honesto.
        await page.evaluate(() => { window.__clicks = []; document.addEventListener('click', (e) => window.__clicks.push('click ' + e.clientX + ',' + e.clientY + ' ' + (e.target?.tagName) + '.' + String(e.target?.className).slice(0, 18)), true); document.addEventListener('pointerdown', (e) => window.__clicks.push('down ' + e.clientX + ',' + e.clientY + ' ' + (e.target?.tagName) + '.' + String(e.target?.className).slice(0, 18)), true); });
        console.log('GEO', await net.evaluate((n) => { const t = n.querySelector('.switch .track'); const r = t.getBoundingClientRect(); const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); return JSON.stringify({ y: Math.round(r.y), h: Math.round(r.height), hit: hit?.className?.toString?.().slice(0, 30), nodeOpen: n.className.includes('open'), scroll: document.querySelector('main')?.scrollTop }); }));
        await net.locator('.switch .track').click();
        await page.waitForFunction(() => window.serverCalls.some(c => c.args?.request?.action === 'public_access' && c.args.request.operation === 'enable'));
        await openNode(net);
        console.log('CLICKS', await page.evaluate(() => window.__clicks.join(' | ')));
        await net.getByText(/redirecionada no roteador/).waitFor({ state: 'attached' });
        await net.locator('.addr').filter({ hasText: 'oi.joinmc.link' }).waitFor({ state: 'attached', timeout: 15000 }).catch(async (cause) => {
            console.error('NET BODY:', (await net.innerText()).slice(0, 900)); console.error('DETAIL:', await net.evaluate((n) => JSON.stringify({ bodyText: n.querySelector('.body-node')?.innerText?.slice(0, 200), bodyDisplay: getComputedStyle(n.querySelector('.body-node') || document.body).display, bodyBox: n.querySelector('.body-node') ? n.querySelector('.body-node').getBoundingClientRect().height : -1, ops: [...new Set(window.serverCalls.filter((c) => c.args?.request?.action === 'public_access').map((c) => c.args.request.operation))] })));
            console.error('PUBLIC FLAG:', await page.evaluate(() => JSON.stringify(window.publicEnabled)));
            throw cause;
        });
        await net.locator('.switch .track').click();
        await page.waitForFunction(() => window.serverCalls.some(c => c.args?.request?.action === 'public_access' && c.args.request.operation === 'disable'));
        await openNode(net);
        await page.screenshot({ path: `${output}/network.png` });

        // Console: saída real do processo e ida-e-volta de comando.
        const consoleNode = page.locator('.node[aria-label="Módulo Console"]');
        await openNode(consoleNode);
        await consoleNode.getByText('Done (0.1s)! Ready', { exact: true }).waitFor();
        await consoleNode.getByPlaceholder('Comando (ex: list)').fill('list');
        await consoleNode.getByRole('button', { name: 'Enviar comando', exact: true }).click();
        await consoleNode.getByText('Command received by fixture', { exact: true }).waitFor();
        await page.screenshot({ path: `${output}/console.png` });

        // Jogadores: registros reais do processo.
        const playersNode = page.locator('.node[aria-label="Módulo Jogadores"]');
        await openNode(playersNode);
        await playersNode.getByText('Friend', { exact: true }).waitFor();
        await playersNode.getByText(/1\/\d+ online/).waitFor();

        // Mundo: catálogo de regras do mundo.
        const worldNode = page.locator('.node[aria-label="Módulo Mundo"]');
        await openNode(worldNode);
        await worldNode.getByText('Regras do jogo', { exact: true }).waitFor({ timeout: 20000 });
        await worldNode.getByText('Manter inventário ao morrer', { exact: true }).waitFor();
        await page.screenshot({ path: `${output}/world.png` });

        // Cabos saem de vários lados do hub (multi-âncora).
        const sides = await page.evaluate(() => {
            const pins = [...document.querySelectorAll('.node[aria-label="Módulo Servidor"] .port')].map(pin => pin.className.match(/pin-(\w+)/)?.[1]);
            return new Set(pins).size;
        });
        assert.ok(sides >= 3, 'cabos devem sair de pelo menos 3 lados');

        // Backups: criar e listar cópias reais.
        const backupsNode = page.locator('.node[aria-label="Módulo Backups"]');
        await openNode(backupsNode);
        await backupsNode.getByRole('button', { name: 'Criar backup', exact: true }).click();
        await backupsNode.getByText(/2 arquivos/).waitFor();

        // Configuração: nome espelhado no hub e MOTD persistido no fixture.
        const configNode = page.locator('.node[aria-label="Módulo Configuração"]');
        await openNode(configNode);
        const motd = configNode.getByLabel('MOTD');
        await motd.waitFor();
        const nameInput = configNode.getByLabel('Nome do servidor');
        await nameInput.fill('Servidor renomeado');
        await nameInput.blur();
        await page.waitForFunction(() => window.localServerConfigs[0]?.name === 'Servidor renomeado');
        await hub.getByRole('heading', { name: 'Servidor renomeado' }).waitFor();
        await motd.fill('Alterado durante execução');
        await configNode.getByRole('button', { name: 'Salvar configuração', exact: true }).click();
        await page.waitForFunction(() => window.serverSettings.motd === 'Alterado durante execução');

        // Recursos: slider de RAM com limites reais.
        const resNode = page.locator('.node[aria-label="Módulo Recursos"]');
        await openNode(resNode);
        const ram = resNode.getByRole('slider');
        await ram.waitFor();
        assert.ok(Number(await ram.getAttribute('max')) > 0);
        await page.screenshot({ path: `${output}/performance.png` });

        // Exclusão digitada; volta ao canvas vazio.
        await configNode.getByRole('button', { name: 'Excluir servidor', exact: true }).click();
        const delInput = configNode.locator('.del-input');
        await delInput.fill('errado');
        assert.equal(await configNode.getByRole('button', { name: 'Excluir', exact: true }).isDisabled(), true);
        await delInput.fill('Servidor renomeado');
        await configNode.getByRole('button', { name: 'Excluir', exact: true }).click();
        await page.waitForFunction(() => window.localServerConfigs.length === 0);
        await page.getByText('f(x) → servidor', { exact: true }).waitFor();
        await page.screenshot({ path: `${output}/empty-state.png` });

        const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth + 2);
        assert.equal(overflow, false);
        assert.deepEqual(errors, []);
        console.log('PASS: empty canvas, 5-step wizard, unchecked EULA, real install timeline, multi-side cables, copyable addresses, online-mode toggle, UPnP state, 12 diagnostics, console round-trip, players, gamerules, backups, mirrored rename, RAM limits, typed deletion. Browser fixtures only.');
        await page.close();
    } finally {
        await browser.close();
    }
})().catch(async error => {
    console.error(error);
    process.exitCode = 1;
});
