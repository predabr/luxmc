const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        for (const height of [900, 600]) {
            const page = await browser.newPage({ viewport: { width: 1440, height } });
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await setupLauncherDemo(page);
            await page.addInitScript(image => {
                window.contentCalls = [];
                window.worldFixtures = [];
                window.worldReads = 0;
                const old = window.electronAPI.invoke;
                let recovered = false;
                const entries = Array.from({ length: 1000 }, (_, index) => ({
                    name: index === 0 ? 'AccuracyCrosshair.zip' : `Mod-${String(index).padStart(4, '0')}.jar`,
                    path: `/mods/${index}.jar`, size: 10692, isDir: false,
                    icon: index === 0 ? 'https://media.forgecdn.net/avatars/fixture.jpeg' : null,
                    iconKey: `content:fixture:${index}`, iconResolved: index <= 1
                }));
                const invoke = async (command, args = {}) => {
                    if (command === 'instance_file_tree') return args.subPath === 'mods' ? entries.map((entry, index) => index === 0 && recovered ? { ...entry, icon: image } : entry) : [];
                    if (command === 'instance_content_icons') {
                        window.contentCalls.push(args);
                        if (args.cacheImages) recovered = true;
                        return args.fileNames.map(name => ({ name, icon: name === 'Mod-0001.jar' ? null : image, iconKey: entries.find(entry => entry.name === name).iconKey, resolved: true }));
                    }
                    if (command === 'instance_worlds_list') {
                        window.worldReads++;
                        await new Promise(resolve => setTimeout(resolve, 160));
                        return window.worldFixtures;
                    }
                    return old(command, args);
                };
                window.electronAPI.invoke = invoke;
                window.__TAURI_INTERNALS__.invoke = invoke;
            }, `data:image/png;base64,${fs.readFileSync('static/grass_head.png').toString('base64')}`);
            await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/instances/demo-vp');
            const list = page.locator('[data-virtual-list]');
            await list.locator('[data-index="0"] img').waitFor();
            await page.waitForFunction(() => {
                const image = document.querySelector('[data-index="0"] img');
                return image?.naturalWidth > 0 && getComputedStyle(image).visibility === 'visible' && image.src.startsWith('data:');
            });
            assert.equal(await page.evaluate(() => window.contentCalls.filter(call => call.cacheImages).length), 1);
            assert.equal(await list.locator('[data-index="1"] img').count(), 0);
            await page.evaluate(async () => {
                const source = performance.getEntriesByType('resource').find(entry => entry.name.includes('/src/lib/utils/contentPrefetch.ts')).name;
                const { rememberContentIcons } = await import(source);
                rememberContentIcons([{ iconKey: 'content:fixture:1', icon: '/grass_head.png', resolved: true }]);
            });
            await page.waitForFunction(() => document.querySelector('[data-index="1"] img')?.naturalWidth > 0);
            assert.equal(await page.evaluate(() => window.worldReads), 0, 'Opening mods does not recursively inspect worlds');
            assert.ok(await list.locator('[data-index]').count() < 25);
            await list.evaluate(node => { node.scrollTop = node.scrollHeight; });
            await page.getByRole('heading', { name: 'Mod 0999', exact: true }).waitFor();
            await list.evaluate(node => { node.scrollTop = 0; });
            await page.waitForFunction(() => document.querySelector('[data-index="0"] img')?.naturalWidth > 0);
            assert.equal(await page.evaluate(() => window.contentCalls.filter(call => call.cacheImages).length), 1, 'Recycled first row reuses recovery');
            const worlds = page.getByRole('tab', { name: 'Mundos', exact: true });
            await worlds.click();
            await page.waitForFunction(() => window.worldReads === 1);
            await page.locator('#instance-mundos [role="status"]').waitFor({ state: 'hidden' });
            assert.equal(await page.locator('#instance-mundos .world-card').count(), 0);
            await page.evaluate(() => { window.worldFixtures = [{ name: 'Meu mundo preservado', folderName: 'New World', sizeBytes: 15723316, lastPlayed: 1792421797, gameMode: 'Sobrevivência', iconBase64: null }]; });
            await page.getByRole('tab', { name: 'Mods', exact: true }).click();
            await worlds.click();
            await page.getByText('Meu mundo preservado', { exact: true }).waitFor();
            assert.equal(await page.locator('#instance-mundos .world-card').count(), 1);
            await page.evaluate(() => {
                window.worldFixtures.push({ ...window.worldFixtures[0], folderName: 'New World 2', name: 'Segundo mundo' });
                window.dispatchEvent(new Event('focus'));
            });
            await page.getByText('Segundo mundo', { exact: true }).waitFor();
            await page.getByRole('tab', { name: 'Datapacks', exact: true }).click();
            assert.equal(await page.getByRole('combobox', { name: 'Mundos', exact: true }).locator('option').count(), 2);
            await page.getByRole('tab', { name: 'Mundos', exact: true }).click();
            await page.evaluate(async () => {
                const { appState } = await import('/src/lib/stores/app.svelte.ts');
                appState.isGameRunning = true;
                await new Promise(resolve => setTimeout(resolve, 50));
                window.worldFixtures.push({ ...window.worldFixtures[0], folderName: 'New World 3', name: 'Criado durante o jogo' });
                appState.isGameRunning = false;
            });
            await page.getByText('Criado durante o jogo', { exact: true }).waitFor();
            assert.deepEqual(errors, []);
            console.log(JSON.stringify({ height, firstRowRecovered: true, virtualRowsBounded: true, emptyWorldListRefreshed: true, focusRefresh: true, gameExitRefresh: true }));
            await page.close();
        }
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
