const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ locale: 'en-US', viewport: { width: 1440, height: 900 }, reducedMotion: 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        window.createdProfiles = [];
        window.bedrockOpened = [];
        window.bedrock = { supported: true, warning: null, provider: { executable: 'C:\\BedrockLauncher\\BedrockLauncher.exe', dataDirectory: 'C:\\BedrockData' }, installations: [{ id: 'bedrock-one', profileId: 'profile-one', profileName: 'Steve', name: 'Bedrock 1.20', version: 'exact-version', directory: 'C:\\BedrockData\\installations\\Steve\\one' }], instances: [] };
        const invoke = async (command, args) => {
            if (command === 'bedrock_state') return structuredClone(window.bedrock);
            if (command === 'bedrock_open') { window.bedrockOpened.push(args.instanceId); return { providerOpened: true, notice: 'Selecione sua instalação no BedrockLauncher.' }; }
            if (command === 'bedrock_add') { window.bedrock.instances.push({ id: 'bedrock-link', name: args.name, profileId: args.profileId, installationId: args.installationId }); return null; }
            if (command === 'app_init') return { account: null, profiles: [], activeProfileId: null, devMode: true };
            if (command === 'versions_list') return { versions: [{ id: '1.21.4', versionType: 'release', releaseTime: '2024-12-03' }], latestRelease: '1.21.4', latestSnapshot: '' };
            if (command === 'loaders_versions') return { versions: args.loader === 'neoforge'
                ? [{ id: '21.4.159-beta', stable: false }, { id: '21.4.158', stable: true }]
                : [{ id: '0.28.1', stable: true }] };
            if (command === 'profiles_create') {
                window.createdProfiles.push(args.input);
                return { ...args.input, id: 'created-' + window.createdProfiles.length, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), gameDir: '/tmp/luxmc-browser-instance', fullscreen: false };
            }
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'java_scan') return { runtimes: [] };
            if (command === 'deep_links_take' || command === 'profiles_list' || command === 'instances_list' || command === 'screenshots_list') return [];
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'optimizer_install_perf_pack') return [];
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
    });
    await page.goto('http://127.0.0.1:1420/instances');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: /Java Edition/ }).click();
    await page.getByRole('button', { name: 'Criar instância' }).click();
    await page.waitForFunction(() => window.createdProfiles.length === 1);
    const vanilla = await page.evaluate(() => window.createdProfiles[0]);
    assert.equal(vanilla.loader, 'vanilla');
    assert.equal(vanilla.mcVersion, '1.21.4');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: /Java Edition/ }).click();
    await page.getByRole('button', { name: 'NeoForge', exact: true }).click();
    await page.getByRole('button', { name: 'Criar instância' }).isEnabled();
    await page.getByRole('button', { name: 'Criar instância' }).click();
    await page.waitForFunction(() => window.createdProfiles.length === 2);
    const created = await page.evaluate(() => window.createdProfiles[1]);
    assert.equal(created.loader, 'neoforge');
    assert.equal(created.loaderVersion, '21.4.158');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: /Java Edition/ }).click();
    await page.getByRole('button', { name: 'NeoForge', exact: true }).click();
    await page.getByRole('button', { name: 'Última', exact: true }).click();
    await page.getByRole('button', { name: 'Criar instância' }).click();
    await page.waitForFunction(() => window.createdProfiles.length === 3);
    const latest = await page.evaluate(() => window.createdProfiles[2]);
    assert.equal(latest.loaderVersion, '21.4.159-beta');
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: /Bedrock Edition/ }).click();
    await page.getByRole('combobox', { name: 'Instalação', exact: true }).selectOption(JSON.stringify(['profile-one', 'bedrock-one']));
    await page.getByRole('textbox', { name: 'Nome na biblioteca', exact: true }).fill('Meu Bedrock');
    await page.getByRole('button', { name: 'Adicionar à biblioteca', exact: true }).click();
    await page.getByRole('heading', { name: 'Meu Bedrock', exact: true }).waitFor();
    assert.equal(await page.evaluate(() => window.createdProfiles.length), 3);
    assert.deepEqual(await page.evaluate(() => window.bedrock.instances[0]), { id: 'bedrock-link', name: 'Meu Bedrock', profileId: 'profile-one', installationId: 'bedrock-one' });
    await page.locator('[data-bedrock-instance="bedrock-link"]').getByRole('button', { name: 'Jogar', exact: true }).click();
    await page.getByText('Selecione sua instalação no BedrockLauncher.', { exact: true }).waitFor();
    assert.deepEqual(await page.evaluate(() => window.bedrockOpened), ['bedrock-link']);
    await page.evaluate(() => {
        window.bedrock.provider = null;
        window.bedrock.installations.push({id:'Microsoft.MinecraftUWP_8wekyb3d8bbwe!App',profileId:'windows',profileName:'Windows',name:'Minecraft',version:'Instalada',directory:''});
    });
    await page.getByRole('button', { name: 'Nova Instância' }).click();
    await page.getByRole('button', { name: /Bedrock Edition/ }).click();
    assert.equal(await page.getByRole('button',{name:'Conectar BedrockLauncher',exact:true}).count(),0);
    await page.getByRole('combobox',{name:'Instalação',exact:true}).selectOption(JSON.stringify(['windows','Microsoft.MinecraftUWP_8wekyb3d8bbwe!App']));
    await page.getByRole('textbox',{name:'Nome na biblioteca',exact:true}).fill('Bedrock direto');
    await page.evaluate(() => window.bedrock.instances = []);
    await page.getByRole('button',{name:'Adicionar à biblioteca',exact:true}).click();
    await page.locator('[data-bedrock-instance="bedrock-link"]').getByRole('button',{name:'Jogar',exact:true}).waitFor();
    assert.equal(await page.evaluate(() => window.bedrock.instances[0].profileId),'windows');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ loader: created.loader, stable: created.loaderVersion, latest: latest.loaderVersion, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
