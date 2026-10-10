const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        for (const [tag, display, title] of [
            ['v3.6.1-revision.2', 'v3.6.0', 'Luxmc 3.6.0 — revisão 2'],
            ['v3.7.0', 'v3.7.0', 'Luxmc 3.7.0']
        ]) {
            const page = await browser.newPage();
            await page.route('**/api/latest-release*', route => route.fulfill({ json: { tag_name: tag, assets: [] } }));
            await page.goto(process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8790/');
            await page.waitForFunction(expected => window.LuxLatestVersion === expected && [...document.querySelectorAll('.live-version-tag')].every(node => node.title !== ''), display);
            const versions = await page.locator('.live-version-tag').evaluateAll(nodes => nodes.map(node => ({ text: node.textContent, title: node.title })));
            assert.ok(versions.length > 0);
            assert.ok(versions.every(node => node.text === display && node.title === title));
            await page.close();
        }
        console.log('Site release identity passed: same-version revision and future release.');
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
