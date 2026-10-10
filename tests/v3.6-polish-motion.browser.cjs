const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        await setupLauncherDemo(page);
        await page.goto('http://127.0.0.1:1420/instances');
        await page.getByRole('button', { name: /Nova Instância|Nova instância/ }).last().click();
        const card = page.getByRole('dialog').locator('.edition-card').first();
        await card.hover();
        await page.waitForTimeout(500);
        for (const disabled of ['system', 'no-ui-motion', 'no-anim']) {
            if (disabled !== 'system') {
                await page.emulateMedia({ reducedMotion: 'no-preference' });
                await page.evaluate(name => { document.documentElement.classList.remove('no-ui-motion', 'no-anim'); document.documentElement.classList.add(name); }, disabled);
            }
            const properties = await card.evaluate(node => ({ animation: getComputedStyle(node).animationName, translate: getComputedStyle(node).translate, icon: getComputedStyle(node.querySelector('img')).transform }));
            assert.deepEqual(properties, { animation: 'none', translate: 'none', icon: 'none' }, disabled);
        }
        fs.writeFileSync('docs/validation/v3.6/layout-polish/motion-preferences.json', JSON.stringify({ passed: true, reducedMotion: true, animationsDisabled: true, economicalMode: true }, null, 2));
        console.log('New launcher motion respects system preference, disabled animations and economical mode, including hovered icons.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
