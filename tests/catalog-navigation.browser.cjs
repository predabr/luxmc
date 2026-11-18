const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/catalog-navigation';
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
        const errors = [];
        const shot = async (page, path) => { for (let i = 0; i < 5; i++) { try { await page.screenshot({ path, animations: "disabled" }); return; } catch (e) { if (!/UNKNOWN/.test(String(e))) throw e; await new Promise(r => setTimeout(r, 1500)); } } };
        const latestSearch = async (predicate, arg) => { try { await page.waitForFunction(predicate, arg); } catch (e) { console.error("CALLS AT FAIL:", await page.evaluate(() => JSON.stringify(window.catalogCalls))); throw e; } };
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            const invoke = window.electronAPI.invoke;
            window.catalogCalls = [];
            window.electronAPI.invoke = async (command, args = {}) => {
                if (command === 'mods_search') window.catalogCalls.push(args);
                return invoke(command, args);
            };
        });
        await page.goto('http://127.0.0.1:1420/mods');
        await page.locator('.catalog-card').first().waitFor();
        for (const close of await page.locator('.notification button[aria-label]').all()) await close.click();
        await page.waitForTimeout(300);
        assert.equal(await page.locator('.spotlight,.catalog-sidebar').count(), 0);
        // Filter rail is always visible on wide viewports (SKLauncher-style) and the toggle stays hidden.
        await page.locator('.catalog-rail .catalog-filters').waitFor({ state: 'visible' });
        assert.equal(await page.locator('.catalog-filter-toggle').isVisible(), false);
        const content = await page.locator('.catalog-main').boundingBox();
        const filters = await page.locator('.catalog-rail').boundingBox();
        assert.ok(filters.x >= content.x + content.width - 1, 'Filter rail must sit in its own column right of the results');
        assert.equal(await page.locator('.catalog-navigation button').count(), 6);
        await page.keyboard.press('Escape');
        assert.equal(await page.locator('.catalog-rail .catalog-filters').isVisible(), true);

        for (const source of ['Modrinth', 'CurseForge']) {
            await page.locator('.catalog-rail').getByRole('button', { name: source, exact: true }).click();
            await latestSearch(source => window.catalogCalls.at(-1)?.source === source, source.toLowerCase());
        }
        await page.locator('.catalog-rail').getByRole('button', { name: 'Todos os Provedores', exact: true }).click();
        for (const loader of ['Fabric', 'Forge', 'Quilt', 'NeoForge']) {
            await page.locator('.catalog-rail').getByRole('button', { name: loader, exact: true }).click();
            await latestSearch(loader => window.catalogCalls.at(-1)?.loader === loader, loader.toLowerCase());
        }
        await page.locator('.catalog-rail').getByRole('button', { name: 'NeoForge', exact: true }).click();
        await page.getByRole('combobox', { name: 'Versão do Minecraft', exact: true }).selectOption('1.21.1');
        await latestSearch(() => window.catalogCalls.at(-1)?.mcVersion === '1.21.1');
        await page.locator('.catalog-rail').getByRole('button', { name: 'Desempenho', exact: true }).click();
        await latestSearch(() => window.catalogCalls.at(-1)?.category === 'Optimization');
        await page.getByRole('button', { name: 'Limpar pesquisa e filtros', exact: true }).click();
        assert.equal(await page.locator('.catalog-rail').getByRole('button', { name: 'Desempenho', exact: true }).getAttribute('aria-pressed'), 'false');
        // Content type selectable inside the filter panel, kept in sync with the top navigation.
        await page.locator('.catalog-rail').getByRole('button', { name: 'Data Pack', exact: true }).click();
        await latestSearch(() => window.catalogCalls.at(-1)?.contentType === 'datapack');
        assert.equal(await page.locator('.catalog-navigation button').nth(4).getAttribute('aria-current'), 'page');
        await page.evaluate(() => { window.catalogCalls.length = 0; });
        await page.locator('.catalog-rail').getByRole('button', { name: 'Shader', exact: true }).click();
        await latestSearch(() => window.catalogCalls.some(call => call.contentType === 'shader'));
        assert.equal(await page.locator('.catalog-navigation button').nth(3).getAttribute('aria-current'), 'page');
        await page.locator('.catalog-navigation button').nth(1).click();
        await latestSearch(() => window.catalogCalls.at(-1)?.contentType === 'mod');
        await page.locator('.catalog-navigation button').first().click();
        await page.locator('.catalog-card').filter({ hasText: 'Fabulously Optimized' }).waitFor();
        for (const card of await page.locator('.catalog-card').all()) {
            assert.ok(await card.evaluate(node => {
                const card = node.getBoundingClientRect();
                const button = node.querySelector('button').getBoundingClientRect();
                return button.bottom <= card.bottom && button.right <= card.right;
            }), 'Install action is clipped');
        }
        await page.locator('.catalog-card img').evaluateAll(images => Promise.all(images.map(image => image.decode().catch(() => {}))));
        await shot(page, output + '/catalog-dark.png');
        const title = await page.locator('.catalog-card h3').first().textContent();
        await page.locator('.catalog-card').first().press('Enter');
        await page.getByRole('heading', { name: title.trim(), exact: true, level: 1 }).waitFor();
        assert.equal(await page.locator('.catalog-rail').count(), 0);
        await page.goto('http://127.0.0.1:1420/mods');
        await page.locator('.catalog-card').first().waitFor();
        await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.setTheme('light', false));
        await page.waitForTimeout(300);
        await shot(page, output + '/catalog-light.png');
        const search = page.getByRole('searchbox');
        await search.fill('Fabulously');
        await page.locator('.catalog-card').filter({ hasText: 'Fabulously Optimized' }).waitFor();
        await search.fill('');
        for (const width of [1080, 800, 640]) {
            await page.setViewportSize({ width, height: 900 });
            await page.locator('main').evaluate(node => node.scrollTop = 0);
            await page.waitForTimeout(250);
            assert.ok(await page.locator('.catalog-page').evaluate(node => node.scrollWidth <= node.clientWidth + 1), 'Overflow at ' + width);
            await page.locator('.catalog-filter-toggle').click();
            await page.locator('.catalog-rail.open .catalog-filters').waitFor();
            await page.locator('.catalog-rail').getByRole('button', { name: 'Fabric', exact: true }).click();
            assert.equal(await page.locator('.catalog-rail').getByRole('button', { name: 'Fabric', exact: true }).getAttribute('aria-pressed'), 'true');
            await page.locator('.catalog-rail').getByRole('button', { name: 'Fabric', exact: true }).click();
            await page.keyboard.press('Escape');
            await page.locator('.catalog-rail.open').waitFor({ state: 'hidden' });
        }
        await page.locator('.catalog-filter-toggle').click();
        await page.locator('.catalog-rail.open').waitFor();
        await shot(page, output + '/catalog-compact.png');
        await page.setViewportSize({ width: 1600, height: 1000 });
        await page.evaluate(async () => {
            const { settings } = await import('/src/lib/stores/settings.svelte.ts');
            settings.value.catalogPageSize = 24;
            settings.value.catalogCompactCards = true;
        });
        await latestSearch(() => window.catalogCalls.at(-1)?.limit === 24);
        assert.equal(await page.locator('.catalog-compact').count(), 1);
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.locator('.catalog-card').first().evaluate(node => node.dispatchEvent(new MouseEvent('mouseover', { bubbles: true })));
        await page.waitForTimeout(120);
        assert.equal(await page.locator('.catalog-card').first().evaluate(node => getComputedStyle(node).translate), 'none');
        assert.deepEqual(errors, []);
        console.log('Filter rail, content types in filters, providers, all loaders, version/category filters, reset, content navigation, responsive drawer and reduced motion passed.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
