const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.worldFixtures = [];
            window.worldReads = 0;
            window.fileReads = [];
            window.pendingWorldReads = [];
            window.pendingFileReads = [];
            const old = window.electronAPI.invoke;
            const invoke = async (command, args = {}) => {
                if (command === 'instance_worlds_list') {
                    window.worldReads++;
                    const worlds = [...window.worldFixtures];
                    if (window.pauseWorldReads) await new Promise(resolve => window.pendingWorldReads.push(resolve));
                    return worlds;
                }
                if (command === 'instance_file_tree') {
                    window.fileReads.push(args.subPath || '');
                    if (args.subPath === 'slow') {
                        await new Promise(resolve => window.pendingFileReads.push(resolve));
                        return [{ name: 'Stale-file.txt', path: '/slow/Stale-file.txt', isDir: false, size: 1 }];
                    }
                    return args.subPath ? [] : [{ name: 'slow', path: '/slow', isDir: true, size: 1 }];
                }
                return old(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/instances/demo-vp');
        await page.getByRole('tab', { name: 'Mundos', exact: true }).click();
        await page.waitForFunction(() => window.worldReads === 1);
        await page.locator('#instance-mundos [role="status"]').waitFor({ state: 'hidden' });
        await page.evaluate(() => {
            window.pauseWorldReads = true;
            window.worldFixtures = [{ name: 'Primeiro mundo', folderName: 'First', sizeBytes: 1, lastPlayed: 1 }];
            window.dispatchEvent(new Event('focus'));
        });
        await page.waitForFunction(() => window.pendingWorldReads.length === 1);
        await page.evaluate(() => {
            window.worldFixtures.push({ name: 'Mundo criado durante leitura', folderName: 'Second', sizeBytes: 1, lastPlayed: 1 });
            window.dispatchEvent(new Event('focus'));
            window.pauseWorldReads = false;
            window.pendingWorldReads.splice(0).forEach(resolve => resolve());
        });
        await page.getByText('Mundo criado durante leitura', { exact: true }).waitFor({ timeout: 5000 });
        assert.equal(await page.evaluate(() => window.worldReads), 3);
        await page.getByRole('tab', { name: 'Arquivos', exact: true }).click();
        const files = page.locator('#instance-ficheiros');
        await files.getByRole('button', { name: 'slow', exact: false }).first().click();
        await page.waitForFunction(() => window.pendingFileReads.length > 0);
        await files.getByRole('button', { name: '~ raiz', exact: true }).click();
        await page.evaluate(() => window.pendingFileReads.splice(0).forEach(resolve => resolve()));
        await page.waitForTimeout(200);
        assert.equal(await files.getByText('Stale-file.txt', { exact: true }).count(), 0, 'A delayed folder read must not replace the current folder');
        assert.equal(await page.evaluate(() => window.fileReads.filter(path => path === 'slow').length), 1, 'A folder navigation must perform one read');
        await files.getByRole('button', { name: 'slow', exact: false }).first().waitFor();
        assert.deepEqual(errors, []);
        console.log(JSON.stringify({ queuedWorldRefresh: true, staleFolderReadIgnored: true, singleFolderRead: true, errors }));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
