const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        await setupLauncherDemo(page);
        await page.goto('http://127.0.0.1:1420/');
        await page.getByRole('link', { name: 'Mods', exact: true }).waitFor();
        const counts = await page.evaluate(async () => {
            const { frameLoop } = await import('/src/lib/utils/frameLoop.ts');
            let normal = 0, limited = 0;
            const first = frameLoop(() => normal++, 60);
            const second = frameLoop(() => limited++, 30);
            await new Promise(resolve => setTimeout(resolve, 1500));
            first(); second();
            const ended = [normal, limited];
            await new Promise(resolve => setTimeout(resolve, 200));
            return { normal, limited, stopped: ended[0] === normal && ended[1] === limited };
        });
        assert.ok(counts.normal > 40 && counts.limited > 20 && counts.limited < counts.normal * .75, JSON.stringify(counts));
        assert.ok(counts.stopped);
        await page.evaluate(() => { const button = document.createElement('button'); button.id = 'motion-probe'; button.className = 'launcher-button launcher-button--primary'; button.textContent = 'Motion probe'; document.querySelector('main').append(button); });
        const button = page.locator('#motion-probe');
        const clicked = await page.evaluate(() => { const button = document.getElementById('motion-probe'); button.click(); return Boolean(button.querySelector('.launcher-button-wave')); });
        assert.ok(clicked, JSON.stringify(await page.evaluate(async () => { const {appState}=await import('/src/lib/stores/app.svelte.ts');const {settings}=await import('/src/lib/stores/settings.svelte.ts');return {performance:appState.performanceMode,launching:appState.isLaunching,game:appState.isGameRunning,animations:settings.value.animations,hidden:document.hidden,reduced:matchMedia('(prefers-reduced-motion: reduce)').matches}; })));
        await page.waitForFunction(() => !document.querySelector('#motion-probe .launcher-button-wave'));
        await page.evaluate(async () => { const { appState } = await import('/src/lib/stores/app.svelte.ts'); appState.performanceMode = true; });
        await button.click();
        assert.equal(await button.locator('.launcher-button-wave').count(), 0);
        console.log('PASS bounded 30 FPS scheduling reduces callbacks, stops cleanly and button effects leave no work in performance mode', counts);
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
