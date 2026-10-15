const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = 'docs/validation/v3.9.2';
    fs.mkdirSync(output, { recursive: true });
    try {
        const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            localStorage.setItem('luxmc_saved_skins', JSON.stringify([
                { id: 'search-a', name: 'Herói de gelo', model: 'steve', url: '/steve.png' },
                { id: 'search-b', name: 'Exploradora', model: 'alex', url: '/alex.png' }
            ]));
        });
        const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';
        await page.goto(`${base}/skins`);
        const search = page.getByRole('searchbox', { name: 'Buscar na sua coleção de skins' });
        await search.waitFor();
        await search.fill('HEROI');
        assert.equal(await page.locator('.studio-skin-card').count(), 1);
        await page.locator('.studio-skin-card').click();
        await page.getByRole('dialog', { name: 'Aplicar sua nova skin?' }).waitFor();
        await page.getByRole('button', { name: 'Continuar personalizando', exact: true }).click();
        await search.fill('não existe');
        await page.getByText('Nenhuma skin encontrada. Tente outro nome.', { exact: true }).waitFor();
        await search.fill('');
        assert.equal(await page.locator('.studio-skin-card').count(), 2);
        await page.getByRole('button', { name: 'Aproximar personagem', exact: true }).click();
        await page.getByRole('button', { name: 'Afastar personagem', exact: true }).click();
        assert.equal(await page.locator('.skin-preview-stage').evaluate(el => getComputedStyle(el).backgroundImage.includes('radial-gradient')), true);
        await page.getByRole('region', { name: 'Visualizador 3D de Skin' }).locator('canvas').first().waitFor({state:'visible', timeout:60000});
        await page.evaluate(async () => (await import('/src/lib/stores/settings.svelte.ts')).settings.patch({ skinPreviewQuality: 'high' }));
        await page.waitForFunction(() => { const c=document.querySelector('.skin-preview-stage canvas'); return c && Math.abs(c.width / c.clientWidth - 2) < .01; });
        await page.screenshot({ path: `${output}/personalizer.png`, animations: 'disabled' });
        await page.goto(`${base}/settings`);
        await page.getByRole('tab', { name: 'Aparência', exact: true }).click();
        await page.getByLabel('Nitidez da prévia 3D', { exact: true }).selectOption('economy');
        await page.getByLabel('Fluidez da animação', { exact: true }).selectOption('30');
        const preferences = await page.evaluate(async () => (await import('/src/lib/stores/settings.svelte.ts')).settings.value);
        assert.equal(preferences.skinPreviewQuality, 'economy');
        assert.equal(preferences.skinPreviewFps, 30);
        await page.getByRole('tab', { name: 'Aparência', exact: true }).press('ArrowDown');
        assert.equal(await page.getByRole('tab', { name: /Java/ }).getAttribute('aria-selected'), 'true');
        await page.getByRole('tab', { name: 'Aparência', exact: true }).click();
        assert.equal(await page.getByLabel('Nitidez da prévia 3D', { exact: true }).inputValue(), 'economy');
        await page.screenshot({ path: `${output}/settings.png`, animations: 'disabled' });
        for (const width of [1080, 900, 640]) {
            await page.setViewportSize({ width, height: 1000 });
            await page.waitForTimeout(150);
            const overflow = await page.locator('.settings-workspace').evaluate(el => ({overflow:el.scrollWidth > el.clientWidth + 1, nodes:[...el.querySelectorAll('*')].filter(node=>node.getBoundingClientRect().right>el.getBoundingClientRect().right+1).slice(0,8).map(node=>({tag:node.tagName,cls:node.className,text:node.textContent.slice(0,80)}))}));
            assert.equal(overflow.overflow, false, `Settings overflow at ${width}: ${JSON.stringify(overflow.nodes)}`);
        }
        assert.deepEqual(errors, []);
        console.log('3.9.2: skin collection search, zoom, neutral preview, quality and FPS preferences, accessible settings navigation and responsive layout verified.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
