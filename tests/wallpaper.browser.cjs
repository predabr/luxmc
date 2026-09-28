const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    const poster = `data:image/png;base64,${fs.readFileSync('static/steve.png').toString('base64')}`;
    await page.addInitScript(image => {
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' };
        window.wallpaperPoster = image;
        window.nextWallpaperPath = '/tmp/wallpaper-one.mp4';
        window.testCalls = [];
        const invoke = async (command, args) => {
            window.testCalls.push({ command, args });
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'mods_search' || command === 'java_scan' || command === 'screenshots_list') return [];
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'plugin:dialog|open') return window.nextWallpaperPath;
            if (command === 'wallpaper_prepare_poster') return window.wallpaperPoster;
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'mesh_status') return { available: false, state: '', ip: null, peers: [] };
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => `http://asset.localhost/${encodeURIComponent(path)}` };
    }, poster);
    await page.route('http://asset.localhost/**', route => route.fulfill({ status: 200, contentType: 'video/mp4', body: fs.readFileSync('tests/fixtures/wallpaper.mp4') }));
    await page.goto('http://127.0.0.1:1420/');
    await page.evaluate(async () => {
        const { themeStore } = await import('/src/lib/stores/theme.svelte.ts');
        themeStore.setCustomWallpaper('/tmp/wallpaper-one.mp4', 'video');
    });
    const video = page.locator('video').first();
    await page.waitForFunction(() => {
        const element = document.querySelector('video');
        const image = element?.parentElement?.querySelector('img');
        return element?.readyState >= 2 && image?.complete && image.naturalWidth > 0;
    });
    assert.equal(await video.evaluate(element => element.parentElement.querySelectorAll('canvas').length), 0);
    await video.evaluate(element => {
        window.testLoops = 0;
        let previous = element.currentTime;
        element.addEventListener('timeupdate', () => {
            if (element.currentTime + 0.5 < previous) window.testLoops++;
            previous = element.currentTime;
        });
    });
    await page.waitForFunction(() => window.testLoops >= 3, {}, { timeout: 30000 });
    await video.evaluate(element => { element.pause(); element.dispatchEvent(new Event('seeking')); element.dispatchEvent(new Event('waiting')); });
    assert.equal(await video.evaluate(element => getComputedStyle(element).opacity), '0');
    assert.ok(await video.evaluate(element => element.parentElement.querySelector('img').naturalWidth > 0));
    await page.evaluate(async () => { const { appState } = await import('/src/lib/stores/app.svelte.ts'); appState.isGameRunning = true; });
    await page.waitForFunction(() => document.querySelector('video').paused);
    await page.evaluate(async () => { const { appState } = await import('/src/lib/stores/app.svelte.ts'); appState.isGameRunning = false; });
    await page.waitForFunction(() => !document.querySelector('video').paused);

    await page.locator('a[href="/settings"]').first().click();
    await page.getByRole('button', { name: /^(Aparência|Appearance)/ }).click();
    await page.evaluate(() => { window.nextWallpaperPath = '/tmp/wallpaper-two.mp4'; });
    await page.getByRole('button', { name: /^(?:\+ )?(?:Importar|Import)$/ }).click();
    await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper one' }).waitFor();
    await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper two' }).waitFor();
    await page.waitForFunction(() => [...document.querySelectorAll('button[aria-label^="Selecionar wallpaper"] img')].filter(image => image.complete && image.naturalWidth > 0).length === 2);
    await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper one' }).click();
    assert.equal(await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.customWallpaperUrl), '/tmp/wallpaper-one.mp4');
    await page.reload();
    await page.getByRole('button', { name: /^(Aparência|Appearance)/ }).click();
    await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper one' }).waitFor();
    await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper two' }).waitFor();
    const library = await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.wallpaperLibrary.map(item => item.url));
    assert.deepEqual(library.sort(), ['/tmp/wallpaper-one.mp4', '/tmp/wallpaper-two.mp4']);
    await page.getByRole('button', { name: 'Remover wallpaper one da lista' }).click();
    assert.equal(await page.evaluate(async () => (await import('/src/lib/stores/theme.svelte.ts')).themeStore.customWallpaperUrl), '/tmp/wallpaper-two.mp4');
    assert.equal(await page.getByRole('button', { name: 'Selecionar wallpaper wallpaper one' }).count(), 0);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ loops: 3, posterDuringSeek: true, library, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
