const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');

(async () => {
 const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
 try {
  for (const scenario of [{ width: 1600, wallpaper: false, empty: true }, { width: 1600, wallpaper: true, empty: true }, { width: 960, wallpaper: true, empty: false }, { width: 1600, wallpaper: false, empty: false }]) {
   const context = await browser.newContext({ viewport: { width: scenario.width, height: 1000 }, reducedMotion: 'reduce', permissions: ['clipboard-read', 'clipboard-write'] });
   const page = await context.newPage();
   const errors = [];
   page.on('pageerror', error => errors.push(String(error)));
   await page.addInitScript(scenario => {
    localStorage.setItem('luxmc.locale', 'pt-BR');
    if (scenario.wallpaper) {
     localStorage.setItem('luxmc_background', 'custom');
     localStorage.setItem('luxmc_custom_wallpaper', 'http://127.0.0.1:1420/friends-test-wallpaper.svg');
     localStorage.setItem('luxmc_custom_wallpaper_type', 'image');
    }
    const me = { id: 'cccccccc-cccc-4ccc-8ccc-cccccccccccc', username: 'LuxPlayer', avatarUrl: '/grass_head.png' };
    const base = { incoming: false, lastSeen: new Date().toISOString(), activity: null, mcVersion: null, loader: null, serverIp: null, serverPort: null, avatarUrl: '/grass_head.png' };
    window.friendRecords = scenario.empty ? [] : [
     { ...base, id: 'offline', username: 'Builder', status: 'offline' },
     { ...base, id: 'playing', username: 'Alex', status: 'in_game', activity: 'Vanilla Perfected', mcVersion: '26.2', serverIp: '127.0.0.1', serverPort: 25565 },
     { ...base, id: 'online', username: 'Explorer', status: 'online' },
     { ...base, id: 'request', username: 'Miner', status: 'pending', incoming: true }
    ];
    window.calls = [];
    const invoke = async (command, args) => {
     window.calls.push({ command, args });
     if (command === 'app_init') return { account: { id: 'offline_demo', uuid: 'demo', username: 'LuxPlayer', avatarUrl: '/grass_head.png', accessToken: '' }, profiles: [], activeProfileId: null, devMode: true };
     if (command === 'profiles_list' || command === 'deep_links_take') return [];
     if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
     if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, blur: false, interfaceOpacity: 18 }, true];
     if (command === 'social_request') {
      if (args.request.action === 'register') return { me };
      if (args.request.action === 'stream_ticket') return { url: null };
      if (args.request.action === 'search') return { users: [{ id: 'request', username: 'Miner' }] };
      if (args.request.action === 'accept') { window.friendRecords.find(friend => friend.id === args.request.targetId).status = 'online'; return { ok: true }; }
      return { me, friends: structuredClone(window.friendRecords) };
     }
     if (command === 'p2p_scan_lan_worlds') return [];
     if (command === 'tunnel_status') return null;
     if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Windows 11', arch: 'x86_64' };
     if (command === 'java_scan') return { runtimes: [] };
     return [];
    };
    window.electronAPI = { invoke, on: () => () => {} };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
    window.__TAURI_INTERNALS__ = { metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } }, invoke, transformCallback: () => 1, convertFileSrc: path => path };
   }, scenario);
   await page.route('**/friends-test-wallpaper.svg', route => route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000"><defs><linearGradient id="g"><stop stop-color="white"/><stop offset="1" stop-color="lightsteelblue"/></linearGradient></defs><rect width="1600" height="1000" fill="url(#g)"/></svg>' }));
   await page.route('https://**/*', route => route.abort());
   await page.goto('http://127.0.0.1:1420/friends');
   await page.getByText('Pronto para receber convites', { exact: true }).waitFor();
   assert.equal(await page.getByRole('button', { name: 'Adicionar amigo', exact: true }).count(), 1);
   assert.ok((await page.getByRole('button', { name: 'Adicionar amigo', exact: true }).boundingBox()).height >= 48);
   assert.ok((await page.locator('.friends-header').boundingBox()).height < 110);
   assert.ok(await page.locator('main').evaluate(main => main.scrollWidth - main.clientWidth <= 1));
   const copy = page.getByRole('button', { name: 'Copiar código de amizade', exact: true });
   await copy.click();
   assert.equal(await page.evaluate(() => navigator.clipboard.readText()), 'LuxPlayer#cccccccc');
   if (scenario.wallpaper) {
    await page.waitForFunction(() => document.documentElement.classList.contains('has-custom-wallpaper'));
    const alpha = await page.locator('.friends-directory').evaluate(element => Number(getComputedStyle(element).backgroundColor.match(/,\s*([\d.]+)\)$/)?.[1]));
    assert.ok(Math.abs(alpha - .18) < .001, `Wallpaper alpha ${alpha}`);
   }
   if (scenario.empty) {
    await page.getByRole('heading', { name: 'Sua turma começa com um convite' }).waitFor();
    assert.ok((await page.locator('.friends-empty').boundingBox()).height < 240);
    await page.getByText('Encontre um jogador', { exact: true }).waitFor();
   } else {
    const rows = page.locator('.friend-row');
    assert.equal(await rows.count(), 3);
    assert.ok((await rows.first().boundingBox()).height <= 150);
    assert.equal(await rows.first().getByRole('heading').textContent(), 'Alex');
    await page.getByRole('button', { name: 'Favoritar Builder', exact: true }).click();
    assert.equal(await rows.first().getByRole('heading').textContent(), 'Builder');
    await page.locator('summary[aria-label="Mais ações para Builder"]').click();
    await page.waitForFunction(() => document.querySelector('summary[aria-label="Mais ações para Builder"]').getAttribute('aria-expanded') === 'true');
    await page.getByRole('button', { name: 'Remover Builder', exact: true }).waitFor();
    await page.locator('summary[aria-label="Mais ações para Builder"]').press('Escape');
    assert.equal(await page.locator('summary[aria-label="Mais ações para Builder"]').evaluate(element => element.parentElement.open), false);
    await page.waitForFunction(() => document.querySelector('summary[aria-label="Mais ações para Builder"]').getAttribute('aria-expanded') === 'false');
    await page.getByRole('button', { name: 'Online 2', exact: true }).click();
    assert.equal(await rows.count(), 2);
    await page.getByRole('button', { name: 'Solicitações 1', exact: true }).click();
    await page.getByRole('button', { name: 'Aceitar Convite', exact: true }).click();
    await page.getByText('Nenhum convite pendente', { exact: true }).waitFor();
    await page.getByRole('button', { name: 'Todos 4', exact: true }).click();
    await page.getByRole('textbox', { name: 'Buscar amigos', exact: true }).fill('no-match');
    await page.getByRole('heading', { name: 'Nenhum amigo encontrado', exact: true }).waitFor();
    await page.getByRole('textbox', { name: 'Buscar amigos', exact: true }).fill('');
   }
   for (const close of await page.getByRole('button', { name: 'Fechar', exact: true }).all()) await close.click();
   await page.waitForFunction(() => !document.querySelector('.fixed.right-4.top-4 [role="status"]'));
   await page.screenshot({ path: `docs/validation/friends-redesign-${scenario.width}-${scenario.wallpaper ? 'wallpaper' : 'dark'}-${scenario.empty ? 'empty' : 'list'}.png` });
   assert.deepEqual(errors, []);
   console.log(JSON.stringify({ ...scenario, passed: true, compact: true, responsive: true, copyCode: true }));
   await context.close();
  }
 } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
