const { chromium } = require('playwright-core');
const fs = require('node:fs');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
    const checks = [];
    try {
        await page.goto('https://luxmc-r92.pages.dev/', { waitUntil: 'networkidle' });
        const buttons = page.locator('[data-product-screen]');
        for (let index = 0; index < await buttons.count(); index++) {
            const name = await buttons.nth(index).getAttribute('data-product-screen');
            await buttons.nth(index).click();
            await page.waitForFunction(expected => { const image = document.getElementById('productTourImage'); return image.getAttribute('src') === expected && image.complete && image.naturalWidth > 0; }, name);
            const value = await page.locator('#productTourImage').evaluate(image => ({ source: image.getAttribute('src'), width: image.naturalWidth, height: image.naturalHeight }));
            assert.ok(value.source.endsWith('-window.svg'));
            assert.equal(value.source, name);
            assert.ok(value.height <= 1022);
            checks.push({ name, ...value });
            if (/personalizer|skins|personal/i.test(name)) await page.locator('#productTourImage').screenshot({ path: 'docs/validation/v3.6/publication-revision2/personalizer-public.png' });
        }
        assert.equal(checks.length, 5);
        const result = { passed: true, origin: 'https://luxmc-r92.pages.dev', captures: checks };
        fs.writeFileSync('docs/validation/v3.6/publication-revision2/published-captures.json', JSON.stringify(result, null, 2));
        console.log(JSON.stringify(result));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
