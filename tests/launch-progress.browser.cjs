const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1100, height: 900 }, reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.launcherDemo.state.settings.launcherActionOnLaunch = 'keep_open';
            const original = window.electronAPI.invoke;
            const invoke = (command, args) => command === 'launch_game'
                ? new Promise((resolve, reject) => { window.pendingLaunch = { resolve, reject }; })
                : original(command, args);
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto('http://127.0.0.1:1420/instances/demo-vanilla');
        await page.getByRole('button', { name: 'JOGAR MINECRAFT', exact: true }).click();
        await page.waitForFunction(() => window.pendingLaunch);
        await page.evaluate(() => window.launcherDemo.emit('launch-stage', 'waitingForWindow'));
        const panel = page.locator('.launch-progress');
        await panel.getByText('Aguardando a janela do Minecraft...').waitFor();
        assert.equal(await panel.getByText('85%', { exact: true }).count(), 0);
        assert.equal(await panel.locator('.complete').count(), 2);
        assert.equal(await panel.locator('.active').count(), 1);
        assert.equal(await panel.locator('.phase-fill').last().evaluate(element => getComputedStyle(element).animationName), 'none');
        if (process.env.LUXMC_PROGRESS_SCREENSHOT) await page.locator('.instance-hero').screenshot({ path: process.env.LUXMC_PROGRESS_SCREENSHOT });
        await page.locator('aside nav a[href="/mods"]').click();
        await page.waitForURL(/\/mods/);
        await page.locator('aside a[href="/instances/demo-vanilla"]').first().click();
        await panel.waitFor();
        await panel.getByText('Aguardando a janela do Minecraft...').waitFor();
        await page.evaluate(() => window.pendingLaunch.reject(new Error('Falha de teste recuperável')));
        await panel.waitFor({ state: 'detached' });
        assert.equal(errors.length, 0, errors.join('\n'));
        console.log('PASS integrated launch progress: native stages, reduced motion, navigation persistence and recoverable failure');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
