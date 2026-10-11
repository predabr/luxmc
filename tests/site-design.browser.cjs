const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/design-catalog';
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.route('https://api.modrinth.com/**', route => route.fulfill({ json: { total_hits: 3, hits: Array.from({ length: 3 }, (_, index) => ({ project_id: `demo-${index}`, slug: `demo-${index}`, title: ['Vanilla Perfected', 'Fabulously Optimized', 'Better Minecraft'][index], author: 'Demonstração', description: 'Dados de demonstração para verificar o layout do catálogo.', downloads: 123456, icon_url: 'https://cdn.modrinth.com/data/demo/icon.webp', featured_gallery: `https://cdn.modrinth.com/data/demo/${index ? 'gallery' : 'missing'}.webp`, categories: ['fabric', 'adventure'], versions: ['1.21.1'] })) } }));
        await page.route('https://cdn.modrinth.com/**', route => route.request().url().endsWith('/missing.webp') ? route.fulfill({ status: 404, body: '' }) : route.fulfill({ contentType: 'image/webp', body: fs.readFileSync('static/modpack_better_mc.webp') }));
        await page.goto('http://127.0.0.1:8791/');
        await page.locator('.project-cover img').first().waitFor();
        await page.waitForTimeout(900);
        assert.equal(await page.locator('.hero-capture img').getAttribute('src'), 'assets/captures-3.6/home-window.svg');
        await page.screenshot({ path: `${output}/site-desktop.png`, animations: 'disabled' });
        await page.locator('#mods').scrollIntoViewIfNeeded();
        await page.waitForFunction(() => { const image = document.querySelector('.project-cover img'); return image?.src.endsWith('/assets/logo.png') && image.complete && image.naturalWidth > 0; });
        await page.waitForTimeout(800);
        await page.screenshot({ path: `${output}/site-catalog.png`, animations: 'disabled' });
        await page.setViewportSize({ width: 390, height: 844 });
        await page.evaluate(() => scrollTo(0, 0));
        await page.waitForTimeout(400);
        await page.screenshot({ path: `${output}/site-mobile.png`, animations: 'disabled' });
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
        await page.locator('#mods').scrollIntoViewIfNeeded();
        const cards = await page.locator('.project-card').evaluateAll(nodes => nodes.map(node => {
            const rect = node.getBoundingClientRect();
            return rect.width > 0 && rect.left >= 0 && rect.right <= innerWidth;
        }));
        assert.ok(cards.every(Boolean));
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.waitForTimeout(100);
        assert.ok(await page.locator('body').evaluate(node => node.classList.contains('motion-paused')));
        await page.setViewportSize({ width: 1440, height: 1000 });
        await page.goto('http://127.0.0.1:8791/conta');
        await page.locator('#authForm').waitFor();
        await page.screenshot({ path: `${output}/site-account.png`, animations: 'disabled' });
        assert.deepEqual(errors, []);
        console.log('Local website desktop/mobile catalog, account page and reduced motion verified with fixture data.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
