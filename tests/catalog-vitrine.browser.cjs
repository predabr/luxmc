const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/catalog-vitrine';
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1600, height: 1100 } });
        const errors = [];
        const dismissNotifications = async () => {
            for (const close of await page.locator('.notification button[aria-label]').all()) await close.click();
            await page.waitForTimeout(220);
        };
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const invoke = window.electronAPI.invoke;
            window.vitrineCalls = [];
            window.electronAPI.invoke = async (command, args = {}) => {
                window.vitrineCalls.push({ command, args });
                return invoke(command, args);
            };
        });
        await page.goto('http://127.0.0.1:1420/mods');
        await page.locator('.spotlight-pick').first().waitFor();
        for (const close of await page.locator('.notification button[aria-label]').all()) await close.click();
        await page.waitForTimeout(600);
        assert.equal(await page.locator('.catalog-filters').count(), 0);
        const secondTitle = await page.locator('.spotlight-pick strong').nth(1).textContent();
        await page.locator('.spotlight-pick').nth(1).click();
        await page.locator('.spotlight h2').filter({ hasText: secondTitle }).waitFor();
        await page.getByRole('button', { name: 'Explorar modpack', exact: true }).click();
        await page.waitForFunction(() => !document.querySelector('.spotlight'));
        assert.ok(await page.getByRole('heading', { name: secondTitle, exact: true }).count());
        await page.goto('http://127.0.0.1:1420/mods');
        await page.locator('.spotlight-pick').first().waitFor();
        await page.getByRole('button', { name: 'Mais FPS', exact: true }).click();
        await page.waitForFunction(() => window.vitrineCalls.some(call => call.command === 'mods_search' && call.args.category === 'Optimization'));
        await page.getByRole('button', { name: 'Mais FPS', exact: true }).click();
        await page.getByRole('button', { name: 'CurseForge', exact: true }).click();
        await page.waitForFunction(() => window.vitrineCalls.some(call => call.command === 'mods_search' && call.args.source === 'curseforge'));
        await page.getByRole('button', { name: 'Todos os Provedores', exact: true }).click();
        await page.locator('.catalog-filter-toggle').click();
        await page.getByRole('button', { name: 'Fabric', exact: true }).click();
        await page.waitForFunction(() => window.vitrineCalls.some(call => call.command === 'mods_search' && call.args.loader === 'fabric'));
        await page.getByRole('button', { name: 'Fabric', exact: true }).click();
        await page.locator('.catalog-filter-toggle').click();
        await page.waitForTimeout(500);
        await page.locator('.spotlight-pick').nth(1).click();
        await dismissNotifications();
        await page.screenshot({ path: `${output}/vitrine-dark.png`, animations: 'disabled' });
        await page.locator('.catalog-card').first().scrollIntoViewIfNeeded();
        await page.waitForTimeout(300);
        for (const card of await page.locator('.catalog-card').all()) {
            const visible = await card.evaluate(node => {
                const card = node.getBoundingClientRect();
                const button = node.querySelector('button').getBoundingClientRect();
                return button.bottom <= card.bottom && button.right <= card.right;
            });
            assert.ok(visible, 'Install action is clipped');
        }
        await page.screenshot({ path: `${output}/cards-dark.png`, animations: 'disabled' });
        await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.setTheme('light', false));
        await page.locator('main').evaluate(node => node.scrollTop = 0);
        await page.waitForTimeout(300);
        await page.screenshot({ path: `${output}/vitrine-light.png`, animations: 'disabled' });
        const search = page.getByRole('searchbox');
        await search.fill('Fabulously');
        await page.waitForFunction(() => !document.querySelector('.spotlight'));
        await page.locator('.catalog-card').filter({ hasText: 'Fabulously Optimized' }).waitFor();
        await search.fill('');
        for (const width of [1080, 800, 640]) {
            await page.setViewportSize({ width, height: 900 });
            await page.locator('main').evaluate(node => node.scrollTop = 0);
            await page.waitForTimeout(400);
            assert.ok(await page.locator('.catalog-page').evaluate(node => node.scrollWidth <= node.clientWidth + 1), `Overflow at ${width}`);
        }
        await page.screenshot({ path: `${output}/vitrine-compact.png`, animations: 'disabled' });
        await page.evaluate(async()=>{const {settings}=await import('/src/lib/stores/settings.svelte.ts');settings.value.catalogPageSize=24;settings.value.catalogSpotlight=false;settings.value.catalogCompactCards=true;});
        await page.waitForFunction(()=>window.vitrineCalls.some(call=>call.command==='mods_search' && call.args.limit===24));assert.equal(await page.locator('.spotlight').count(),0);assert.equal(await page.locator('.catalog-compact').count(),1);await page.evaluate(async()=>{const {settings}=await import('/src/lib/stores/settings.svelte.ts');settings.value.catalogSpotlight=true;});await page.locator('.spotlight-explore').waitFor();
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.locator('.spotlight-explore').hover();
        assert.equal(await page.locator('.spotlight-explore').evaluate(node => getComputedStyle(node).translate), 'none');
        assert.deepEqual(errors, []);
        console.log('Interactive spotlight, provider/category/loader filters, visible installation actions, search, responsive layouts and reduced motion passed.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
