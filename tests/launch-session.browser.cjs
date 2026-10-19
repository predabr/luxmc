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
            window.testSession = { sessionId: 'old-session', profileId: 'old-profile', versionId: '26.3', startedAt:'2026-10-16T14:16:19Z', updatedAt:'2026-10-16T14:51:04Z', phase:'failed', stage:'failed', failedStage:'running', elapsedSeconds:2084, pid:0, error:'WARNING: sun.misc.Unsafe::putFloat will be removed in a future release', exitCode:-8 };
            window.cancelledSession = null;
            window.savedLogBatches = [];
            let rejectLaunch;
            const invoke = async (command, args) => {
                if (command === 'launch_log_open') return 42;
                if (command === 'launch_log_append_batch') { window.savedLogBatches.push(args); return null; }
                if (command === 'launch_game') {
                    window.testSession = { sessionId: args.request.sessionId, profileId: args.request.profileId,
                        versionId: args.request.versionId, phase: 'preparing', stage: 'checkingJava', pid: 0,
                        startedAt: new Date().toISOString(), updatedAt: new Date().toISOString(), elapsedSeconds: 0,
                        error: null, exitCode: null };
                    window.launcherDemo.emit('launch-session', window.testSession);
                    return new Promise((resolve, reject) => { rejectLaunch = reject; window.rejectLaunch = reject; window.resolveLaunch = resolve; });
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
        assert.equal(await page.locator('aside[aria-label="Inicialização do Minecraft"]').count(),0);
        await page.getByRole('button', { name: 'Jogar', exact: true }).first().click();
        const status = page.getByRole('complementary', { name: 'Inicialização do Minecraft' });
        await page.getByRole('button', { name: 'Cancelar abertura', exact: true }).waitFor();
        assert.equal(await status.count(), 0);
        assert.equal(await page.getByText('Abrindo seu Minecraft', { exact: true }).count(), 0);
        const first = await page.evaluate(() => window.testSession.sessionId);
        assert.match(first, /^[0-9a-f-]{36}$/);
        await page.evaluate(() => {
            for (let i = 0; i < 300; i++) window.launcherDemo.emit('launcher-log', 'modpack-fixture-' + i);
        });
        await page.waitForFunction(() => window.savedLogBatches.flatMap(batch => batch.entries).filter(entry => entry.message.startsWith('modpack-fixture-')).length === 300);
        const batches = await page.evaluate(() => window.savedLogBatches.filter(batch => batch.entries.some(entry => entry.message.startsWith('modpack-fixture-'))));
        assert.equal(batches.length, 3);
        assert.ok(batches.every(batch => batch.logId === 42 && batch.entries.length <= 128));
        await page.locator('aside nav a[href="/mods"]').click();
        await page.waitForURL(/\/mods/);
        await page.getByRole('button', { name: 'Cancelar abertura', exact: true }).click();
        await page.waitForFunction(() => window.cancelledSession !== null);
        assert.equal(await page.evaluate(() => window.cancelledSession), first);
        await page.locator('aside button[title="Início"]').first().click();
        await page.waitForURL(url => url.pathname === '/');
        assert.equal(await status.count(), 0);
        await page.getByRole('button', { name: 'Jogar', exact: true }).first().click();
        await page.waitForFunction(previous => window.testSession.sessionId !== previous, first);
        await page.evaluate(previous => window.launcherDemo.emit('game-exit', {
            sessionId: previous, pid: 1, versionId: '1.21.1', code: 1, success: false, errorMessage: 'Old session'
        }), first);
        assert.equal(await status.count(), 0);
        assert.equal(await page.getByRole('button', { name: 'Cancelar abertura', exact: true }).count(), 1);
        await page.evaluate(() => window.resolveLaunch({ pid: 321, sessionId: window.testSession.sessionId }));
        await page.waitForFunction(() => !document.querySelector('aside[aria-label="Inicialização do Minecraft"]'));
        await page.waitForFunction(() => !Array.from(document.querySelectorAll('button')).some(button => button.getAttribute('aria-label') === 'Cancelar abertura'));
        await page.evaluate(() => window.launcherDemo.emit('game-exit', { sessionId: window.testSession.sessionId, code: 0, success: true }));
        await page.getByRole('button', { name: 'Jogar', exact: true }).first().click();
        await page.getByRole('button', { name: 'Cancelar abertura', exact: true }).waitFor();
        await page.evaluate(() => window.rejectLaunch(new Error('Java incompatível para esta versão')));
        await status.getByText(/^(?:Error: )?Java incompatível para esta versão$/).waitFor();
        assert.equal(await status.getByRole('button', { name: 'Copiar diagnóstico' }).count(), 1);
        assert.deepEqual(errors, []);
        console.log(JSON.stringify({ navigationPreservesLaunch: true, cancellationScoped: true, durableDiagnostic: true, oldExitIgnored: true, errors }));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
