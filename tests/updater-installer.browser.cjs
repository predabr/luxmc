const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        page.setDefaultTimeout(60000);
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await page.addInitScript(() => {
            const invoke = async (command, args) => {
                if (command === 'app_init') return { account: null, profiles: [], activeProfileId: null };
                if (command === 'profiles_list' || command === 'deep_links_take' || command === 'changelog_get') return [];
                if (command === 'plugin:app|version') return '2.0.2';
                if (command === 'app_update_environment') return { mode: 'windows' };
                if (command === 'app_perform_update') {
                    window.updateAttempts = (window.updateAttempts || 0) + 1;
                    if (window.updateAttempts === 1) throw new Error('Feche o Minecraft antes de atualizar o Luxmc.');
                    window.selectedUpdateUrl = args.downloadUrl;
                    return { action: 'downloaded', terminalCommand: null };
                }
                if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                if (command === 'plugin:store|get') return [{ language: 'pt-BR', autoCheckUpdates: false }, true];
                if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64' };
                if (command === 'java_scan') return { runtimes: [] };
                return null;
            };
            window.electronAPI = { invoke, on: () => () => {} };
            window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
            window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
        });
        let release = null;
        let updateRequests = 0;
        await page.route('https://**/*', route => {
            if (/github\.com.*(?:releases|latest\.json)/.test(route.request().url())) updateRequests++;
            if (route.request().url().includes('api.github.com/repos/predabr/luxmc/releases/latest') && release) return route.fulfill({ json: release });
            return route.fulfill({ status: 404, body: '{}' });
        });
        await page.goto('http://127.0.0.1:1420/settings');
        await page.getByText('Versão instalada', { exact: true }).waitFor();
        await page.waitForTimeout(3500);
        assert.equal(updateRequests, 0, 'automatic checking must respect the disabled setting');
        assert.equal(await page.getByText('Sistema Atualizado', { exact: true }).count(), 0);
        await page.getByRole('button', { name: 'Verificar Atualizações', exact: true }).click();
        await page.getByText('Verificação indisponível', { exact: true }).waitFor();
        assert.equal(await page.getByText('Sistema Atualizado', { exact: true }).count(), 0);
        release = { tag_name: 'v2.0.2', assets: [] };
        await page.getByRole('button', { name: 'Verificar Atualizações', exact: true }).click();
        await page.getByText('Sistema Atualizado', { exact: true }).waitFor();
        release = { tag_name: 'v2.0.3', assets: [
            { name: 'Luxmc-x64.exe', browser_download_url: 'https://github.com/predabr/luxmc/releases/download/v2.0.3/Luxmc-x64.exe', size: 1 },
            { name: 'Lux MC Launcher.exe', browser_download_url: 'https://github.com/predabr/luxmc/releases/download/v2.0.3/Lux%20MC%20Launcher.exe', size: 1 }
        ] };
        await page.getByRole('button', { name: 'Verificar Atualizações', exact: true }).click();
        await page.getByRole('button', { name: 'Atualizar Agora Automaticamente', exact: true }).click();
        await page.getByText('Não foi possível concluir a atualização', { exact: true }).waitFor();
        await page.getByRole('button', { name: 'Tentar atualizar novamente', exact: true }).click();
        await page.waitForFunction(() => window.selectedUpdateUrl !== undefined);
        assert.ok((await page.evaluate(() => window.selectedUpdateUrl)).endsWith('/Lux%20MC%20Launcher.exe'));
        assert.deepEqual(errors, []);
        console.log('Updater: disabled automatic checking respected; failed checks never claim current; installer selected; game-open error allows retry.');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
