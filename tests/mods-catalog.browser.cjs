const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
assert.ok(JSON.parse(fs.readFileSync('src-tauri/capabilities/default.json', 'utf8')).permissions.includes('core:window:allow-destroy'));

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ locale: 'en-US', viewport: { width: 1440, height: 900 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' };
        window.searchCalls = []; window.uiCalls = []; window.callbacks = {}; window.eventHandlers = {};
        const results = Array.from({ length: 36 }, (_, index) => ({
            sourceId: String(index),
            source: index % 2 ? 'curseforge' : 'modrinth',
            slug: `pack-${index}`,
            title: `Pack ${index}`,
            description: `Descrição completa do modpack ${index}`,
            downloads: 100000 - index,
            iconUrl: '/steve.png',
            bannerUrl: null,
            author: 'Luxmc',
            categories: ['forge'],
            versions: ['1.21.1']
        }));
        results[0].bannerUrl = '/missing-banner.png';
        const invoke = async (command, args) => {
            window.uiCalls.push({ command, args });
            if (command === 'mods_versions') return [{ id: 'new', name: 'Atual', versionNumber: '2', files: [{ url: 'https://cdn.modrinth.com/test.mrpack', filename: 'test.mrpack', size: 100 }] }, { id: 'old', name: 'Anterior', versionNumber: '1', files: [{ url: 'https://cdn.modrinth.com/old.mrpack', filename: 'old.mrpack', size: 100 }] }];
            if (command === 'mods_project_details') return { id: args.projectId, slug: 'pack', title: 'Pack', description: 'Descrição', body: '', bodyType: 'markdown', categories: ['forge'], loaders: ['forge'], gameVersions: ['1.21.1'], downloads: 100, gallery: [], links: {} };
            if (command === 'versions_list') return { versions: [{ id: '1.21.1', versionType: 'release', releaseTime: '2024-08-08' }], latestRelease: '1.21.1', latestSnapshot: '' };
            if (command === 'mods_download_to_temp') return new Promise((resolve, reject) => { window.cancelDownload = () => reject(new Error('Importação cancelada')); });
            if (command === 'instance_cancel_import') { window.cancelDownload?.(); return null; }
            if (command === 'plugin:dialog|message') return window.approveClose === true ? 'Fechar' : 'Cancelar';

            if (command === 'mods_search') {
                window.searchCalls.push(args);
                if (args.query === 'lenta') return new Promise(resolve => { window.releaseSlow = () => resolve([{ ...results[0], title: 'Resposta antiga' }]); });
                return results;
            }
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'java_scan') return { runtimes: [] };
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'screenshots_list') return [];
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'curseforge_status') return true;
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'mesh_status') return { available: false, state: '', ip: null, peers: [] };
            return null;
        };
        window.electronAPI = { invoke, on: (event, handler) => { window.eventHandlers[event] = handler; return () => delete window.eventHandlers[event]; } };
        window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
        window.__TAURI_INTERNALS__ = { metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } }, invoke, transformCallback: handler => { const id = Object.keys(window.callbacks).length + 1; window.callbacks[id] = handler; return id; }, convertFileSrc: path => path };
    });
    await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/mods');
    await page.getByText('Pack 0', { exact: true }).waitFor();
    assert.equal(await page.locator('[data-virtual-list]').count(), 0);
    assert.ok(await page.getByText(/^Pack \d+$/).count() < 20);
    assert.equal(await page.evaluate(() => window.searchCalls.at(-1)?.limit), 36);
    const icon = page.locator(".catalog-card").filter({ has: page.getByText("Pack 1", { exact: true }) }).locator("img").first();
    const bounds = await icon.boundingBox();
    assert.ok(bounds && bounds.height >= 180 && bounds.width >= 250, JSON.stringify(bounds));
    await icon.evaluate(image => image.decode());
    assert.ok(await icon.evaluate(image => image.complete && image.naturalWidth > 0));
    const nestedScroll = await icon.evaluate(element => {
        const main = element.closest('main');
        let current = element.parentElement;
        while (current && current !== main) {
            const overflow = getComputedStyle(current).overflowY;
            if (overflow === 'auto' || overflow === 'scroll') return true;
            current = current.parentElement;
        }
        return false;
    });
    assert.equal(nestedScroll, false);
    await page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
    await page.mouse.wheel(0, 600);
    await page.waitForTimeout(150);
    const scrollTop = await page.locator('main').evaluate(element => element.scrollTop);
    assert.ok(scrollTop > 0, `mouse wheel did not move the catalog: ${scrollTop}`);
    await page.waitForFunction(() => !document.querySelector('main').classList.contains('is-scrolling'));
    const frames = await page.evaluate(async () => {
        const root = document.querySelector('main');
        root.scrollTop = 0;
        const durations = [];
        let previous = performance.now();
        for (let index = 0; index < 60; index++) {
            await new Promise(requestAnimationFrame);
            const now = performance.now();
            durations.push(now - previous);
            previous = now;
            root.scrollTop += 60;
        }
        return durations.sort((a, b) => a - b);
    });
    assert.ok(frames[57] < 150, `scroll frame p95 exceeded 150ms: ${frames[57]}`);
    const search = page.locator('input[placeholder^="Buscar"]');
    await search.fill('lenta');
    await page.waitForFunction(() => typeof window.releaseSlow === 'function');
    await search.fill('atual');
    await page.evaluate(() => window.releaseSlow());
    await page.waitForTimeout(100);
    assert.equal(await page.getByText('Resposta antiga', { exact: true }).count(), 0);
    await page.waitForFunction(() => window.searchCalls.at(-1)?.query === 'atual');
    await page.getByRole('button', { name: 'Próxima página' }).click();
    await page.waitForFunction(() => window.searchCalls.at(-1)?.offset === 36);
    assert.equal(await page.locator('main').evaluate(element => element.scrollTop), 0);
    await page.getByRole('button', { name: 'Lista', exact: true }).click();
    const install = page.locator('.catalog-list-card button').first();
    await install.focus();
    await page.keyboard.press('Enter');
    await page.getByRole('dialog', { name: 'Instalar modpack', exact: true }).waitFor();
    const installer = page.getByRole('dialog', { name: 'Instalar modpack', exact: true });
    await installer.getByLabel('Nome da instância', { exact: true }).fill('Meu pack');
    await installer.getByRole('button', { name: '6 GB', exact: true }).click();
    assert.equal(await installer.locator('#modpack-ram').inputValue(), '6144');
    if (process.env.LUXMC_ARTIFACT_DIR) { fs.mkdirSync(process.env.LUXMC_ARTIFACT_DIR, { recursive: true }); await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/installer-config.png` }); }
    await installer.getByRole('button', { name: 'Instalar modpack', exact: true }).click();
    await page.waitForFunction(() => typeof window.cancelDownload === 'function');
    await page.evaluate(() => window.eventHandlers['modpack-progress']({ phase: 'mods', current: 12, total: 187, percent: 6, status: 'Baixando mods (12/187)…' }));
    await installer.getByRole('progressbar').getAttribute('aria-valuenow').then(value => assert.equal(value, '6'));
    await page.waitForFunction(() => document.querySelector('.installer-progress-fill')?.getBoundingClientRect().width > 20);
    if (process.env.LUXMC_ARTIFACT_DIR) await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/installer-progress.png` });
    await installer.getByRole('button', { name: 'Cancelar instalação', exact: true }).click();
    await page.getByText('Importação cancelada.', { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => window.uiCalls.filter(call => call.command === 'instance_cancel_import').length), 1);
    await installer.getByRole('button', { name: 'Cancelar', exact: true }).click();
    await page.locator('.catalog-list-card').first().click();
    await page.getByRole('button', { name: /^Versões/ }).click();
    const previousVersion = page.getByText('Anterior', { exact: true }).locator('..').locator('..').locator('..');
    await previousVersion.getByRole('button', { name: 'Criar', exact: true }).click();
    await installer.getByRole('button', { name: 'Instalar modpack', exact: true }).click();
    await page.waitForFunction(() => window.uiCalls.filter(call => call.command === 'mods_download_to_temp').length === 2);
    assert.equal(await page.evaluate(() => window.uiCalls.filter(call => call.command === 'mods_download_to_temp').at(-1).args.url), 'https://cdn.modrinth.com/old.mrpack');
    await installer.getByRole('button', { name: 'Cancelar instalação', exact: true }).click();
    await installer.getByRole('button', { name: 'Cancelar', exact: true }).click();
    const closeListenersIdle = await page.evaluate(() => window.uiCalls.filter(call => call.command === 'plugin:event|listen' && call.args.event === 'tauri://close-requested').length);
    assert.equal(closeListenersIdle, 0);
    if (!process.env.LUXMC_BASE_URL) {
    await page.evaluate(async () => { const url = performance.getEntriesByType('resource').find(entry => entry.name.includes('/stores/app.svelte.ts')).name; const { appState } = await import(url); window.testAppState = appState; appState.isGameRunning = true; });
    await page.waitForTimeout(200);
    assert.equal(await page.evaluate(() => window.uiCalls.filter(call => call.command === 'plugin:window|destroy').length), 0);
    assert.equal(await page.evaluate(() => window.uiCalls.filter(call => call.command === 'plugin:event|listen' && call.args.event === 'tauri://close-requested').length), 0);
    await page.evaluate(() => { window.testAppState.isGameRunning = false; });
    }
    await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/settings');
    await page.locator('select').first().waitFor();
    await page.locator('select').first().selectOption('beta');
    await page.locator('select').nth(1).selectOption('8');
    assert.ok(await page.locator('select').first().evaluate(element => getComputedStyle(element).backgroundImage !== 'none'));
    const control = page.getByRole('switch').first();
    assert.equal(await control.evaluate(element => element.getBoundingClientRect().width), 44);
    const checked = await control.getAttribute('aria-checked'); await control.click(); assert.notEqual(await control.getAttribute('aria-checked'), checked);
    if (process.env.LUXMC_ARTIFACT_DIR) await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/launcher-settings.png` });
    assert.equal(await page.getByText('Carregando informações completas...').count(), 0);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ cards: 36, icon: bounds, nestedScroll, scrollTop, frameP95: frames[57], staleSearchIgnored: true, paginationResetsScroll: true, keyboardInstall: true, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
