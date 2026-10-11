const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/design-catalog';
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
        const errors = [];
        const clearNotifications = async () => {
            for (const close of await page.locator('.notification button[aria-label]').all()) await close.click();
            await page.waitForTimeout(220);
        };
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.goto('http://127.0.0.1:1420/mods');
        await page.locator('.catalog-card').first().waitFor();
        await page.getByRole('heading', { name: 'Um novo mundo começa aqui.' }).waitFor();
        await page.waitForTimeout(800);
        await clearNotifications();
        assert.equal(await page.locator('.catalog-navigation button').count(), 6);
        await page.screenshot({ path: `${output}/modpacks-dark.png`, animations: 'disabled' });
        await page.locator('.catalog-navigation button').nth(1).click();
        await page.waitForFunction(() => document.querySelectorAll('.catalog-navigation button')[1].getAttribute('aria-pressed') === 'true');
        await page.locator('.catalog-navigation button').first().click();
        await page.locator('.catalog-filter-toggle').click();
        await page.getByRole('button', { name: 'Fabric', exact: true }).click();
        assert.equal(await page.getByRole('button', { name: 'Fabric', exact: true }).getAttribute('aria-pressed'), 'true');
        await page.getByRole('button', { name: 'Fabric', exact: true }).click();
        await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.setTheme('light', false));
        await page.waitForTimeout(300);
        await page.screenshot({ path: `${output}/modpacks-light.png`, animations: 'disabled' });
        for (const width of [1080, 900, 700]) {
            await page.setViewportSize({ width, height: 900 });
            await page.reload();
            await page.locator('.catalog-navigation').waitFor();
            await page.waitForTimeout(300);
            await page.locator('.catalog-filter-toggle').click();
            assert.equal(await page.locator('.catalog-filters details').getAttribute('open'), null);
            await page.locator('.catalog-filters summary').click();
            await page.getByRole('combobox', { name: 'Versão' }).waitFor();
            const bounds = await page.evaluate(() => {
                const main = document.querySelector('main');
                const rect = main.getBoundingClientRect();
                return [...document.querySelectorAll('.catalog-hero,.catalog-navigation,.catalog-toolbar,.catalog-filters')].map(node => {
                    const box = node.getBoundingClientRect();
                    return { left: box.left >= rect.left - 1, right: box.right <= rect.right + 1 };
                });
            });
            assert.ok(bounds.every(bound => bound.left && bound.right), `Catalog overflow at ${width}`);
        }
        await page.screenshot({ path: `${output}/modpacks-compact.png`, animations: 'disabled' });
        await page.setViewportSize({ width: 1600, height: 1000 });
        await page.getByRole('button', { name: 'Abrir minhas instâncias' }).click();
        await page.waitForURL('**/instances');
        await page.screenshot({ path: `${output}/instances.png`, animations: 'disabled' });
        for (const [route, name] of [['/friends', 'friends'], ['/settings', 'settings'], ['/', 'home'], ['/workshop', 'workshop'], ['/news', 'news'], ['/screenshots', 'screenshots'], ['/logs', 'logs'], ['/logs-history', 'history'], ['/boost', 'boost']]) {
            await page.goto(`http://127.0.0.1:1420${route}`);
            await page.locator('main[data-refined-ui="true"]').waitFor();
            await page.waitForTimeout(600);
            await clearNotifications();
            await page.screenshot({ path: `${output}/${name}.png`, animations: 'disabled' });
        }
        assert.deepEqual(errors, []);
        console.log('Premium catalog navigation, filters, compact layout, themes and core screens verified with fixture data.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
