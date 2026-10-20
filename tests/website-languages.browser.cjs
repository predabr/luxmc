const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const base = process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8790/';
const output = process.env.LUXMC_SITE_LANGUAGE_OUTPUT;
const languages = {
    pt: { titles: ['Sua biblioteca.', 'Seus mods.', 'Sua turma.', 'Seu espaço.'], friends: 'Perfis públicos, favoritos e convites.', space: 'Crie pastas com nome e imagem', cta: 'Encontre seu plano', heading: 'Seu mundo.', download: 'Baixar LuxMC' },
    en: { titles: ['Your library.', 'Your mods.', 'Your friends.', 'Your space.'], friends: 'Public profiles, favourites and invitations.', space: 'Create folders with a name and image', cta: 'Find your plan', heading: 'Your world.', download: 'Download LuxMC' },
    es: { titles: ['Tu biblioteca.', 'Tus mods.', 'Tus amigos.', 'Tu espacio.'], friends: 'Perfiles públicos, favoritos e invitaciones.', space: 'Crea carpetas con nombre e imagen', cta: 'Encuentra tu plan', heading: 'Tu mundo.', download: 'Descargar LuxMC' }
};

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const evidence = [];
    try {
        const page = await browser.newPage({ reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.route('**/api/latest-release*', route => route.fulfill({ json: { tag_name: 'v4.0.0', assets: [] } }));
        await page.route('https://api.modrinth.com/**', route => route.fulfill({ json: { hits: [], total_hits: 0 } }));
        await page.route('**/api/session', route => route.fulfill({ json: { user: null, csrfToken: 'fixture' } }));
        await page.route('**/api/account/me', route => route.fulfill({ status: 401, json: { error: 'Not signed in' } }));
        await page.route('**/api/commerce/status', route => route.fulfill({ status: 401, json: { error: 'Not signed in' } }));
        for (const width of [1440, 390, 320]) {
            await page.setViewportSize({ width, height: 1000 });
            await page.goto(base);
            await page.waitForFunction(() => Boolean(window.LuxI18n));
            for (const [locale, expected] of Object.entries(languages)) {
                if (width < 901) {
                    await page.evaluate(() => scrollTo(0, 0));
                    await page.waitForFunction(() => document.getElementById('navMobileToggle').getBoundingClientRect().top >= 0);
                    const menu = await page.locator('#navMobileToggle').boundingBox();
                    assert.ok(menu.x >= 0 && menu.x + menu.width <= width + 1, `${width}px ${locale} menu must fit the viewport`);
                    assert.ok(menu.width >= 44 && menu.height >= 44);
                    await page.locator('#navMobileToggle').click();
                    await page.locator('#mobileNavDrawer [data-lang=' + locale + ']').click();
                    await page.keyboard.press('Escape');
                    assert.equal(await page.locator('#navMobileToggle').getAttribute('aria-expanded'), 'false');
                } else await page.locator('.nav-actions [data-lang=' + locale + ']').click();
                await page.waitForFunction(locale => window.LuxI18n.getLanguage() === locale, locale);
                assert.equal(await page.locator('.navbar').getByRole('link', { name: expected.download, exact: true }).count(), 1);
                if (width < 901) {
                    const download = await page.locator('.nav-actions > .btn-primary-nav').boundingBox();
                    assert.ok(download.x >= 0 && download.x + download.width <= width + 1);
                    assert.ok(download.width >= 44 && download.height >= 44);
                }
                const tour = await page.locator('.product-step').evaluateAll(nodes => nodes.map(node => ({ title: node.querySelector('h3').textContent.trim(), description: node.querySelector('p').textContent.trim(), number: node.querySelector('button > span').textContent.trim() })));
                assert.deepEqual(tour.map(card => card.title), expected.titles);
                assert.ok(tour[2].description.startsWith(expected.friends));
                assert.ok(tour[3].description.startsWith(expected.space));
                assert.deepEqual(tour.map(card => card.number), ['01', '02', '03', '04']);
                for (const [index, title] of expected.titles.entries()) {
                    assert.equal(await page.getByRole('button', { name: `${String(index + 1).padStart(2, '0')} · ${title}`, exact: true }).count(), 1);
                }
                for (const index of [2, 3]) {
                    await page.locator('[data-product-screen]').nth(index).click();
                    await page.waitForFunction(index => document.querySelectorAll('[data-product-screen]')[index].getAttribute('aria-pressed') === 'true', index);
                    assert.equal(await page.locator('[data-product-screen][aria-pressed="true"] h3').textContent(), expected.titles[index]);
                }
                assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
                evidence.push({ page: 'home', width, locale, tour });
            }
            if (width < 901 && output) {
                await page.evaluate(() => scrollTo(0, 0));
                fs.mkdirSync(path.dirname(output), { recursive: true });
                await page.locator('.navbar').screenshot({ path: path.join(path.dirname(output), `website-header-${width}.png`), animations: 'disabled' });
            }
            await page.goto(new URL('/planos', base).href);
            await page.waitForFunction(() => Boolean(window.LuxI18n));
            for (const [locale, expected] of Object.entries(languages)) {
                await page.locator('[data-language-select]').selectOption(locale);
                await page.waitForFunction(locale => window.LuxI18n.getLanguage() === locale, locale);
                if (width < 901) {
                    const languageControl = await page.locator('[data-language-select]').boundingBox();
                    assert.ok(languageControl.x >= 0 && languageControl.x + languageControl.width <= width + 1);
                }
                assert.equal(await page.locator('.plans-explore [data-i18n="plans.find"]').textContent(), expected.cta);
                assert.equal(await page.locator('.plans-explore > [aria-hidden="true"]').textContent(), '↓');
                assert.equal(await page.locator('.plans-explore').getAttribute('href'), '#plan-grid');
                assert.ok((await page.locator('.plans-intro h1').textContent()).includes(expected.heading));
                assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1));
                evidence.push({ page: 'plans', width, locale, cta: expected.cta });
            }
            await page.reload();
            await page.waitForFunction(() => document.documentElement.lang === 'es');
            assert.equal(await page.locator('.plans-explore [data-i18n="plans.find"]').textContent(), languages.es.cta);
            if (width < 901) for (const route of ['/conta', '/privacidade', '/termos']) {
                await page.goto(new URL(route, base).href);
                const header = await page.locator('.navbar').boundingBox();
                assert.ok(header.x >= 0 && header.x + header.width <= width + 1);
                for (const action of await page.locator('.navbar .nav-actions > a,.navbar #navMobileToggle').all()) {
                    if (!await action.isVisible()) continue;
                    const box = await action.boundingBox();
                    assert.ok(box.x >= 0 && box.x + box.width <= width + 1, `${route} action must fit ${width}px`);
                }
                if (await page.locator('#navMobileToggle').count()) {
                    await page.locator('#navMobileToggle').click();
                    assert.equal(await page.locator('#mobileNavDrawer').getAttribute('aria-hidden'), 'false');
                    await page.keyboard.press('Escape');
                    assert.equal(await page.locator('#mobileNavDrawer').getAttribute('aria-hidden'), 'true');
                }
                evidence.push({ page: route, width, headerActionsFit: true });
            }
        }
        assert.deepEqual(errors, []);
        if (output) {
            fs.mkdirSync(path.dirname(output), { recursive: true });
            fs.writeFileSync(output, JSON.stringify({ passed: true, evidence }, null, 2));
        }
        console.log('Website language regression passed: all four tour cards and plans CTA in PT/EN/ES, desktop/mobile, selection, persistence and no horizontal overflow.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
