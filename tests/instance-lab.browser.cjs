const { chromium } = require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright-core');
const assert = require('node:assert/strict');

(async () => {
    const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE || undefined });
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.addInitScript(() => {
        const profile = { id: 'fabric-profile', name: 'Fabric de teste', icon: 'grass_block', mcVersion: '1.20.1', loader: 'fabric', loaderVersion: '0.16.0', gameDir: '/tmp/luxmc-test', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), ramMb: 4096, modCount: 4 };
        const account = { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' };
        let isolation = null;
        let capsules = [];
        let benchmarks = [];
        window.testCalls = [];
        const invoke = async (command, args = {}) => {
            window.testCalls.push({ command, args });
            if (command === 'app_init') return { account, profiles: [profile], activeProfileId: profile.id, devMode: true };
            if (command === 'profiles_list' || command === 'instances_list') return [profile];
            if (command === 'java_scan') return { runtimes: [] };
            if (command === 'deep_links_take' || command === 'mods_search' || command === 'screenshots_list' || command === 'instance_file_tree' || command === 'instance_worlds_list' || command === 'instances_screenshots') return [];
            if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
            if (command === 'plugin:store|get') return [{ animations: false, liveWallpaper: false, soundscapesEnabled: false }, true];
            if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Linux', arch: 'x86_64' };
            if (command === 'mesh_status') return { available: false, state: '', ip: null, peers: [] };
            if (command === 'versions_check_installed') return true;
            if (command === 'instance_isolation_status') return isolation;
            if (command === 'instance_isolation_start') return isolation = { profileId: profile.id, originalEnabled: ['alpha.jar', 'beta.jar'], suspects: ['alpha.jar', 'beta.jar'], trialDisabled: ['alpha.jar'], round: 1, phase: 'testing' };
            if (command === 'instance_isolation_report') return isolation = { ...isolation, suspects: [args.crashed ? 'beta.jar' : 'alpha.jar'], trialDisabled: [], round: 2, phase: 'found' };
            if (command === 'instance_isolation_restore') { isolation = null; return null; }
            if (command === 'instance_capsules_list') return capsules;
            if (command === 'instance_capsule_create') { const item = { filename: 'capsule-one.zip', label: args.label, createdAt: new Date().toISOString(), sizeBytes: 1024 }; capsules.unshift(item); return item; }
            if (command === 'instance_capsule_restore') { const item = { filename: 'capsule-before.zip', label: 'Antes da restauração', createdAt: new Date().toISOString(), sizeBytes: 1024 }; capsules.unshift(item); return item; }
            if (command === 'instance_benchmarks_list') return benchmarks;
            if (command === 'plugin:dialog|open') return '/tmp/mangohud.csv';
            if (command === 'instance_benchmark_import') { const item = { id: String(benchmarks.length + 1), label: args.label, createdAt: new Date().toISOString(), averageFps: 120, lowOnePercentFps: 70, fpsDrops: 3, samples: 100, source: 'MangoHud CSV' }; benchmarks.unshift(item); return item; }
            return null;
        };
        window.electronAPI = { invoke, on: () => () => {} };
        window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: () => '/grass_head.png' };
    });
    await page.goto((process.env.LUXMC_BASE_URL || 'http://127.0.0.1:1420') + '/instances/fabric-profile');
    await page.getByRole('tab', { name: 'Configurações', exact: true }).click();
    await page.getByRole('button', { name: 'Ferramentas', exact: true }).click();
    await page.locator('summary').filter({ hasText: 'Laboratório de diagnóstico' }).click();
    await page.getByRole('button', { name: 'Iniciar diagnóstico reversível' }).click();
    await page.getByText('Rodada 1: 1 mods temporariamente desativados.').waitFor();
    await page.getByRole('button', { name: 'Abriu sem crash' }).click();
    await page.getByText('Suspeito:').waitFor();
    assert.ok(await page.getByText('alpha.jar').count());
    await page.getByRole('button', { name: 'Restaurar todos os mods' }).click();
    await page.getByRole('button', { name: 'Iniciar diagnóstico reversível' }).waitFor();
    await page.getByRole('textbox', { name: 'Nome da cápsula' }).fill('Antes dos shaders');
    await page.getByRole('button', { name: 'Criar cápsula' }).click();
    await page.getByText('Antes dos shaders').waitFor();
    page.once('dialog', dialog => dialog.accept());
    await page.getByRole('button', { name: 'Restaurar', exact: true }).click();
    await page.getByText('Antes da restauração').waitFor();
    await page.getByRole('textbox', { name: 'Nome da medição' }).fill('Antes');
    await page.getByRole('button', { name: 'Importar CSV MangoHud' }).click();
    await page.getByText('120.0 FPS médios').waitFor();
    const enabledAfterImport = await page.getByRole('button', { name: 'Importar CSV MangoHud' }).isEnabled();
    assert.equal(enabledAfterImport, true);
    assert.deepEqual(errors, []);
    await page.screenshot({ path: require('node:path').join(require('node:os').tmpdir(), 'luxmc-instance-lab.png'), fullPage: true });
    console.log(JSON.stringify({ isolation: true, capsules: true, benchmark: true, enabledAfterImport, errors }));
    await browser.close();
})().catch(error => { console.error(error); process.exit(1); });
