const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        const errors = [];
        let libraryUrl;
        page.on('request', request => { if (request.url().includes('skinview3d') && /\.js(?:\?|$)/.test(request.url())) libraryUrl = request.url(); });
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.goto(`${process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420'}/skins`);
        const canvas = page.locator('.skin-preview-stage canvas');
        await page.waitForFunction(() => document.querySelector('.skin-preview-stage canvas') && !document.querySelector('.skin-preview-stage canvas').classList.contains('invisible'));
        await page.evaluate(async moduleUrl => {
            if (!moduleUrl) throw new Error('Skinview3d não foi carregado');
            const library = await import(moduleUrl);
            const update = library.PlayerAnimation.prototype.update;
            window.skinMotionSamples = [];
            library.PlayerAnimation.prototype.update = function(player, delta) {
                update.call(this, player, delta);
                window.skinMotionSamples.push({ type: this.constructor.name, leg: player.skin.leftLeg.rotation.x, arm: player.skin.rightArm.rotation.x, height: player.position.y, tilt: player.rotation.x, delta });
                if (window.skinMotionSamples.length > 100) window.skinMotionSamples.shift();
            };
            window.previewAppState = (await import('/src/lib/stores/app.svelte.ts')).appState;
            window.previewAppState.performanceMode = true;
        }, libraryUrl);
        for (const [label, type] of [['Andar', 'WalkingAnimation'], ['Correr', 'RunningAnimation'], ['Voar', 'FlyingAnimation']]) {
            await page.evaluate(() => { window.skinMotionSamples = []; });
            await page.getByRole('button', { name: label, exact: true }).click();
            await page.waitForFunction(expected => window.skinMotionSamples.filter(sample => sample.type === expected && sample.delta > 0).length > 12, type);
            const samples = await page.evaluate(expected => window.skinMotionSamples.filter(sample => sample.type === expected), type);
            assert.ok(samples.some(sample => Math.abs(sample.leg - samples[0].leg) > .02 || Math.abs(sample.arm - samples[0].arm) > .02 || Math.abs(sample.height - samples[0].height) > .02 || Math.abs(sample.tilt - samples[0].tilt) > .02), `${label} não alterou a pose`);
            assert.equal(await canvas.evaluate(element => element.classList.contains('invisible')), false);
        }
        await page.evaluate(() => { window.previewAppState.performanceMode = false; window.previewAppState.isGameRunning = true; window.skinMotionSamples = []; });
        await page.getByRole('button', { name: 'Andar', exact: true }).click();
        await page.waitForFunction(() => window.skinMotionSamples.filter(sample => sample.type === 'WalkingAnimation').length > 12);
        await page.evaluate(() => { window.previewAppState.isGameRunning = false; });
        assert.deepEqual(errors, []);
        console.log('PASS actual 3D poses advance for walking, running and flying in performance mode and while Minecraft is running.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
