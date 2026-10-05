const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
(async () => {
 const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
 try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
   const profile = { id: 'vp', name: 'Vanilla Perfected', icon: '/modpack_vanilla_perfected_icon.webp', loader: 'fabric', loaderVersion: '0.19.3', mcVersion: '26.2', gameDir: 'C:\\Minecraft\\vp', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), modCount: 81 };
   const invoke = async command => {
    if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' }, profiles: [profile], activeProfileId: 'vp', devMode: true };
    if (command === 'profiles_list') return [profile];
    if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false }, true];
    if (command === 'plugin:store|load') return 1;
    if (['deep_links_take', 'changelog_get', 'instance_get_mods', 'instance_get_resourcepacks', 'instance_get_shaders'].includes(command)) return [];
    if (command === 'java_scan') return { runtimes: [] };
    if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64' };
    return [];
   };
   window.electronAPI = { invoke, on: () => () => {} };
   window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
  });
  await page.goto('http://127.0.0.1:1420/');
  await page.locator('img[src="/modpack_vanilla_perfected_icon.webp"]').first().waitFor();
  assert.ok(await page.locator('img[src="/modpack_vanilla_perfected_icon.webp"]').count() >= 2);
  await page.goto('http://127.0.0.1:1420/instances/vp');
  await page.getByRole('heading', { name: 'Vanilla Perfected', exact: true }).waitFor();
  await page.locator('.instance-photo > img[src="/modpack_vanilla_perfected.png"]').waitFor();
  await page.waitForFunction(() => Array.from(document.querySelectorAll('img[src*="modpack_vanilla_perfected"]')).every(image => image.complete && image.naturalWidth > 0));
  assert.equal(await page.locator('img[src="/vanilla_banner.png"]').count(), 0);
  assert.deepEqual(errors, []);
  await page.screenshot({ path: 'docs/validation/modpack-artwork-browser.png' });
  console.log('Official Vanilla Perfected logo renders in home/sidebar; instance header uses the dedicated banner; all artwork images load successfully.');
 } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });



