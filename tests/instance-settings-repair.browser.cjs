const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const output = process.env.LUXMC_ARTIFACT_DIR || 'docs/validation/modpack-repair-2026-10-25';
    fs.mkdirSync(output, { recursive: true });
    try {
        for (const width of [1440, 1000]) {
            const page = await browser.newPage({ viewport: { width, height: 1000 }, reducedMotion: 'reduce' });
            const errors = [];
            page.on('pageerror', error => errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(() => {
                window.instanceSettingsCalls = [];
                const original = window.electronAPI.invoke;
                const invoke = async (command, args = {}) => {
                    window.instanceSettingsCalls.push({ command, args });
                    return original(command, args);
                };
                window.electronAPI.invoke = invoke;
                window.__TAURI_INTERNALS__.invoke = invoke;
            });
            await page.goto('http://127.0.0.1:1420/instances/demo-fo');
            await page.getByRole('tab', { name: 'Configurações', exact: true }).click();
            const navigation = page.getByRole('navigation', { name: 'Categorias das configurações da instância' });
            await navigation.waitFor();
            assert.equal(await navigation.getByRole('button').count(), 6);
            assert.equal(await page.locator('#instance-settings-geral').isVisible(), true);
            assert.equal(await page.locator('#instance-settings-maintenance').isVisible(), false);
            const name = page.getByRole('textbox', { name: 'Nome da instância', exact: true });
            await name.fill('Meu modpack de teste');
            for (const category of ['Desempenho', 'Java e loader', 'Janela do jogo', 'Comandos avançados', 'Manutenção']) {
                await navigation.getByRole('button', { name: category, exact: true }).click();
                assert.equal(await navigation.getByRole('button', { name: category, exact: true }).getAttribute('aria-pressed'), 'true');
                assert.equal(await page.locator('.instance-settings-content > section:visible').count(), category === 'Java e loader' ? 2 : 1);
            }
            assert.equal(await page.getByRole('button', { name: /Excluir instância/ }).isVisible(), true);
            assert.equal(await page.evaluate(() => window.instanceSettingsCalls.some(call => ['profiles_delete', 'storage_delete_instance', 'instance_repair', 'instance_repair_modpack'].includes(call.command))), false);
            await navigation.getByRole('button', { name: 'Identidade', exact: true }).click();
            assert.equal(await name.inputValue(), 'Meu modpack de teste');
            await page.getByRole('button', { name: 'Cancelar', exact: true }).click();
            await page.waitForFunction(() => document.querySelector('#instance-settings-geral input[type="text"]')?.value !== 'Meu modpack de teste');
            await name.fill('Meu modpack de teste');
            await page.getByRole('button', { name: 'Guardar alterações', exact: true }).click();
            await page.waitForFunction(() => window.instanceSettingsCalls.some(call => call.command === 'profiles_update' && call.args.input.name === 'Meu modpack de teste'));
            const bounds = await page.locator('.instance-settings-layout').boundingBox();
            assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width + 1);
            await page.locator('#instance-configuracoes').screenshot({ path: `${output}/settings-${width}.png` });
            assert.deepEqual(errors, []);
            await page.close();
        }
        console.log('PASS instance settings categories, hidden maintenance, draft preservation, cancel, save and responsive layout');
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
