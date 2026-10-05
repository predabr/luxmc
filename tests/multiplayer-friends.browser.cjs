const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.addInitScript(() => {
            const me = { id: 'cccccccc-cccc-4ccc-8ccc-cccccccccccc', username: 'Steve' };
            const matches = [{ id: 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', username: 'Alex' }, { id: 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', username: 'Alex' }];
            window.calls = [];
            window.friendRecords = [];
            const invoke = async (command, args) => {
                window.calls.push({ command, args });
                if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'steve', username: 'Steve', minecraftToken: '' }, profiles: [], activeProfileId: null, devMode: true };
                if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                if (command === 'plugin:store|get') return [{ language: 'pt-BR', animations: false, liveWallpaper: false }, true];
                if (['deep_links_take', 'profiles_list', 'instances_list', 'changelog_get'].includes(command)) return [];
                if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64', launcherVersion: '2.0.2' };
                if (command === 'p2p_scan_lan_worlds') return [];
                if (command === 'social_request') {
                    const request = args.request;
                    if (request.action === 'register') return { me };
                    if (request.action === 'stream_ticket') return { url: null };
                    if (request.action === 'search') return { users: request.query.includes('#') ? matches.filter(item => item.id.startsWith(request.query.split('#')[1])) : matches };
                    if (request.action === 'invite') {
                        window.friendRecords = [{ ...matches.find(item => item.id === request.targetId), status: 'pending', incoming: false, lastSeen: null, activity: null, mcVersion: null, loader: null, serverIp: null, serverPort: null }];
                        return { ok: true };
                    }
                    if (request.action === 'accept') { window.friendRecords[0].status = 'online'; window.friendRecords[0].incoming = false; return { ok: true }; }
                    return { me, friends: structuredClone(window.friendRecords) };
                }
                return null;
            };
            window.electronAPI = { invoke, on: () => () => {} };
            window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
            window.__TAURI_INTERNALS__ = { metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } }, invoke, transformCallback: () => 1, convertFileSrc: path => path };
        });
        await page.route('https://**/*', route => route.abort());
        await page.goto('http://127.0.0.1:1420/friends?tab=add');
        const input = page.getByPlaceholder('Nickname ou Nickname#código');
        await input.fill('Alex');
        await page.getByRole('button', { name: 'Alex#aaaaaaaa' }).waitFor();
        await page.getByRole('button', { name: 'Adicionar Amigo', exact: true }).click();
        await page.getByText('Há mais de um perfil com esse nickname.', { exact: false }).waitFor();
        assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'social_request' && call.args.request.action === 'invite').length), 0);
        await input.fill('Alex#bbbbbbbb');
        await page.getByRole('button', { name: 'Adicionar Amigo', exact: true }).click();
        await page.getByRole('heading', { name: 'Solicitações de Amizade' }).waitFor();
        assert.equal(await page.evaluate(() => window.calls.find(call => call.command === 'social_request' && call.args.request.action === 'invite').args.request.targetId), 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb');
        await page.getByText('Aguardando resposta', { exact: true }).waitFor();
        await page.evaluate(() => { window.friendRecords[0].incoming = true; });
        await page.getByRole('button', { name: 'Aceitar Convite', exact: true }).waitFor({ timeout: 12000 });
        assert.ok(await page.getByText('Alex enviou uma solicitação de amizade.', { exact: true }).count());
        await page.getByRole('button', { name: 'Aceitar Convite', exact: true }).click();
        await page.getByText('Nenhum convite pendente', { exact: true }).waitFor();
        assert.equal(await page.evaluate(() => window.calls.filter(call => call.command === 'social_request' && call.args.request.action === 'accept').length), 1);
        assert.deepEqual(errors, []);
        console.log('Friend requests: duplicate nickname protected, exact code selected, outgoing request visible, incoming fallback refresh and notification, acceptance passed');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
