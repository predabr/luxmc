const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ locale: 'en-US', viewport: { width: 1440, height: 900 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    const skin = `data:image/png;base64,${fs.readFileSync('static/alex.png').toString('base64')}`;
    await page.addInitScript(texture => {
        performance.setResourceTimingBufferSize(5000);
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
            if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false }, true];
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
    await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/skins');
    await page.getByRole('region', { name: 'Visualizador 3D de Skin' }).locator('canvas').first().waitFor({ state: 'visible' });
    await page.getByText('Steve MMC', { exact: true }).waitFor();
    let saved = await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')));
    assert.deepEqual(saved.map(item => item.id), ['saved-steve-mmc']);
    await page.getByRole('region', { name: 'Visualizador 3D de Skin' }).locator('canvas').first().waitFor({ state: 'visible' });
    await page.evaluate(async () => {
        const urls = performance.getEntriesByType('resource').map(entry => entry.name).filter(name => /\.js(?:\?|$)/.test(name));
        let SkinViewer;
        for (const url of urls) {
            const module = await import(url);
            SkinViewer = Object.values(module).find(value => typeof value === 'function' && typeof value.prototype?.loadSkin === 'function');
            if (SkinViewer) break;
        }
        if (!SkinViewer) throw new Error('SkinViewer não encontrado nos módulos carregados');
        const original = SkinViewer.prototype.loadSkin;
        SkinViewer.prototype.loadSkin = function (...args) { window.importViewer = this; return original.apply(this, args); };
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
        const source = texture.image.getContext('2d').getImageData(0, 0, texture.image.width, texture.image.height).data;
        const palette = new Set();
        for (let i=0;i<source.length;i+=4) if(source[i+3] === 255) palette.add(`${source[i]},${source[i+1]},${source[i+2]}`);
        let faithfulPixels = 0, opaquePixels = 0;
        for (let i=0;i<pixels.length;i+=4) if(pixels[i+3] === 255) { opaquePixels++; if(palette.has(`${pixels[i]},${pixels[i+1]},${pixels[i+2]}`)) faithfulPixels++; }
        let colored = 0;
        for (let i = 3; i < pixels.length; i += 4) if (pixels[i] > 0 && Math.max(pixels[i - 1], pixels[i - 2], pixels[i - 3]) > 15) colored++;
        return { colored, faithfulPixels, opaquePixels, materials: materials.size, emissive: [...materials].every(material => material.emissiveMap === texture && material.emissiveIntensity === 1) };
    });
    assert.ok(render && render.colored > 100 && render.materials > 0 && render.emissive, JSON.stringify(render));
    assert.ok(render.faithfulPixels / render.opaquePixels > 0.8, `Cores alteradas: ${JSON.stringify(render)}`);
    assert.ok(await page.getByRole('button', { name: 'Web Studio', exact: true }).evaluate(element => element.clientWidth > 90));
    assert.equal(await page.getByRole('button', { name: 'Capa Luxmc Oficial', exact: true }).count(), 0);
    await page.evaluate(async () => {
        const { getFullCapeDataUrl } = await import('/src/lib/utils/capeTextures.ts');
        window.importPath = 'C:/fixture/cape.png';
        window.importTexture = getFullCapeDataUrl('luxmc');
    });
    const importCape = page.getByRole('button', { name: 'Importar capa PNG', exact: true });
    await importCape.click();
    await page.getByText('Sua capa atual', {exact:true}).waitFor();
    await page.waitForFunction(() => window.importViewer?.playerObject.cape.visible);
    await page.evaluate(() => {
        const viewer = window.importViewer;
        const load = viewer.loadCape;
        viewer.loadCape = function(...args) { this.loadCape = load; throw new Error('Capa indisponível'); };
        window.importTexture = document.createElement('canvas').toDataURL();
    });
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.getByRole('button', { name: 'Capa importada', exact: true }).click();
    await page.getByText('Não foi possível carregar a capa. Selecione outra ou importe o PNG novamente.').waitFor();
    assert.equal(await page.evaluate(() => window.importViewer.playerObject.cape.visible), false);
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.getByRole('button', { name: 'Capa importada', exact: true }).click();
    await page.waitForFunction(() => window.importViewer.playerObject.cape.visible);
    assert.equal(await page.getByText('Não foi possível carregar a capa. Selecione outra ou importe o PNG novamente.').count(), 0);
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.waitForFunction(() => !window.importViewer.playerObject.cape.visible);
    const canvas = page.getByRole('region', { name: 'Visualizador 3D de Skin' }).locator('canvas');
    await canvas.hover();
    const rotateButton = page.getByTitle('Ativar rotação', { exact: true });
    if (await rotateButton.count()) await rotateButton.click();
    const animatedDuringScroll = await page.evaluate(async () => {
        const start = window.importViewer.playerWrapper.rotation.y;
        const root = document.querySelector('main');
        for (let index = 0; index < 20; index++) { root.dispatchEvent(new WheelEvent('wheel', { bubbles: true, deltaY: 6 })); await new Promise(requestAnimationFrame); }
        return Math.abs(window.importViewer.playerWrapper.rotation.y - start) > 0.01;
    });
    assert.equal(animatedDuringScroll, true, 'personagem deve continuar animando durante a rolagem');
    const animatedDuringGame = await page.evaluate(async () => {
        const { appState } = await import('/src/lib/stores/app.svelte.ts');
        appState.isGameRunning = true;
        await new Promise(resolve => setTimeout(resolve, 100));
        const start = window.importViewer.playerWrapper.rotation.y;
        await new Promise(resolve => setTimeout(resolve, 600));
        const moved = Math.abs(window.importViewer.playerWrapper.rotation.y - start) > 0.01;
        appState.isGameRunning = false;
        return moved;
    });
    assert.equal(animatedDuringGame, true, 'personagem deve continuar animando com Minecraft aberto');

    const beforeScroll = await page.locator('main').evaluate(element => element.scrollTop);
    await page.mouse.wheel(0, 380);
    await page.waitForFunction(before => document.querySelector('main').scrollTop > before, beforeScroll);
    await page.waitForFunction(() => !document.querySelector('main').classList.contains('is-scrolling'));
    assert.equal(await canvas.evaluate(element => getComputedStyle(element).touchAction), 'pan-y');
    await page.locator('main').evaluate(element => { element.scrollTop = 0; });
    await page.getByRole('button', { name: 'Sem capa', exact: true }).click();
    await page.reload();
    await page.getByText('Alex MMC imported', { exact: true }).first().waitFor();
    assert.equal(await page.evaluate(() => JSON.parse(localStorage.getItem('luxmc_saved_skins')).length), 2);
    await page.getByRole('button', { name: 'Aplicar skin e capa' }).click();
    await page.waitForFunction(() => window.testCalls.some(call => call.command === 'auth_save_appearance'));
    const appliedSkin = await page.evaluate(() => window.testCalls.find(call => call.command === 'auth_save_appearance').args.skinUrl);
    assert.equal(appliedSkin, skin);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ imported: saved[0].name, preserved: saved[1].name, render, persistedAfterReload: true, appliedOriginalPng: true, builtinCapeCatalogRemoved: true, importedCapePreserved: true, capeErrorRecovery: true, nativeWheelOverPreview: true, animatedDuringScroll, animatedDuringGame, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });

