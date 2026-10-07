const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    try {
        for (const width of [1440, 960]) {
            const page = await browser.newPage({ locale: 'en-US', viewport: { width, height: 900 } });
            const errors = [];
            page.on('pageerror', error => errors.push(String(error)));
            await page.addInitScript(() => {
                const profile = { id: 'content-test', name: 'Biblioteca de teste', icon: 'grass_block', mcVersion: '1.21.1', loader: 'neoforge', loaderVersion: '21.1.1', gameDir: 'C:\\Users\\Pedro Silva\\AppData\\Roaming\\Luxmc\\instances\\teste', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), ramMb: 4096, modCount: 1000 };
                const account = { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' };
                let mods = Array.from({ length: 1000 }, (_, index) => ({ name: `project-${String(index).padStart(4, '0')}-1.0.jar`, path: `${profile.gameDir}\\mods\\project-${index}.jar`, isDir: false, size: 1024, icon: '/grass_block.png' }));
                window.testCalls = [];
                const invoke = async (command, args = {}) => {
                    window.testCalls.push({ command, args });
                    if (command === 'app_init') return { account, profiles: [profile], activeProfileId: profile.id, devMode: true };
                    if (command === 'profiles_list' || command === 'instances_list') return [profile];
                    if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
                    if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: true, liveWallpaper: true, soundEnabled: false, soundscapesEnabled: false }, true];
                    if (command === 'instance_file_tree') {
                        if (args.subPath === 'mods') return mods;
                        if (args.subPath === 'resourcepacks' || args.subPath?.endsWith('/datapacks')) return [{ name: 'Pack.zip', path: `${profile.gameDir}\\${args.subPath}\\Pack.zip`, isDir: false, size: 2048, icon: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=' }];
                        if (args.subPath === 'shaderpacks') return [{ name: 'Complementary.zip', path: `${profile.gameDir}\\shaderpacks\\Complementary.zip`, isDir: false, size: 2048, icon: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=' }];
                        return [];
                    }
                    if (command === 'instance_mod_toggle') {
                        mods = mods.map(mod => mod.name === args.fileName ? { ...mod, name: args.enabled ? mod.name.replace(/\.disabled$/, '') : `${mod.name}.disabled` } : mod);
                        return null;
                    }
                    if (command === 'mods_search') return [];
                    if (command === 'java_scan') return { runtimes: [{ path: '/java/21', major: 21, versionString: '21', installed: true }] };
                    if (command === 'profiles_update') { await new Promise(resolve => setTimeout(resolve, 200)); if (window.rejectSettingsSave) throw new Error('Falha simulada'); Object.assign(profile, args.input); return profile; }
                    if (command === 'mods_check_updates') return [{ projectId: 'mod-0', projectName: 'project 0000', fileName: 'project-0000-1.0.jar', currentVersionId: 'one', latestVersionId: 'two', latestVersionNumber: '2.0' }];
                    if (command === 'keybinds_list') return { keybinds: [{ id: 'key_key.forward', label: 'Andar para frente', category: 'Movimento', rawKey: 'key.keyboard.w', displayKey: 'W', isConflict: false, conflictingWith: [] }], totalConflicts: 0 };
                    if (command === 'instance_options_get') return { gamma: 1, fov: 70, renderDistance: 12, simulationDistance: 12, maxFps: 120, guiScale: 0, fullscreen: false, vsync: false, autoJump: false, bobView: true, soundMaster: 1, soundMusic: 0.5 };
                    if (command === 'instance_config_read') return { content: 'gamma:1.0', relativePath: 'options.txt' };
                    if (command === 'versions_check_installed' || command === 'curseforge_status') return true;
                    if (command === 'instance_shield_scan') return { isClean: true, threats: [] };
                    if (command === 'instance_worlds_list') return [{ folderName: 'Meu mundo', name: 'Meu mundo', path: '/saves/Meu mundo', size: 2048, lastPlayed: 0 }];
                    if (command === 'deep_links_take' || command === 'screenshots_list' || command === 'instance_worlds_list' || command === 'instances_screenshots') return [];
                    if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Windows', arch: 'x86_64' };
                    if (command === 'mesh_status') return { available: false, peers: [] };
                    return null;
                };
                window.electronAPI = { invoke, on: () => () => {} };
                window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
            });
            await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/instances/content-test');
            const list = page.locator('[data-virtual-list]');
            await list.locator('[data-index]').first().waitFor();
            assert.ok(await list.locator('[data-index]').count() < 40);
            assert.equal(await page.locator('main').evaluate(element => element.scrollWidth > element.clientWidth), false);
            await list.getByRole('button', { name: 'Mais opções', exact: true }).first().click();
            await page.getByRole('button', { name: 'Congelar versão', exact: true }).click();
            await page.getByRole('button', { name: 'Verificar atualizações', exact: true }).click();
            await page.getByRole('button', { name: 'Atualizar Todos os Mods (1)', exact: true }).click();
            await page.getByText('As atualizações disponíveis estão congeladas.', { exact: true }).waitFor();
            assert.equal(await page.evaluate(() => window.testCalls.filter(call => call.command === 'mods_update').length), 0);
            await page.reload();
            await list.getByTitle('Versão congelada', { exact: true }).first().waitFor();
            if (process.env.LUXMC_ARTIFACT_DIR) {
                fs.mkdirSync(process.env.LUXMC_ARTIFACT_DIR, { recursive: true });
                await page.locator('main').evaluate(element => { element.scrollTop = 540; });
                await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/instance-mods-${width}.png` });
            }
            await list.evaluate(element => { element.scrollTop = element.scrollHeight; });
            await page.getByRole('heading', { name: 'project 0999', exact: true }).waitFor();
            assert.ok(await list.locator('[data-index]').count() < 40);
            const input = page.getByRole('searchbox', { name: 'Buscar conteúdo instalado' });
            await input.fill('0999');
            await page.waitForFunction(() => document.querySelectorAll('[data-virtual-list] [data-index]').length === 1);
            await page.getByRole('checkbox', { name: 'Selecionar project 0999', exact: true }).check();
            await page.getByRole('button', { name: 'Desativar', exact: true }).click();
            await page.waitForFunction(() => window.testCalls.some(call => call.command === 'instance_mod_toggle'));
            const toggle = await page.evaluate(() => window.testCalls.find(call => call.command === 'instance_mod_toggle'));
            assert.equal(toggle.args.profileId, 'content-test');
            assert.equal(toggle.args.enabled, false);
            assert.equal(await page.locator('main [role="tablist"]').count(), 1);
            assert.equal(await page.getByRole('tab').count(), 8);
            await page.getByRole('tab', { name: 'Mods', exact: true }).focus();
            await page.keyboard.press('ArrowRight');
            assert.equal(await page.getByRole('tab', { name: 'Pacotes de recursos', exact: true }).getAttribute('aria-selected'), 'true');
            await page.keyboard.press('Home');
            assert.equal(await page.getByRole('tab', { name: 'Mods', exact: true }).getAttribute('aria-selected'), 'true');
            assert.equal(await page.getByRole('tabpanel').count(), 1);
            assert.equal(await page.locator('#instance-mundos').count(), 0);
            await input.fill('');
            const shaders = page.getByRole('region', { name: 'Shaders', exact: true });
            const toggleWidth = await list.getByRole('switch').first().evaluate(element => element.getBoundingClientRect().width);
            assert.equal(toggleWidth, 44);
            const trash = await list.getByRole('button', { name: /^Excluir / }).first().locator('svg').evaluate(element => element.getBoundingClientRect().width);
            assert.equal(trash, 16);
            await page.getByRole('tab', { name: 'Shaders', exact: true }).click();
            await shaders.getByText('Complementary.zip', { exact: true }).waitFor();
            await shaders.locator('img').waitFor();
            assert.equal(await shaders.locator('img').getAttribute('loading'), 'lazy');
            await shaders.getByRole('button', {name: /^Abrir pasta:/}).click();
            assert.ok(await page.evaluate(() => window.testCalls.some(call => call.command === 'instance_pack_open_folder' && call.args.packType === 'shaderpacks')));
            assert.equal(await list.count(), 0);
            assert.equal(await page.getByRole('tabpanel').count(), 1);
            for (const name of ['Pacotes de recursos', 'Datapacks', 'Mundos', 'Screenshots', 'Arquivos']) {
                await page.getByRole('tab', { name, exact: true }).click();
                if (name === 'Datapacks') {
                    await page.getByText('Pack.zip',{exact:true}).waitFor();
                    assert.equal(await page.getByRole('combobox',{name:'Mundos'}).inputValue(),'Meu mundo');
                    assert.ok(await page.evaluate(() => window.testCalls.some(call => call.command === 'instance_file_tree' && call.args.subPath === 'saves/Meu mundo/datapacks')));
                }
                assert.equal(await page.getByRole('tabpanel').count(), 1);
                assert.equal(await page.getByRole('tab', { name, exact: true }).getAttribute('aria-selected'), 'true');
                assert.equal(await list.count(), 0);
            }
            await page.locator('main').evaluate(element => { element.scrollTop = 0; });
            if (process.env.LUXMC_ARTIFACT_DIR) {
                fs.mkdirSync(process.env.LUXMC_ARTIFACT_DIR, { recursive: true });
                await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/instance-content-${width}.png`, fullPage: true });
            }
            await page.getByRole('tab', { name: 'Configurações', exact: true }).click();
            const settings = page.getByRole('region', { name: 'Configurações da instância', exact: true });
            assert.equal(await settings.getByText('Pacote de Mods de Performance', {exact:true}).count(), 0);
            assert.equal(await page.evaluate(()=>window.testCalls.some(call=>call.command === 'optimizer_get_perf_pack')), false);
            await settings.getByRole('textbox', { name: 'Nome da instância', exact: true }).fill('Perfil integrado');
            assert.equal(await page.getByRole('dialog', { name: 'Configurações da instância' }).count(), 0);
            await settings.locator('#instance-ram-min').evaluate(element => { element.value = '2048'; element.dispatchEvent(new Event('input', { bubbles: true })); });
            await settings.locator('#instance-ram-max').evaluate(element => { element.value = '6144'; element.dispatchEvent(new Event('input', { bubbles: true })); });
            await settings.locator('#instance-java').selectOption('/java/21');
            await settings.getByRole('button', { name: 'G1GC', exact: true }).click();
            assert.equal(await settings.getByRole('button', { name: /GameMode/ }).count(), 0);
            assert.equal(await settings.getByRole('button', { name: /MangoHud/ }).count(), 0);
            await settings.getByRole('button', { name: /Verificação completa/ }).click();
            await page.evaluate(() => { window.rejectSettingsSave = true; });
            const save = settings.getByRole('button', { name: 'Guardar alterações', exact: true });
            await save.click();
            assert.equal(await save.isDisabled(), true);
            await page.getByText(/Erro ao salvar no banco de dados:.*Falha simulada/).waitFor();
            assert.equal(await save.isEnabled(), true);
            await page.evaluate(() => { window.rejectSettingsSave = false; });
            await save.click();
            await page.getByText('Configurações salvas com sucesso!', { exact: true }).waitFor();
            const savedSettings = await page.evaluate(() => window.testCalls.filter(call => call.command === 'profiles_update').at(-1).args.input);
            assert.equal(savedSettings.name, 'Perfil integrado');
            assert.equal(savedSettings.ramMb, 6144);
            assert.equal(savedSettings.javaPath, '/java/21');
            assert.ok(savedSettings.jvmArgs.includes('-Xms2048M') && savedSettings.jvmArgs.includes('-XX:+UseG1GC'));
            assert.equal(savedSettings.useGamemode, false);
            assert.equal(savedSettings.useMangohud, false);
            assert.equal(savedSettings.forceFullVerification, true);
            if (process.env.LUXMC_ARTIFACT_DIR) {
                await page.getByRole('button', { name: 'Fechar', exact: true }).evaluateAll(elements => elements.forEach(element => element.click()));
                await settings.evaluate(element => { const root = document.querySelector('main'); root.scrollTop += element.getBoundingClientRect().top - root.getBoundingClientRect().top - 16; });
                await page.screenshot({ path: `${process.env.LUXMC_ARTIFACT_DIR}/instance-settings-${width}.png` });
            }
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
            const jukebox = page.getByRole('dialog', { name: 'Jukebox do Launcher — Discos e Sons' });
            await jukebox.waitFor();
            await jukebox.getByRole('button', { name: 'Fechar', exact: true }).last().click();
            await tools.getByRole('button', { name: 'Fechar', exact: true }).click();
            const toolCommands = await page.evaluate(() => window.testCalls.filter(call => ['keybinds_list', 'instance_options_get'].includes(call.command)));
            assert.ok(toolCommands.some(call => call.command === 'keybinds_list' && call.args.profileId === 'content-test'));
            assert.ok(toolCommands.some(call => call.command === 'instance_options_get' && call.args.profileId === 'content-test'));
            await settings.getByRole('textbox', { name: 'Nome da instância', exact: true }).fill('Rascunho descartado');
            await settings.getByRole('button', { name: 'Cancelar', exact: true }).click();
            await page.waitForFunction(() => document.querySelector('input[aria-label="Nome da instância"]').value === 'Perfil integrado');
            await page.getByRole('tab', { name: 'Shaders', exact: true }).click();
            await shaders.getByRole('link', { name: 'Explorar', exact: true }).click();
            await page.waitForFunction(() => window.testCalls.some(call => call.command === 'mods_search' && call.args.contentType === 'shader' && call.args.mcVersion === '1.21.1'));
            const search = await page.evaluate(() => window.testCalls.filter(call => call.command === 'mods_search' && call.args.contentType === 'shader').at(-1));
            assert.equal(search.args.contentType, 'shader');
            assert.equal(search.args.mcVersion, '1.21.1');
            assert.deepEqual(errors, []);
            console.log(JSON.stringify({ width, mods: 1000, boundedRows: true, lastModAccessible: true, filtering: true, frozenUpdatesSkipped: true, freezeSurvivesReload: true, batchToggle: toggle.args, singleTabBar: true, isolatedPanels: true, integratedSettings: true, toolsReachable: true, saveErrorRecovery: true, catalog: search.args, errors }));
            await page.close();
        }
    } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
