const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const scenarios = [
        { system: 'pt-PT', browser: 'en-US', expected: 'pt-BR' },
        { system: 'es-MX', browser: 'en-US', expected: 'es' },
        { system: 'de-DE', browser: 'pt-BR', expected: 'en' },
        { system: 'pt-BR', browser: 'pt-BR', stored: { language: 'en', languageMode: 'manual' }, expected: 'en' },
        { system: 'en-US', browser: 'en-US', stored: { language: 'es' }, expected: 'es' },
        { system: 'fr-FR', browser: 'pt-BR', stored: { language: 'pt-BR', languageMode: 'system' }, expected: 'en' },
        { system: null, browser: 'pt-PT', expected: 'pt-BR' }
    ];
    try {
        for (const scenario of scenarios) {
            const context = await browser.newContext({ locale: scenario.browser, viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
            const page = await context.newPage();
            page.setDefaultTimeout(30000);
            const errors = [];
            page.on('pageerror', error => errors.push(error.message));
            await page.addInitScript(({ system, stored }) => {
                window.localeCalls = [];
                window.persistedSettings = null;
                const invoke = async (command, args) => {
                    window.localeCalls.push(command);
                    if (command === 'app_system_locale') return system;
                    if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' }, profiles: [], activeProfileId: null, devMode: true };
                    if (command === 'profiles_list' || command === 'deep_links_take' || command === 'changelog_get') return [];
                    if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                    if (command === 'plugin:store|get') return args.key === 'app' ? [stored || null, Boolean(stored)] : [null, false];
                    if (command === 'plugin:store|set') { if (args.key === 'app') window.persistedSettings = args.value; return null; }
                    if (command === 'plugin:app|version') return '2.0.2';
                    if (command === 'java_scan') return { runtimes: [] };
                    if (command === 'get_system_specs') return { osDistro: 'Windows 11', arch: 'x86_64', totalRamMb: 8192 };
                    return null;
                };
                window.electronAPI = { invoke, on: () => () => {} };
                window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: value => value };
                window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
            }, scenario);
            await page.route('https://**/*', route => route.fulfill({ status: 404, body: '{}' }));
            await page.goto('http://127.0.0.1:1420/settings');
            await page.waitForFunction(locale => document.documentElement.lang === locale, scenario.expected);
            const label = { en: 'Updates, audio and diagnostics', es: 'Actualizaciones, audio y diagnóstico', 'pt-BR': 'Atualizações, áudio e diagnóstico' }[scenario.expected];
            await page.getByRole('heading', { name: label, exact: true }).waitFor();
            if (scenario.stored && scenario.stored.languageMode !== 'system') assert.equal(await page.evaluate(() => window.localeCalls.includes('app_system_locale')), false);
            if (scenario.expected === 'en') {
                if (process.env.LUXMC_ARTIFACT_DIR) {
                    fs.mkdirSync(process.env.LUXMC_ARTIFACT_DIR, { recursive: true });
                    await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/languages-settings-en.png` });
                }
                await page.getByRole('tab', { name: 'Language', exact: true }).click();
                await page.getByRole('button', { name: /Español/ }).click();
                await page.waitForFunction(() => document.documentElement.lang === 'es');
                await page.getByRole('heading', { name: 'Más opciones', exact: true }).waitFor();
                await page.waitForFunction(() => window.persistedSettings?.language === 'es' && window.persistedSettings?.languageMode === 'manual');
                await page.getByRole('button', { name: 'Usar el idioma del sistema', exact: true }).click();
                await page.waitForFunction(locale => document.documentElement.lang === locale, scenario.system.startsWith('pt') ? 'pt-BR' : 'en');
                await page.waitForFunction(() => window.persistedSettings?.languageMode === 'system');
            }
            assert.deepEqual(errors, [], JSON.stringify(scenario));
            console.log(JSON.stringify({ system: scenario.system, navigator: scenario.browser, expected: scenario.expected, passed: true }));
            await context.close();
        }
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
