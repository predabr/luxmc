const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' };
        window.searchCalls = [];
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
        const invoke = async (command, args) => {
            if (command === 'mods_search') {
                window.searchCalls.push(args);
                return results;
            }
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'java_scan' || command === 'screenshots_list') return [];
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'curseforge_status') return true;
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'mesh_status') return { available: false, state: '', ip: null, peers: [] };
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
    });
    await page.goto('http://127.0.0.1:1420/mods');
    await page.getByText('Pack 35', { exact: true }).waitFor();
    assert.equal(await page.locator('[data-virtual-list]').count(), 0);
    assert.equal(await page.getByText(/^Pack \d+$/).count(), 36);
    assert.equal(await page.evaluate(() => window.searchCalls.at(-1)?.limit), 36);
    const icon = page.locator('img[alt="Pack 0"]');
    const bounds = await icon.boundingBox();
    assert.ok(bounds && bounds.height >= 140 && bounds.width >= 140, JSON.stringify(bounds));
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
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ cards: 36, icon: bounds, nestedScroll, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
