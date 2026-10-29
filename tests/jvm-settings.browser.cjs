const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        await setupLauncherDemo(page);
        await page.addInitScript(() => {
            window.jvmSettingsCalls = [];
            const original = window.electronAPI.invoke;
            const invoke = async (command, args = {}) => {
                window.jvmSettingsCalls.push({ command, args });
                if (command === 'jvm_args_validate' && args.ramMb !== undefined) {
                    if (window.rejectJvmSave) throw new Error('Aspas incompletas nos argumentos do Java');
                    return { valid: true, rejected: [], suggestions: [], normalized: `"-Dpath=C:\\Game  Data\\natives" "-Dmessage=keep -Xmx8G here" -Xms${args.minRamMb}M -Xmx${args.ramMb}M` };
                }
                return original(command, args);
            };
            window.electronAPI.invoke = invoke;
            window.__TAURI_INTERNALS__.invoke = invoke;
        });
        await page.goto(`${process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420'}/instances/demo-fo`);
        await page.getByRole('tab', { name: 'Configurações', exact: true }).click();
        await page.getByRole('button', { name: 'Java e loader', exact: true }).click();
        const input = '-Dpath="C:\\Game  Data\\natives" "-Dmessage=keep -Xmx8G here" -Xms8G -Xmx16G';
        const field = page.getByRole('textbox', { name: 'Argumentos JVM', exact: true });
        await field.fill(input);
        const save = page.getByRole('button', { name: 'Guardar alterações', exact: true });
        await save.click();
        await page.waitForFunction(() => window.jvmSettingsCalls.some(call => call.command === 'profiles_update'));
        const calls = await page.evaluate(() => window.jvmSettingsCalls);
        const validation = calls.find(call => call.command === 'jvm_args_validate' && call.args.ramMb !== undefined);
        assert.equal(validation.args.input, input);
        assert.equal(validation.args.minRamMb, 1024);
        assert.equal(validation.args.ramMb, 4096);
        const updated = calls.find(call => call.command === 'profiles_update').args.input;
        assert.equal(updated.jvmArgs, '"-Dpath=C:\\Game  Data\\natives" "-Dmessage=keep -Xmx8G here" -Xms1024M -Xmx4096M');
        await page.evaluate(() => window.rejectJvmSave = true);
        await field.fill('-Dpath="incomplete');
        await save.click();
        await page.getByText(/Aspas incompletas nos argumentos do Java/).waitFor();
        assert.equal(await page.evaluate(() => window.jvmSettingsCalls.filter(call => call.command === 'profiles_update').length), 1);
        assert.deepEqual(errors, []);
        console.log('PASS JVM settings preserve the original quoted input, use native heap normalization, and reject invalid saves without updating the profile');
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
