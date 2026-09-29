const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, reducedMotion: 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        window.createdProfiles = [];
        const invoke = async (command, args) => {
            if (command === 'app_init') return { account: null, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'versions_list') return { versions: [{ id: '1.21.4', versionType: 'release', releaseTime: '2024-12-03' }], latestRelease: '1.21.4', latestSnapshot: '' };
            if (command === 'loaders_versions') return { versions: args.loader === 'neoforge'
                ? [{ id: '21.4.159-beta', stable: false }, { id: '21.4.158', stable: true }]
                : [{ id: '0.28.1', stable: true }] };
            if (command === 'profiles_create') {
                window.createdProfiles.push(args.input);
                return { ...args.input, id: 'created-' + window.createdProfiles.length, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), gameDir: '/tmp/luxmc-browser-instance', fullscreen: false };
            }
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'java_scan' || command === 'screenshots_list') return [];
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'optimizer_install_perf_pack') return [];
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
    });
    await page.goto('http://127.0.0.1:1420/instances');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: 'NeoForge', exact: true }).click();
    await page.getByRole('button', { name: 'Criar instância' }).isEnabled();
    await page.getByRole('button', { name: 'Criar instância' }).click();
    await page.waitForFunction(() => window.createdProfiles.length === 1);
    const created = await page.evaluate(() => window.createdProfiles[0]);
    assert.equal(created.loader, 'neoforge');
    assert.equal(created.loaderVersion, '21.4.158');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: 'NeoForge', exact: true }).click();
    await page.getByRole('button', { name: 'Última', exact: true }).click();
    await page.getByRole('button', { name: 'Criar instância' }).click();
    await page.waitForFunction(() => window.createdProfiles.length === 2);
    const latest = await page.evaluate(() => window.createdProfiles[1]);
    assert.equal(latest.loaderVersion, '21.4.159-beta');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ loader: created.loader, stable: created.loaderVersion, latest: latest.loaderVersion, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
