const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { createHash } = require('node:crypto');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        const url = process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8790/';
        await page.goto(url);
        const image = page.locator('.identity-capture img');
        await image.scrollIntoViewIfNeeded();
        await image.evaluate(node => node.decode());
        const sectionDimensions = await image.evaluate(node => ({ width: node.naturalWidth, height: node.naturalHeight, src: node.src }));
        assert.deepEqual({ width: sectionDimensions.width, height: sectionDimensions.height }, { width: 1919, height: 1022 });
        assert.ok(sectionDimensions.src.endsWith('/personalizer-window.svg'));
        await page.locator('.identity-capture').screenshot({ path: 'docs/validation/v3.6/layout-polish/personalizer-site-section.png' });
        await page.locator('[data-product-screen*="personalizer-window.svg"]').click();
        await page.waitForFunction(() => document.getElementById('productTourImage').src.endsWith('/personalizer-window.svg'));
        await page.locator('#btnZoomShowcase').click();
        await page.locator('#showcaseModalImg').evaluate(node => node.decode());
        const expanded = await page.locator('#showcaseModalImg').evaluate(node => ({ width: node.naturalWidth, height: node.naturalHeight, src: node.src }));
        assert.deepEqual(expanded, sectionDimensions);
        await page.locator('#showcaseModal').screenshot({ path: 'docs/validation/v3.6/layout-polish/personalizer-site-expanded.png' });
        const response = await page.request.get(sectionDimensions.src);
        assert.equal(response.status(), 200);
        const delivered = await response.body();
        const local = fs.readFileSync('website/assets/captures-3.6/personalizer-window.svg');
        assert.equal(createHash('sha256').update(delivered).digest('hex'), createHash('sha256').update(local).digest('hex'));
        assert.deepEqual(errors, []);
        fs.writeFileSync('docs/validation/v3.6/layout-polish/personalizer-site-browser.json', JSON.stringify({ passed: true, url, sectionDimensions, expanded, imageMatchesLocalAsset: true, taskbarOutsideViewport: true }, null, 2));
        console.log('Personalizer screenshot verified in the site section and enlarged view: 1919 × 1022, taskbar excluded.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
