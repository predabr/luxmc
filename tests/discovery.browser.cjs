const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
    const errors = [];
    const searches = [];
    let offline = false;
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => { localStorage.setItem('luxmc_lang', 'pt'); localStorage.setItem('luxmc_cookie_consent', 'true'); });
    await page.route('https://**/*', route => {
        const url = new URL(route.request().url());
        if (url.pathname === '/v2/tag/game_version') return route.fulfill({ json: [{ version: '1.21.1', version_type: 'release' }] });
        if (url.pathname === '/v2/search') {
            searches.push(url);
            if (offline) return route.fulfill({ status: 503, body: 'Unavailable' });
            const offset = Number(url.searchParams.get('offset'));
            return route.fulfill({ json: { total_hits: 24, hits: Array.from({ length: 12 }, (_, index) => ({ project_id: `id-${offset + index}`, slug: `project-${offset + index}`, title: `Projeto ${offset + index}`, description: 'Mods para construir novos mundos e explorar com seus amigos.', author: 'Luxmc', downloads: 12000, categories: ['fabric', 'adventure'], icon_url: '/assets/logo.png' })) } });
        }
        return route.abort();
    });
    await page.goto(process.env.LUXMC_WEBSITE_URL || 'http://127.0.0.1:4174/');
    await page.getByText('Projeto 0', { exact: true }).waitFor();
    await page.locator('#modLoader').selectOption('fabric');
    await page.locator('#modGameVersion').selectOption('1.21.1');
    await page.getByText('Projeto 0', { exact: true }).waitFor();
    const facets = JSON.parse(searches.at(-1).searchParams.get('facets'));
    assert.ok(facets.some(group => group.includes('versions:1.21.1')));
    await page.locator('#modNext').click();
    await page.getByText('Projeto 12', { exact: true }).waitFor();
    assert.equal(searches.at(-1).searchParams.get('offset'), '12');
    await page.locator('#modReset').click();
    await page.getByText('Projeto 0', { exact: true }).waitFor();
    assert.equal(await page.locator('#modGameVersion').inputValue(), '');
    offline = true;
    await page.locator('#modSearchInput').fill('offline');
    await page.locator('.catalog-retry').waitFor();
    assert.equal(await page.locator('.project-card').count(), 0);
    offline = false;
    await page.locator('.catalog-retry').click();
    await page.getByText('Projeto 0', { exact: true }).waitFor();
    const directory = process.env.LUXMC_ARTIFACT_DIR || 'docs/visual/2026-10-03-discovery';
    fs.mkdirSync(directory, { recursive: true });
    await page.locator('#mods').scrollIntoViewIfNeeded();
    await page.evaluate(() => window.scrollTo(0, document.getElementById('mods').offsetTop - 80));
    await page.screenshot({ path: `${directory}/website-catalog.png` });
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.screenshot({ path: `${directory}/website-home.png` });
    for (const width of [1024, 768, 390]) {
        await page.setViewportSize({ width, height: 900 });
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `Horizontal overflow at ${width}`);
        const toggle = page.locator('#navMobileToggle');
        await toggle.click();
        assert.equal(await page.locator('#mobileNavDrawer').evaluate(element => element.inert), false);
        await page.keyboard.press('Escape');
        assert.equal(await toggle.getAttribute('aria-expanded'), 'false');
        assert.equal(await page.locator('#mobileNavDrawer').evaluate(element => element.inert), true);
    }
    await page.screenshot({ path: `${directory}/website-mobile.png` });
    await page.locator('#mods').scrollIntoViewIfNeeded();
    await page.evaluate(() => window.scrollTo(0, document.getElementById('mods').offsetTop - 80));
    await page.screenshot({ path: `${directory}/website-catalog-mobile.png` });
    for (const route of ['conta.html', 'skins.html', 'privacidade.html', 'termos.html']) {
        await page.goto(new URL(route, process.env.LUXMC_WEBSITE_URL || 'http://127.0.0.1:4174/').href);
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `Horizontal overflow in ${route}`);
    }
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ filters: true, pagination: true, recovery: true, widths: [1440, 1024, 768, 390], errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
