const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        for (const width of [1600, 960]) {
            const context = await browser.newContext({ viewport: { width, height: 1000 }, locale: 'pt-BR', reducedMotion: 'reduce' });
            const page = await context.newPage();
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await page.addInitScript(() => {
                const profile = { id: 'design-instance', name: 'Vanilla Perfected', icon: 'grass_block', mcVersion: '1.21.1', loader: 'fabric', loaderVersion: '0.16.9', gameDir: 'C:\\PRIVATE\\Minecraft\\design', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), ramMb: 4096 };
                window.designCalls = [];
                window.designStored = { language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false, soundscapesEnabled: false, soundEnabled: true, startFullscreen: true };
                const invoke = async (command, args = {}) => {
                    window.designCalls.push({ command, args });
                    if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' }, profiles: [profile], activeProfileId: profile.id, devMode: true };
                    if (command === 'profiles_list' || command === 'instances_list') return [profile];
                    if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                    if (command === 'plugin:store|get') return args.key === 'app' ? [window.designStored, true] : [null, false];
                    if (command === 'plugin:store|set') { if (args.key === 'app') window.designStored = args.value; return null; }
                    if (command === 'plugin:app|version') return '2.0.2';
                    if (command === 'java_scan') return { runtimes: [{ major: 21, installed: true, path: 'java.exe', versionString: '21' }] };
                    if (command === 'get_system_specs') return { osDistro: 'Windows 11', arch: 'x86_64', totalRamMb: 8192 };
                    if (command === 'keybinds_list') return { keybinds: [{ id: 'key_key.forward', label: 'Andar para frente', category: 'Movimento', rawKey: 'key.keyboard.w', displayKey: 'W', isConflict: false, conflictingWith: [] }], totalConflicts: 0 };
                    if (command === 'instance_options_get') return { gamma: 1, fov: 70, renderDistance: 12, simulationDistance: 12, maxFps: 120, guiScale: 0, fullscreen: false, vsync: false, autoJump: false, bobView: true, soundMaster: 1, soundMusic: 0.5 };
                    if (command === 'instance_shield_scan') return { isClean: true, threats: [] };
                    if (command === 'versions_check_installed') return true;
                    if (command === 'deep_links_take' || command === 'changelog_get' || command === 'instance_file_tree' || command === 'instances_screenshots' || command === 'instance_worlds_list') return [];
                    return null;
                };
                window.electronAPI = { invoke, on: () => () => {} };
                window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: value => value };
                window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
            });
            await page.route('https://**/*', route => route.fulfill({ status: 404, body: '{}' }));
            await page.goto('http://127.0.0.1:1420/settings');
            await page.getByRole('heading', { name: 'Seu launcher, do seu jeito', exact: true }).waitFor();
            assert.equal(await page.getByRole('tab').count(), 8);
            await page.getByRole('tab', { name: 'Geral', exact: true }).focus();
            await page.keyboard.press('ArrowRight');
            assert.equal(await page.getByRole('tab', { name: 'Contas', exact: true }).getAttribute('aria-selected'), 'true');
            await page.keyboard.press('Home');
            await page.getByRole('combobox', { name: 'Resolução do Jogo', exact: true }).selectOption('custom');
            await page.getByRole('spinbutton', { name: 'Largura da janela (px)', exact: true }).waitFor();
            await page.getByRole('combobox', { name: 'Resolução do Jogo', exact: true }).selectOption('1280x720');
            await page.waitForFunction(() => window.designStored.defaultResWidth === 1280 && window.designStored.defaultResHeight === 720 && window.designStored.startFullscreen === false);
            await page.getByRole('combobox', { name: 'Resolução do Jogo', exact: true }).selectOption('custom');
            await page.getByRole('spinbutton', { name: 'Largura da janela (px)', exact: true }).fill('1600');
            await page.getByRole('spinbutton', { name: 'Largura da janela (px)', exact: true }).press('Tab');
            await page.getByRole('spinbutton', { name: 'Altura da janela (px)', exact: true }).fill('900');
            await page.getByRole('spinbutton', { name: 'Altura da janela (px)', exact: true }).press('Tab');
            await page.waitForFunction(() => window.designStored.defaultResWidth === 1600 && window.designStored.defaultResHeight === 900);
            await page.getByRole('checkbox', { name: /Sons do launcher/ }).uncheck();
            await page.waitForFunction(() => window.designStored.soundEnabled === false);
            assert.equal(await page.getByRole('button', { name: 'Testar som', exact: true }).isDisabled(), true);
            await page.getByRole('checkbox', { name: /Sons do launcher/ }).check();
            await page.getByRole('slider', { name: /Volume dos efeitos/ }).evaluate(input => { input.value = '67'; input.dispatchEvent(new Event('input', { bubbles: true })); });
            await page.waitForFunction(() => window.designStored.sfxVolume === 0.67);
            assert.equal(await page.locator('main').evaluate(element => element.scrollWidth > element.clientWidth), false);
            fs.mkdirSync('docs/validation/design-settings', { recursive: true });
            await page.screenshot({ path: `docs/validation/design-settings/settings-${width}.png` });
            await page.getByRole('tab', { name: 'Java', exact: true }).click();
            await page.getByRole('button', { name: 'G1GC', exact: true }).click();
            await page.waitForFunction(() => window.designStored.jvmArgs === '-XX:+UseG1GC -XX:MaxGCPauseMillis=200');
            await page.getByRole('button', { name: 'Automático (recomendado)', exact: true }).click();
            await page.waitForFunction(() => window.designStored.jvmArgs === undefined);
            assert.equal(await page.getByRole('textbox', { name: 'Argumentos da JVM (Flags de Otimização)', exact: true }).inputValue(), '');
            await page.goto('http://127.0.0.1:1420/instances/design-instance');
            await page.getByRole('tab', { name: 'Configurações', exact: true }).click();
            const settings = page.getByRole('region', { name: 'Configurações da instância', exact: true });
            await settings.waitFor();
            assert.equal(await settings.getByRole('button', { name: /^Editor de controles|^Editor de configuração|^Jukebox/ }).count(), 0);
            await page.getByRole('button', { name: 'Ferramentas', exact: true }).click();
            const tools = page.locator('#instance-toolbox');
            await tools.getByRole('button', { name: /^Editor de controles/ }).click();
            await page.getByText('Andar para frente', { exact: true }).waitFor();
            await page.getByRole('button', { name: 'Fechar controles', exact: true }).click();
            await tools.getByRole('button', { name: /^Editor de configuração/ }).click();
            await page.getByRole('heading', { name: 'Editor Visual de Configurações', exact: true }).waitFor();
            await page.getByRole('button', { name: 'Cancelar', exact: true }).last().click();
            await tools.getByRole('button', { name: /^Jukebox/ }).click();
            const jukebox = page.getByRole('dialog', { name: 'Jukebox do Launcher — Discos e Sons', exact: true });
            await jukebox.waitFor();
            await jukebox.getByRole('button', { name: 'Fechar', exact: true }).last().click();
            await tools.getByRole('button', { name: 'Fechar', exact: true }).click();
            assert.equal(await tools.count(), 0);
            await page.getByRole('tab', { name: 'Configurações', exact: true }).waitFor();
            assert.equal(await page.getByRole('tab').count(), 8);
            assert.deepEqual(errors, []);
            console.log(JSON.stringify({ width, settingsTabs: 8, keyboard: true, resolutionSaved: '1600x900', soundToggleAndVolume: true, toolsMovedAndReachable: true, noExtraInstanceTabs: true, errors }));
            await context.close();
        }
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });

