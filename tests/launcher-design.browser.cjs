const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');

(async () => {
 const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
 try {
  for (const wallpaper of [false, true]) {
   const page = await browser.newPage({ viewport: { width: 1600, height: 1000 }, reducedMotion: 'reduce' });
   const errors = [];
   page.on('pageerror', error => errors.push(String(error)));
   await page.addInitScript(wallpaper => {
    if (wallpaper) {
     localStorage.setItem('luxmc_background', 'custom');
     localStorage.setItem('luxmc_custom_wallpaper', '/modpack_vanilla_perfected.png');
     localStorage.setItem('luxmc_custom_wallpaper_type', 'image');
    }
    const profile = { id: 'vp', name: 'Vanilla Perfected', loader: 'fabric', icon: '/modpack_vanilla_perfected_icon.webp', mcVersion: '26.2', gameDir: 'C:\\Minecraft\\vp', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
    const me = { id: 'test', username: 'Steve' };
    const invoke = async (command, args) => {
     if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'test', username: 'Steve', accessToken: '' }, profiles: [profile], activeProfileId: profile.id, devMode: true };
     if (command === 'profiles_list') return [profile];
     if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
     if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, blur: false, interfaceOpacity: 18 }, true];
     if (command === 'social_request') return { me, friends: [] };
     if (command === 'tunnel_status') return null;
     if (command === 'get_system_specs') return { totalRamMb: 16384, osDistro: 'Windows 11', arch: 'x86_64' };
     if (command === 'java_scan') return { runtimes: [] };
     return [];
    };
    window.electronAPI = { invoke, on: () => () => {} };
    window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
   }, wallpaper);
   await page.route('https://**/*', route => route.abort());
   await page.goto('http://127.0.0.1:1420/');
   const sidebar = page.locator('.launcher-sidebar');
   await page.getByRole('button', { name: 'Expandir menu', exact: true }).click();
   await page.waitForFunction(() => document.querySelector('.launcher-sidebar').getBoundingClientRect().width >= 239);
   const profile = sidebar.getByRole('link', { name: 'Vanilla Perfected', exact: true });
   assert.ok((await profile.boundingBox()).width > 200);
   await profile.getByText('26.2 · fabric', { exact: true }).waitFor();
   await sidebar.getByText('Portal Luxmc', { exact: true }).waitFor();
   const overflow = await sidebar.evaluate(aside => {
    const box = aside.getBoundingClientRect();
    return [...aside.querySelectorAll('a,button')].filter(element => element.getBoundingClientRect().width > 0 && !element.closest('[role=dialog]')).filter(element => {
     const bounds = element.getBoundingClientRect();
     return bounds.left < box.left - .5 || bounds.right > box.right + .5;
    }).map(element => element.textContent);
   });
   assert.deepEqual(overflow, []);
   await page.setViewportSize({ width: 960, height: 760 });
   await page.locator('.home-instance-grid').waitFor();
   assert.ok((await page.locator('.home-instance-grid > div').first().boundingBox()).width >= 200);
   const contentOverflow = await page.locator('main').evaluate(element => element.scrollWidth - element.clientWidth);
   assert.ok(contentOverflow <= 1, `Home overflow at minimum width: ${contentOverflow}`);
   await page.screenshot({ path: `docs/validation/launcher-design-small-${wallpaper ? 'wallpaper' : 'dark'}.png` });
   await page.setViewportSize({ width: 1600, height: 1000 });
   await page.goto('http://127.0.0.1:1420/friends');
   const addFriend = page.getByRole('button', { name: 'Adicionar amigo', exact: true });
   await addFriend.waitFor();
   assert.equal(await addFriend.count(), 1);
   assert.ok((await addFriend.boundingBox()).height >= 48);
   await page.getByRole('heading', { name: 'Sua turma começa com um convite' }).waitFor();
   const tab = page.getByRole('button', { name: 'Solicitações', exact: true });
   await tab.click();
   assert.equal(await tab.getAttribute('aria-pressed'), 'true');
   if (wallpaper) {
    const alpha = await addFriend.evaluate(element => Number(getComputedStyle(element).backgroundColor.match(/,\s*([\d.]+)\)$/)?.[1]));
    assert.ok(Math.abs(alpha - .18) < .001);
   }
   await page.screenshot({ path: `docs/validation/launcher-design-${wallpaper ? 'wallpaper' : 'dark'}.png` });
   assert.deepEqual(errors, []);
   await page.close();
  }
  const page = await browser.newPage({ viewport: { width: 1280, height: 1000 }, reducedMotion: 'reduce' });
  await page.addInitScript(() => {
   window.authCalls = 0;
   const invoke = async (command, args) => {
    if (command === 'app_init') return { account: null, profiles: [], activeProfileId: null, devMode: false };
    if (command === 'plugin:store|load' || command === 'plugin:event|listen') return 1;
    if (command === 'plugin:store|get') return args?.key === 'current_account' ? [null, false] : [{ language: 'pt-BR', languageMode: 'manual', animations: false }, true];
    if (command === 'auth_get_client_id') return 'test-client';
    if (command === 'auth_login') {
     window.authCalls += 1;
     return new Promise((resolve, reject) => { window.rejectLogin = () => reject(Error('Test login stopped')); });
    }
    if (command === 'java_scan') return { runtimes: [] };
    return null;
   };
   window.electronAPI = { invoke, on: () => () => {} };
   window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
  });
  await page.route('https://**/*', route => route.abort());
  await page.goto('http://127.0.0.1:1420/');
  const signIn = page.locator('button.launcher-button--microsoft');
  await signIn.waitFor();
  assert.ok((await signIn.boundingBox()).height >= 56);
  await page.screenshot({ path: 'docs/validation/launcher-design-login.png' });
  await signIn.click();
  await page.getByText('Conclua o login no navegador', { exact: true }).waitFor();
  assert.equal(await page.evaluate(() => window.authCalls), 1);
  assert.equal(await page.locator('.launcher-tab:disabled').count(), 2);
  await page.evaluate(() => window.rejectLogin());
  await signIn.waitFor();
  console.log(JSON.stringify({ passed: true, sidebarResponsive: true, singleFriendAction: true, wallpaperOpacity: true, microsoftConcurrency: true }));
 } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });
