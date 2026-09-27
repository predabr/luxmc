const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    const skin = `data:image/png;base64,${fs.readFileSync('static/alex.png').toString('base64')}`;
    await page.addInitScript(texture => {
        window.importPath = '/tmp/Alex MMC imported.png';
        window.importTexture = texture;
        window.testCalls = [];
        localStorage.setItem('luxmc.locale', 'pt-BR');
        if (!sessionStorage.getItem('skinImportFixtureReady')) {
            localStorage.setItem('luxmc_saved_skins', JSON.stringify([
                { id: 'default_steve', name: 'Steve', url: '/steve.png', model: 'steve' },
                { id: 'saved-steve-mmc', name: 'Steve MMC', url: texture, model: 'alex' }
            ]));
            sessionStorage.setItem('skinImportFixtureReady', '1');
        }
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Player', accessToken: '', skinUrl: 'https://textures.minecraft.net/texture/test', skinVariant: 'slim' };
        const invoke = async (command, args) => {
            window.testCalls.push({ command, args });
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'mods_search' || command === 'java_scan' || command === 'screenshots_list') return [];
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false }, true];
            if (command === 'plugin:dialog|open') return window.importPath;
            if (command === 'auth_read_local_texture') return window.importTexture;
            if (command === 'auth_resolve_texture') return window.importTexture;
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'mesh_status') return { available: false, state: '', ip: null, peers: [] };
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '' };
    }, skin);
    await page.route('https://textures.minecraft.net/**', route => route.abort());
    await page.goto('http://127.0.0.1:1420/skins');
    await page.waitForFunction(() => {
        const preview = document.querySelector('canvas[aria-label="Player"]');
        return preview && preview.getContext('2d').getImageData(0, 0, preview.width, preview.height).data.some((value, index) => index % 4 === 3 && value > 0);
    });
    await page.getByText('Steve MMC', { exact: true }).waitFor();
    let saved = await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')));
    assert.deepEqual(saved.map(item => item.id), ['saved-steve-mmc']);
    await page.waitForFunction(() => performance.getEntriesByType('resource').some(entry => /skinview3d.*\.js/.test(entry.name)));
    await page.evaluate(async () => {
        const url = performance.getEntriesByType('resource').find(entry => /skinview3d.*\.js/.test(entry.name)).name;
        const { SkinViewer } = await import(url);
        const original = SkinViewer.prototype.loadSkin;
        SkinViewer.prototype.loadSkin = function (...args) {
            window.importViewer = this;
            return original.apply(this, args);
        };
    });
    await page.getByRole('button', { name: /Adicionar skin/ }).click();
    await page.getByText('Alex MMC imported', { exact: true }).first().waitFor();
    await page.getByRole('region', { name: 'Visualizador 3D de Skin' }).locator('canvas').first().waitFor({ state: 'visible' });
    saved = await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')));
    assert.equal(saved.length, 2);
    assert.equal(saved[0].name, 'Alex MMC imported');
    assert.equal(saved[0].url, skin);
    assert.equal(await page.evaluate(() => localStorage.getItem('luxmc_selected_skin_id')), saved[0].id);
    const render = await page.evaluate(() => {
        const viewer = window.importViewer;
        if (!viewer) return null;
        const texture = viewer.playerObject.skin.map;
        const materials = new Set();
        viewer.playerObject.skin.traverse(object => {
            if (object.isMesh) for (const material of Array.isArray(object.material) ? object.material : [object.material]) {
                if (material.map === texture) materials.add(material);
            }
        });
        viewer.renderer.setRenderTarget(null);
        viewer.renderer.render(viewer.scene, viewer.camera);
        const gl = viewer.renderer.getContext();
        const pixels = new Uint8Array(gl.drawingBufferWidth * gl.drawingBufferHeight * 4);
        gl.readPixels(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
        let colored = 0;
        for (let i = 3; i < pixels.length; i += 4) if (pixels[i] > 0 && Math.max(pixels[i - 1], pixels[i - 2], pixels[i - 3]) > 15) colored++;
        return { colored, materials: materials.size, emissive: [...materials].every(material => material.emissiveMap === texture && material.emissiveIntensity === 1) };
    });
    assert.ok(render && render.colored > 100 && render.materials > 0 && render.emissive, JSON.stringify(render));
    await page.reload();
    await page.getByText('Alex MMC imported', { exact: true }).first().waitFor();
    assert.equal(await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')).length), 2);
    await page.getByRole('button', { name: 'Aplicar skin e capa' }).click();
    await page.waitForFunction(() => window.testCalls.some(call => call.command === 'auth_save_appearance'));
    const appliedSkin = await page.evaluate(() => window.testCalls.find(call => call.command === 'auth_save_appearance').args.skinUrl);
    assert.equal(appliedSkin, skin);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ imported: saved[0].name, preserved: saved[1].name, render, persistedAfterReload: true, appliedOriginalPng: true, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
