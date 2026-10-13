const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const errors = [];
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.launcherDemo.state.settings.launcherActionOnLaunch = 'keep_open';
            const original = window.electronAPI.invoke;
            window.testSession = null;
            window.cancelledSession = null;
            let rejectLaunch;
            const invoke = async (command, args) => {
                if (command === 'launch_game') {
                    window.testSession = { sessionId: args.request.sessionId, profileId: args.request.profileId,
                        versionId: args.request.versionId, phase: 'preparing', stage: 'checkingJava', pid: 0,
                        startedAt: new Date().toISOString(), updatedAt: new Date().toISOString(), elapsedSeconds: 0,
                        error: null, exitCode: null };
                    window.launcherDemo.emit('launch-session', window.testSession);
                    return new Promise((resolve, reject) => { rejectLaunch = reject; window.resolveLaunch = resolve; });
                }
                if (command === 'launch_session_status') return window.testSession;
                if (command === 'cancel_launch') {
                    if (args.sessionId !== window.testSession.sessionId) throw new Error('Wrong session');
                    window.cancelledSession = args.sessionId;
                    window.testSession.phase = 'cancelled';
                    window.launcherDemo.emit('launch-session', window.testSession);
                    rejectLaunch(new Error('Abertura cancelada. Os arquivos válidos foram preservados.'));
                    return true;
                }
                return original(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto('http://127.0.0.1:1420/');
        await page.getByRole('button', { name: 'Jogar', exact: true }).first().click();
        const status = page.getByRole('complementary', { name: 'Inicialização do Minecraft' });
        await status.getByText('Abrindo seu Minecraft').waitFor();
        const first = await page.evaluate(() => window.testSession.sessionId);
        assert.match(first, /^[0-9a-f-]{36}$/);
        await page.locator('aside nav a[href="/mods"]').click();
        await page.waitForURL(/\/mods/);
        await status.getByRole('button', { name: 'Cancelar abertura' }).click();
        await status.getByText(/Abertura cancelada/).waitFor();
        assert.equal(await page.evaluate(() => window.cancelledSession), first);
        await page.locator('aside button[title="Início"]').first().click();
        await page.waitForURL(url => url.pathname === '/');
        await status.getByRole('button', { name: 'Fechar diagnóstico' }).click();
        await page.getByRole('button', { name: 'Jogar', exact: true }).first().click();
        await page.waitForFunction(previous => window.testSession.sessionId !== previous, first);
        await page.evaluate(previous => window.launcherDemo.emit('game-exit', {
            sessionId: previous, pid: 1, versionId: '1.21.1', code: 1, success: false, errorMessage: 'Old session'
        }), first);
        assert.equal(await status.getByText('Abrindo seu Minecraft').count(), 1);
        await page.evaluate(() => window.resolveLaunch({ pid: 321, sessionId: window.testSession.sessionId }));
        await page.waitForFunction(() => !document.querySelector('aside[aria-label="Inicialização do Minecraft"]'));
        assert.deepEqual(errors, []);
        console.log(JSON.stringify({ navigationPreservesLaunch: true, cancellationScoped: true, durableDiagnostic: true, oldExitIgnored: true, errors }));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
