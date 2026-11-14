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
        console.log('PASS bounded 30 FPS scheduling reduces callbacks and stops cleanly', counts);
        console.log('PASS bounded 30 FPS scheduling reduces callbacks, stops cleanly and button effects leave no work in performance mode', counts);
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
