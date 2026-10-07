const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.addInitScript(() => {
            const defaults = { language: 'pt-BR', languageMode: 'manual', pauseWallpaperOnBlur: false, customBackground: 'obsidian' };
            window.mediaCalls = 0;
            const invoke = async (command, args) => {
                if (command === 'plugin:store|load') return 1;
                if (command === 'plugin:store|get') {
                    if(localStorage.getItem('fixture-store-unavailable'))throw Error('Temporary store failure');
                    return [JSON.parse(localStorage.getItem('fixture-settings') || JSON.stringify(defaults)), true];
                }
                if (command === 'settings_get') return JSON.parse(localStorage.getItem('fixture-settings') || JSON.stringify(defaults));
                if (command === 'plugin:store|set') { localStorage.setItem('fixture-settings', JSON.stringify(args.value)); return null; }
                if (command === 'app_init') return { account: { id: 'offline_test', username: 'Steve', uuid: 'offline_test', accessToken: '' }, profiles: [], devMode: true };
                if (command === 'profiles_list' || command === 'changelog_get' || command === 'deep_links_take') return [];
                if (command === 'media_server_port') return ++window.mediaCalls < 3 ? null : Number(localStorage.getItem('fixture-port') || 59001);
                if (command === 'wallpaper_prepare_poster') return '';
                if (command === 'wallpaper_import') return args.path;
                if (command === 'plugin:dialog|open') return 'C:\\Luxmc\\wallpapers\\scene.mp4';
                if (command === 'java_scan') return { runtimes: [] };
                return null;
            };
            window.electronAPI = { invoke, on: () => () => {} };
            window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '/grass_block.png' };
        });
        const video = fs.readFileSync('website/assets/launcher-personalizer.mp4');
        await page.route(/http:\/\/127\.0\.0\.1:59\d+\/media\?/, async route => {
            const range = route.request().headers().range;
            const match = range?.match(/bytes=(\d+)-(\d*)/);
            if (match) {
                const start = Number(match[1]);
                const end = match[2] ? Math.min(Number(match[2]), video.length - 1) : video.length - 1;
                await route.fulfill({ status: 206, contentType: 'video/mp4', headers: { 'Access-Control-Allow-Origin': '*', 'Accept-Ranges': 'bytes', 'Content-Range': `bytes ${start}-${end}/${video.length}` }, body: video.subarray(start, end + 1) });
            } else await route.fulfill({ contentType: 'video/mp4', headers: { 'Access-Control-Allow-Origin': '*', 'Accept-Ranges': 'bytes' }, body: video });
        });
        await page.route('https://**/*', route => route.abort());
        await page.goto('http://127.0.0.1:1420/settings');
        await page.locator('aside nav a[href="/mods"]').waitFor();
        await page.getByRole('tab', { name: /Aparência/ }).click();
        await page.getByRole('button', { name: '+ Importar', exact: true }).click();
        try {
            await page.waitForFunction(() => Array.from(document.querySelectorAll('video')).some(video => video.currentTime > 0.3 && !video.paused), null, { timeout: 30000 });
        } catch (error) {
            console.log(await page.evaluate(() => ({ mediaCalls: window.mediaCalls, videos: Array.from(document.querySelectorAll('video')).map(video => ({ src: video.currentSrc, time: video.currentTime, paused: video.paused, error: video.error?.message })) })));
            throw error;
        }
        assert.ok(await page.evaluate(() => window.mediaCalls >= 3));
        await page.evaluate(() => {
            localStorage.setItem('fixture-port', '59002');
            for (const key of Object.keys(localStorage)) if (key.startsWith('luxmc_')) localStorage.removeItem(key);
        });
        await page.reload();
        try {
            await page.waitForFunction(() => Array.from(document.querySelectorAll('video')).some(video => video.currentSrc.includes(':59002/media?') && video.currentTime > 0.3 && !video.paused), null, { timeout: 20000 });
        } catch (error) {
            console.log(await page.evaluate(async () => ({ saved: JSON.parse(localStorage.getItem('fixture-settings')), mediaCalls: window.mediaCalls, videos: Array.from(document.querySelectorAll('video')).map(video => ({ src: video.currentSrc, time: video.currentTime, paused: video.paused, error: video.error?.message })), theme: (await import('/src/lib/stores/theme.svelte.ts')).themeStore.background, errors: [] })));
            throw error;
        }
        assert.equal(await page.evaluate(() => JSON.parse(localStorage.getItem('fixture-settings')).customWallpaperUrl), 'C:\\Luxmc\\wallpapers\\scene.mp4');
        await page.evaluate(() => {
            localStorage.setItem('fixture-store-unavailable','true');
            for (const key of Object.keys(localStorage)) if (key.startsWith('luxmc_')) localStorage.removeItem(key);
        });
        await page.reload();
        await page.waitForFunction(() => Array.from(document.querySelectorAll('video')).some(video => video.currentSrc.includes(':59002/media?') && video.currentTime > 0.3 && !video.paused), null, { timeout: 20000 });
        assert.deepEqual(errors, []);
        fs.mkdirSync('docs/validation/current-patch', { recursive: true });
        await page.screenshot({ path: 'docs/validation/current-patch/wallpaper-restarted.png' });
        await page.locator('aside button[title^="Meu Perfil"]').click();
        const profile = page.getByRole('dialog', { name: 'Seu perfil' });
        await profile.getByText('Conta local', { exact: true }).waitFor();
        await profile.getByText(/a personalização fica neste computador/).waitFor();
        await page.screenshot({ path: 'docs/validation/current-patch/profile-wallpaper.png' });
        await page.setViewportSize({ width: 960, height: 700 });
        assert.ok(await profile.getByRole('button', { name: 'Copiar identificação' }).isVisible());
        await page.screenshot({ path: 'docs/validation/current-patch/profile-compact.png' });
        await page.keyboard.press('Escape');
        await profile.waitFor({ state: 'detached' });
        console.log('Animated wallpaper: actual playback, delayed server, persisted settings, empty local cache and changed server port after restart passed');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
