const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1500, height: 950 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const invoke = window.electronAPI.invoke;
            window.presenceCalls = [];
            window.failPresenceSync = false;
            window.presenceSession = null;
            window.lastPresenceSnapshot = null;
            window.electronAPI.invoke = async (command, args = {}) => {
                window.presenceCalls.push({ command, args });
                if (command === 'social_presence_start') { window.presenceSession = args.sessionId; return true; }
                if (command === 'social_presence_stop') {
                    if (window.presenceSession === args.sessionId) window.presenceSession = null;
                    return null;
                }
                if (command === 'social_request' && args.request.action === 'sync') {
                    if (window.failPresenceSync) throw Error('Temporary fixture connection failure');
                    if (JSON.stringify(args.request).length > 4096) throw Error('Presence payload too large');
                    const result = await invoke(command, args);
                    window.lastPresenceSnapshot = result;
                    return result;
                }
                return invoke(command, args);
            };
        });
        await page.goto('http://127.0.0.1:1420/friends');
        await page.waitForFunction(() => window.lastPresenceSnapshot && window.presenceSession);
        await page.evaluate(async () => { window.presenceStore = (await import('/src/lib/stores/friends.svelte.ts')).friendsState; });
        await page.waitForFunction(() => window.presenceStore.connected && !window.presenceStore.busy && window.presenceStore.list.length > 0);
        const initial = await page.evaluate(async () => {
            const { friendsState } = await import('/src/lib/stores/friends.svelte.ts');
            return { list: friendsState.list, connected: friendsState.connected, session: window.presenceSession, error: friendsState.error, busy: friendsState.busy, calls: window.presenceCalls.slice(-12).map(call => ({ command: call.command, action: call.args.request?.action, status: call.args.request?.status })) };
        });
        assert.ok(initial.connected, JSON.stringify({ error: initial.error, busy: initial.busy, calls: initial.calls }));
        assert.ok(initial.list.some(friend => friend.status === 'online' || friend.status === 'in_game'));
        await page.locator('.friends-directory').waitFor();
        await page.evaluate(async () => {
            window.failPresenceSync = true;
            await (await import('/src/lib/stores/friends.svelte.ts')).friendsState.refresh();
        });
        const failed = await page.evaluate(async () => {
            const { friendsState } = await import('/src/lib/stores/friends.svelte.ts');
            return { list: friendsState.list, connected: friendsState.connected, error: friendsState.error };
        });
        assert.deepEqual(failed.list.map(friend => friend.status), initial.list.map(friend => friend.status));
        assert.equal(failed.connected, false);
        assert.ok(failed.error.includes('Temporary fixture'));
        await page.locator('.friends-directory').waitFor();
        await page.getByText('Reconectando', { exact: true }).first().waitFor();
        await page.evaluate(() => {
            Object.defineProperty(document, 'hidden', { configurable: true, value: true });
            window.launcherDemo.emit('social-presence', { sessionId: window.presenceSession, snapshot: window.lastPresenceSnapshot });
        });
        await page.waitForFunction(() => window.presenceStore.connected);
        await page.evaluate(() => window.launcherDemo.emit('social-presence', { sessionId: window.presenceSession, snapshot: { me: { id: 'invalid' }, friends: [{ status: 'bad' }] } }));
        assert.equal(await page.evaluate(async () => (await import('/src/lib/stores/friends.svelte.ts')).friendsState.connected), true);
        await page.evaluate(async () => {
            window.failPresenceSync = false;
            Object.defineProperty(document, 'hidden', { configurable: true, value: false });
            const { account } = await import('/src/lib/stores/account.svelte.ts');
            account.value = { ...account.value, id: 'local-fixture', uuid: 'local-fixture', username: 'Local_Player', minecraftToken: '' };
        });
        await page.waitForFunction(() => window.presenceSession && window.presenceSession !== window.presenceCalls.find(call => call.command === 'social_presence_start').args.sessionId);
        await page.waitForFunction(() => window.presenceCalls.some(call => call.command === 'social_request' && call.args.presenceSession === window.presenceSession && call.args.request.action === 'sync'));
        await page.locator('.launcher-sidebar').getByRole('button', { name: 'Início', exact: true }).click();
        await page.waitForURL('http://127.0.0.1:1420/');
        await page.getByText('Conta local', { exact: true }).first().waitFor();
        await page.getByText('Online no Luxmc', { exact: true }).first().waitFor();
        assert.equal(await page.getByText('Conta Offline', { exact: true }).count(), 0);
        const currentSession = await page.evaluate(() => window.presenceSession);
        await page.evaluate(async () => {
            const { account } = await import('/src/lib/stores/account.svelte.ts');
            account.value = { ...account.value, avatarUrl: '/grass_head.png' };
        });
        await page.waitForTimeout(150);
        assert.equal(await page.evaluate(() => window.presenceSession), currentSession);
        assert.ok(await page.evaluate(session => window.presenceCalls.some(call => call.command === 'social_presence_stop' && call.args.sessionId === session), initial.session));
        await page.evaluate(session => window.launcherDemo.emit('social-presence', { sessionId: session, snapshot: { me: { id: 'obsolete', username: 'Previous_Account' }, friends: [] } }), initial.session);
        assert.notEqual(await page.evaluate(async () => (await import('/src/lib/stores/friends.svelte.ts')).friendsState.me?.id), 'obsolete');
        await page.evaluate(async () => {
            const { activeSkinStore } = await import('/src/lib/stores/skin.svelte.ts');
            activeSkinStore.setSkin({ avatarUrl: 'data:image/png;base64,' + 'A'.repeat(9000) });
            await (await import('/src/lib/stores/friends.svelte.ts')).friendsState.refresh();
        });
        assert.equal(await page.evaluate(async () => (await import('/src/lib/stores/friends.svelte.ts')).friendsState.connected), true);
        assert.equal(await page.evaluate(() => window.presenceCalls.filter(call => call.command === 'social_request' && call.args.request.action === 'sync').at(-1).args.request.avatarUrl), undefined);
        await page.evaluate(async () => {
            const { appState } = await import('/src/lib/stores/app.svelte.ts');
            appState.isGameRunning = true;
            appState.activeGameDetails = { profileId: 'fixture', name: 'Presence World', version: '1.20.1', loader: 'fabric' };
        });
        await page.waitForFunction(() => window.presenceCalls.some(call => call.command === 'social_request' && call.args.request.status === 'in_game' && call.args.request.instanceName === 'Presence World'));
        const activeSession = await page.evaluate(() => window.presenceSession);
        await page.evaluate(async () => (await import('/src/lib/stores/friends.svelte.ts')).friendsState.disconnect());
        await page.waitForFunction(() => window.presenceSession === null);
        await page.evaluate(session => window.launcherDemo.emit('social-presence', { sessionId: session, snapshot: window.lastPresenceSnapshot }), activeSession);
        assert.equal(await page.evaluate(async () => (await import('/src/lib/stores/friends.svelte.ts')).friendsState.me), null);
        assert.equal(await page.evaluate(() => window.presenceStore.connectionStatus), 'disconnected');
        assert.deepEqual(errors, []);
        console.log('Presence: temporary failure preserves friend status and list; native snapshots recover hidden UI; local account appears online; stale sessions are ignored; large avatars cannot break presence; game activity updates immediately; disconnect stops the worker.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
