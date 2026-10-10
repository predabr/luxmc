const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
    const results = [];
    try {
        for (const performanceMode of [false, true]) {
            const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
            await setupLauncherDemo(page);
            await page.addInitScript(performanceMode => {
                const state = window.launcherDemo.state;
                Object.assign(state.settings, { theme: 'default-light', accentTheme: 'gold', customBackground: 'custom', customWallpaperUrl: '/vanilla_banner.png', wallpaperLibrary: [{ url: '/vanilla_banner.png', type: 'image', name: 'Meu fundo' }], performanceMode, animations: true, respectReducedMotion: false, settingsSavedAt: 200 });
                localStorage.setItem('luxmc_theme', 'dark');
                window.testFailure = true;
                window.testCalls = [];
                window.testBedrock = { supported: true, provider: null, warning: null, installations: [{ id: 'package', profileId: 'managed', profileName: 'Luxmc', name: 'Bedrock 1.20', version: '1.20.81', directory: 'C:/Luxmc/bedrock/packages/test' }], instances: [{ id: 'bedrock-test', profileId: 'managed', installationId: 'package', name: 'Meu Bedrock' }] };
                const invoke = window.electronAPI.invoke;
                const wrapped = async (command, args) => {
                    window.testCalls.push({ command, args });
                    if (command === 'settings_get') return state.settings;
                    if (command === 'settings_set') { Object.assign(state.settings, args.value); return; }
                    if (command === 'plugin:store|get' && args.key === 'app') return [{ ...state.settings, theme: 'default-dark', customBackground: 'obsidian', customWallpaperUrl: '', settingsSavedAt: 100 }, true];
                    if (command === 'bedrock_state') return window.testBedrock;
                    if (command === 'bedrock_versions') return [
                        { id: 'new', version: '1.26.52.3', channel: 'release', packageType: 'GDK' },
                        { id: 'old', version: '1.20.81.01', channel: 'release', packageType: 'UWP' },
                        { id: 'preview', version: '1.26.60.1', channel: 'preview', packageType: 'GDK' }
                    ];
                    if (command === 'bedrock_install') return;
                    if (command === 'bedrock_rename') { window.testBedrock.instances[0].name = args.name; return; }
                    if (command === 'bedrock_remove') {
                        await new Promise(resolve => setTimeout(resolve, 800));
                        if (window.testFailure) throw new Error('Pacote em uso');
                        window.testBedrock.instances = []; return;
                    }
                    if (command === 'profiles_delete') {
                        await new Promise(resolve => window.finishDelete = resolve);
                        if (window.testFailure) throw new Error('Arquivo em uso');
                    }
                    return invoke(command, args);
                };
                window.electronAPI.invoke = wrapped;
                window.__TAURI_INTERNALS__.invoke = wrapped;
                window.motionCalls = [];
                window.motionDetails = [];
                const animate = Element.prototype.animate;
                Element.prototype.animate = function(frames, options) {
                    if (this.hasAttribute('data-page-route')) window.motionCalls.push(options.duration);
                    window.motionDetails.push({ route: location.pathname, target: this.id || this.className, frames, duration: options.duration, delay: options.delay });
                    return animate.call(this, frames, options);
                };
            }, performanceMode);
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await page.goto('http://127.0.0.1:1420/instances');
            await page.bringToFront();
            console.log('instances loaded', performanceMode);
            await page.locator('[data-page-route="/instances"]').waitFor();
            await page.waitForFunction(() => document.documentElement.classList.contains('light') && document.documentElement.classList.contains('has-custom-wallpaper'));
            console.log('appearance restored', performanceMode);
            const card = page.locator('div').filter({ has: page.getByRole('heading', { name: 'Vanilla Perfected', exact: true }) }).filter({ has: page.getByTitle('Mais Opções', { exact: true }) }).last();
            const openDelete = async () => {
                await card.getByTitle('Mais Opções', { exact: true }).click();
                console.log('menu open');
                assert.notEqual(await page.evaluate(() => getComputedStyle(document.body).pointerEvents), 'none', 'O menu de ações não deve bloquear os cliques do launcher inteiro');
                await page.getByRole('menuitem', { name: /Excluir/ }).click();
                console.log('delete requested');
                await page.getByRole('dialog', { name: 'Excluir Instância', exact: true }).waitFor();
            };
            await openDelete();
            let dialog = page.getByRole('dialog', { name: 'Excluir Instância', exact: true });
            assert.equal(await dialog.evaluate(node => node.parentElement === document.body), true);
            await dialog.getByRole('button', { name: 'Cancelar', exact: true }).click();
            await dialog.waitFor({ state: 'detached' });
            for (let repeat = 0; repeat < 4; repeat++) {
                await openDelete();
                await dialog.getByRole('button', { name: 'Cancelar', exact: true }).click();
                await dialog.waitFor({ state: 'detached' });
                await page.getByRole('textbox', { name: 'Buscar instâncias... (Ctrl+F)', exact: true }).fill('Vanilla');
                await page.getByRole('textbox', { name: 'Buscar instâncias... (Ctrl+F)', exact: true }).fill('');
            }
            await openDelete();
            await dialog.getByRole('button', { name: 'Excluir Definitivamente', exact: true }).click();
            const spinner = dialog.locator('.animate-spin');
            await spinner.waitFor();
            const before = await spinner.evaluate(node => ({ transform: getComputedStyle(node).transform, duration: getComputedStyle(node).animationDuration, iterations: getComputedStyle(node).animationIterationCount, playState: getComputedStyle(node).animationPlayState, hidden: document.hidden }));
            console.log('spinner', before);
            await page.waitForFunction(transform => {
                const node = document.querySelector('[role="dialog"] .animate-spin');
                return node && getComputedStyle(node).transform !== transform;
            }, before.transform, { timeout: 5000 });
            const after = await spinner.evaluate(node => getComputedStyle(node).transform);
            assert.equal(before.duration, '0.9s'); assert.equal(before.iterations, 'infinite'); assert.notEqual(before.transform, after);
            await page.waitForFunction(() => window.testCalls.some(call => call.command === 'profiles_delete'));
            await page.evaluate(() => window.finishDelete());
            await page.waitForFunction(() => [...document.querySelectorAll('[role="dialog"] button')].some(button => button.textContent.includes('Excluir Definitivamente') && !button.disabled));
            assert.equal(await dialog.isVisible(), true);
            assert.equal(await page.getByRole('heading', { name: 'Vanilla Perfected', exact: true }).count(), 1);
            await page.evaluate(() => window.testFailure = false);
            await dialog.getByRole('button', { name: 'Excluir Definitivamente', exact: true }).click();
            await page.waitForFunction(() => window.testCalls.filter(call => call.command === 'profiles_delete').length === 2);
            await page.evaluate(() => window.finishDelete());
            await dialog.waitFor({ state: 'detached' });
            assert.equal(await page.getByRole('heading', { name: 'Vanilla Perfected', exact: true }).count(), 0);
            assert.notEqual(await page.evaluate(() => getComputedStyle(document.body).pointerEvents), 'none');
            await page.waitForTimeout(20000);
            const remaining = page.getByTitle('Mais Opções', { exact: true }).first();
            await remaining.click();
            await page.getByRole('menuitem', { name: /Excluir/ }).click();
            await dialog.waitFor();
            await dialog.getByRole('button', { name: 'Cancelar', exact: true }).click();
            await dialog.waitFor({ state: 'detached' });
            await page.locator('aside a[href="/settings"]').click();
            await page.locator('[data-page-route="/settings"]').waitFor();
            await page.waitForFunction(() => window.motionDetails.some(call => call.route === '/settings' && call.duration === 360) || document.documentElement.classList.contains('no-anim'));
            await page.getByRole('tab', { name: /Aparência/ }).click();
            await page.locator('#settings-panel[aria-labelledby="settings-tab-appearance"]').waitFor();
            await page.waitForFunction(() => !document.querySelector('#settings-panel').getAnimations({ subtree: true }).some(animation => animation.id === 'luxmc-page-entry'));
            await page.locator('aside a[href="/instances"]').first().click();
            await page.locator('[data-page-route="/instances"]').waitFor();
            assert.equal(await page.getByRole('dialog').count(), 0);
            await page.getByRole('button', { name: 'Editar Meu Bedrock', exact: true }).click();
            dialog = page.getByRole('dialog', { name: 'Editar instância Bedrock', exact: true });
            await dialog.getByRole('textbox').fill('Bedrock renomeado');
            await dialog.getByRole('button', { name: 'Salvar', exact: true }).click();
            await dialog.waitFor({ state: 'detached' });
            await page.getByRole('button', { name: 'Excluir Bedrock renomeado', exact: true }).click();
            dialog = page.getByRole('dialog', { name: 'Excluir instância Bedrock', exact: true });
            await page.evaluate(() => window.testFailure = true);
            await dialog.getByRole('button', { name: 'Excluir definitivamente', exact: true }).click();
            await dialog.getByRole('alert').filter({ hasText: 'Pacote em uso' }).waitFor();
            assert.equal(await page.getByRole('heading', { name: 'Bedrock renomeado', exact: true }).count(), 1);
            await page.evaluate(() => window.testFailure = false);
            await dialog.getByRole('button', { name: 'Excluir definitivamente', exact: true }).click();
            await dialog.waitFor({ state: 'detached' });
            assert.equal(await page.locator('[data-bedrock-instance]').count(), 0);
            await page.getByRole('button', { name: 'Nova Instância', exact: true }).click();
            await page.getByRole('button', { name: /Bedrock Edition/ }).click();
            await page.getByRole('textbox', { name: 'Buscar versão Bedrock', exact: true }).fill('1.20');
            const versionList = page.getByRole('listbox', { name: 'Versões disponíveis do Bedrock', exact: true });
            assert.equal(await versionList.getByRole('option').count(), 1);
            await versionList.getByRole('option', { name: /1.20.81.01/ }).click();
            await page.getByRole('textbox', { name: 'Nome da instância Bedrock', exact: true }).fill('Instalação interna');
            await page.getByRole('button', { name: 'Instalar versão', exact: true }).click();
            await page.waitForFunction(() => window.testCalls.some(call => call.command === 'bedrock_install' && call.args.versionId === 'old' && call.args.name === 'Instalação interna'));
            assert.equal(await page.getByRole('button', { name: 'Instalar Bedrock oficial', exact: true }).count(), 0);
            assert.deepEqual(errors, []);
            assert.equal(await page.evaluate(mode => window.motionCalls.includes(mode ? 160 : 360), performanceMode), true);
            if (!performanceMode) assert.equal(await page.evaluate(() => window.motionDetails.some(call => call.frames.some(frame => frame.transform?.includes('32px')) && call.delay > 0)), true);
            await page.reload();
            await page.waitForFunction(() => document.documentElement.classList.contains('light') && document.documentElement.classList.contains('has-custom-wallpaper'));
            assert.equal(await page.evaluate(() => localStorage.getItem('luxmc_custom_wallpaper')), '/vanilla_banner.png');
            results.push({ performanceMode, reducedMotion: true, explicitAnimations: true, themeRestored: true, modalClickable: true, menuNeverBlocksBody: true, repeatedCancelAndSearch: true, clickableAfterDeletionAndWaiting: true, actualNavigationAndTabMotion: true, failedDeletePreservesInstance: true, retryDeletesInstance: true, bedrockRenameAndDelete: true, bedrockInternalVersionSelection: true, spinnerRotates: true, errors });
            await page.close();
        }
        fs.mkdirSync('docs/validation/v3.6/hotfix', { recursive: true });
        fs.writeFileSync('docs/validation/v3.6/hotfix/ui-results.json', JSON.stringify(results, null, 2));
        console.log(JSON.stringify(results));
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
