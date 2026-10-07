const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const page = await browser.newPage({ viewport: { width: 1480, height: 1000 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    const skin = `data:image/png;base64,${fs.readFileSync('static/alex.png').toString('base64')}`;
    await page.addInitScript(texture => {
        window.calls = [];
        window.nameMcListeners = new Set();
        localStorage.setItem('luxmc.locale', 'pt-BR');
        localStorage.setItem('luxmc_saved_skins', JSON.stringify([{ id: 'saved-steve', name: 'Steve salvo', url: '/steve.png', model: 'steve' }]));
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Player', accessToken: '', skinUrl: '/steve.png', skinVariant: 'classic' };
        const invoke = async (command, args) => {
            window.calls.push({ command, args });
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false, language: 'pt-BR', languageMode: 'manual' }, true];
            if (command === 'namemc_import_skin') {
                if (window.failNameMc) throw Error('NameMC temporarily unavailable');
                return { skinId: '63455d7069b397c2', skinUrl: texture };
            }
            if (['deep_links_take', 'profiles_list', 'instances_list', 'screenshots_list', 'changelog_get'].includes(command)) return [];
            if (command === 'java_scan') return { runtimes: [] };
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Windows 11', arch: 'x86_64' };
            return null;
        };
        window.electronAPI = {
            invoke,
            on: (event, callback) => {
                if (event !== 'namemc-skin-selected') return () => {};
                window.nameMcListeners.add(callback);
                return () => window.nameMcListeners.delete(callback);
            }
        };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '' };
    }, skin);
    await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/skins');
    await page.getByRole('button', { name: 'Explorar NameMC', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'Escolha sua próxima skin' });
    await page.waitForFunction(() => window.calls.some(call => call.command === 'namemc_open_picker'));
    assert.equal(await dialog.count(), 0);
    await page.evaluate(() => {
        for (const callback of window.nameMcListeners) callback('https://namemc.com/skin/63455d7069b397c2');
    });
    await dialog.waitFor({ state: 'hidden' });
    await page.getByText('NameMC 63455d7069b397c2', { exact: true }).waitFor();
    const saved = await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')));
    assert.equal(saved.length, 2);
    assert.equal(saved[0].url, skin);
    assert.equal(saved[1].name, 'Steve salvo');
    await page.locator('main p[role="status"]').filter({hasText:'Alterações ainda não aplicadas. Clique em Aplicar e sincronizar'}).waitFor();
    assert.equal(await page.getByRole('button', { name: 'Capa Luxmc Oficial', exact: true }).count(), 0);
    assert.equal(await page.evaluate(() => window.calls.some(call => call.command === 'auth_change_skin' || call.command === 'auth_save_appearance')), false);
    assert.equal(await page.evaluate(() => window.nameMcListeners.size), 0);
    assert.ok(await page.evaluate(() => window.calls.some(call => call.command === 'namemc_close_picker')));
    await page.getByRole('button', { name: 'Link da skin no NameMC', exact: true }).click();
    await dialog.waitFor();
    await dialog.getByLabel('Link da skin no NameMC').fill('https://pt.namemc.com/skin/63455d7069b397c2');
    await page.evaluate(() => { window.failNameMc = true; });
    await dialog.getByRole('button', { name: 'Importar skin', exact: true }).click();
    await dialog.getByRole('alert').waitFor();
    assert.equal(await dialog.getByRole('button', { name: 'Importar skin', exact: true }).isEnabled(), true);
    await page.evaluate(() => { window.failNameMc = false; });
    await dialog.getByRole('button', { name: 'Importar skin', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    assert.equal(await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')).length), 2);
    assert.deepEqual(errors, []);
    fs.mkdirSync('docs/validation', { recursive: true });
    await page.screenshot({ path: 'docs/validation/skins-studio-design.png' });
    console.log('NameMC native selection and pasted links import original PNG without applying the account, preserve saved skins, deduplicate, clean listeners, and recover from import failure. Generated cape catalog removed.');
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
