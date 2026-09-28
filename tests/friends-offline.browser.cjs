const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        const account = { id: 'luxmc:test', uuid: 'test', username: 'Steve', accessToken: '' };
        const invoke = async (command, args) => {
            if (command === 'app_init') return { account, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, soundEnabled: false }, true];
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'mods_search' || command === 'java_scan' || command === 'screenshots_list') return [];
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'p2p_scan_lan_worlds') return [{ motd: 'Mundo LAN', port: 25565, host: '192.168.1.10' }];
            if (command === 'tunnel_status') return null;
            if (command === 'social_request') throw new Error('Invalid state: Serviço social temporariamente indisponível.');
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '/grass_head.png' };
    });

    await page.goto('http://127.0.0.1:1420/friends');
    await page.getByText('Modo Local / P2P Ativo').waitFor({ timeout: 30000 });
    await page.getByRole('heading', { name: 'Seu mundo, um convite' }).waitFor();
    await page.getByRole('button', { name: 'Hospedar meu mundo' }).waitFor();
    await page.getByText('Mundos LAN detectados').waitFor();
    await page.waitForFunction(() => {
        const label = [...document.querySelectorAll('p')].find(element => element.textContent === 'Mundos LAN detectados');
        return label?.parentElement?.textContent?.includes('1');
    });
    assert.equal(await page.getByText('Invalid state: Serviço social temporariamente indisponível.').count(), 0);
    assert.deepEqual(errors, []);
    if (process.env.LUXMC_BROWSER_SCREENSHOT) await page.screenshot({ path: process.env.LUXMC_BROWSER_SCREENSHOT, fullPage: true });
    console.log(JSON.stringify({ localMode: true, p2pVisible: true, lanWorlds: 1, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
