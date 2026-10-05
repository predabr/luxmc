const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
(async () => {
 const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
 try {
  for (const scenario of [
   { type: 'image', blur: true }, { type: 'image', blur: false },
   { type: 'video', blur: true }, { type: 'video', blur: false },
   { type: 'video', blur: false, performanceMode: true }
  ]) {
   const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
   const page = await context.newPage();
   const errors = [];
   page.on('pageerror', error => errors.push(String(error)));
   await page.addInitScript(scenario => {
    localStorage.setItem('luxmc_background', 'custom');
    localStorage.setItem('luxmc_custom_wallpaper', `http://127.0.0.1:1420/fixture-wallpaper.${scenario.type === 'video' ? 'mp4' : 'png'}`);
    localStorage.setItem('luxmc_custom_wallpaper_type', scenario.type);
    const profile = { id: 'vp', name: 'Vanilla Perfected', icon: '/modpack_vanilla_perfected_icon.webp', loader: 'fabric', mcVersion: '26.2', gameDir: 'C:\\Minecraft\\vp', createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
    const invoke = async command => {
     if (command === 'app_init') return { account: { id: 'offline_test', uuid: 'offline_test', username: 'Steve', accessToken: '' }, profiles: [profile], activeProfileId: 'vp', devMode: true };
     if (command === 'profiles_list') return [profile];
     if (command === 'plugin:store|get') return [{ language: 'pt-BR', languageMode: 'manual', animations: false, blur: scenario.blur, performanceMode: scenario.performanceMode || false, interfaceOpacity: 18, pauseWallpaperOnBlur: false }, true];
     if (command === 'plugin:store|load') return 1;
     if (command === 'mods_search') return [{ sourceId: 'test', source: 'modrinth', title: 'Test modpack', slug: 'test', description: 'Test', downloads: 100, iconUrl: '/modpack_vanilla_perfected_icon.webp', author: 'Test', categories: ['fabric'], versions: ['26.2'] }];
     if (command === 'curseforge_status') return true;
     if (command === 'java_scan') return { runtimes: [] };
     if (command === 'get_system_specs') return { totalRamMb: 8192, osDistro: 'Windows 11', arch: 'x86_64' };
     return [];
    };
    window.electronAPI = { invoke, on: () => () => {} };
    window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 1, convertFileSrc: path => path };
   }, scenario);
   await page.route('**/fixture-wallpaper.png', route => route.fulfill({ path: 'static/modpack_vanilla_perfected.png', contentType: 'image/png' }));
   await page.route('**/fixture-wallpaper.mp4', route => route.fulfill({ path: 'website/assets/cinema/cherry-loop.mp4', contentType: 'video/mp4' }));
   await page.goto('http://127.0.0.1:1420/');
   await page.waitForFunction(() => document.documentElement.classList.contains('has-custom-wallpaper'));
   await page.locator('main [class~="bg-bg/35"]').first().waitFor();
   await page.waitForFunction(expected => document.documentElement.classList.contains('no-blur') === expected, !scenario.blur || Boolean(scenario.performanceMode));
   const measure = () => page.evaluate(() => {
    const targets = [...document.querySelectorAll('main [class~="bg-bg/35"], aside')].filter(element => element.getBoundingClientRect().height > 0 && !element.closest('[role=dialog]'));
    return targets.map(element => ({ alpha: Number(getComputedStyle(element).backgroundColor.match(/,\s*([\d.]+)\)$/)?.[1] || 1), opacity: getComputedStyle(element).opacity }));
   });
   const initial = await measure();
   assert.ok(initial.length >= 4);
   assert.ok(initial.every(item => Math.abs(item.alpha - .18) < .001), JSON.stringify({ scenario, initial }));
   assert.ok(initial.every(item => item.opacity === '1'));
   if (scenario.type === 'video' && scenario.performanceMode) await page.waitForFunction(() => { const video = document.querySelector('video'); return video && video.paused; });
   if (scenario.type === 'video' && !scenario.performanceMode) await page.waitForFunction(() => { const video = document.querySelector('video'); return video && video.currentTime > .1 && !video.paused; });
   await page.goto('http://127.0.0.1:1420/settings');
   await page.getByRole('tab', { name: 'Aparência', exact: false }).click();
   const opacity = page.getByLabel('Opacidade da interface com wallpaper', { exact: false });
   await opacity.fill('45');
   await opacity.dispatchEvent('input');
   await page.locator('aside button[title="Início"]').first().click();
   await page.waitForFunction(() => Math.abs(Number(getComputedStyle(document.documentElement).getPropertyValue('--interface-opacity')) - .45) < .001);
   const changed = await measure();
   assert.ok(changed.every(item => Math.abs(item.alpha - .45) < .001));
   await page.goto('http://127.0.0.1:1420/mods?type=modpack');
   await page.getByText('Test modpack', { exact: true }).first().waitFor();
   const catalog = page.locator('main .catalog-card, main .luxmc-card, main .surface-glass').first();
   await catalog.waitFor();
   assert.ok(Math.abs(await catalog.evaluate(element => Number(getComputedStyle(element).backgroundColor.match(/,\s*([\d.]+)\)$/)?.[1])) - .18) < .001);
   await page.goto('http://127.0.0.1:1420/');
   await page.locator('main [class~="bg-bg/35"]').first().waitFor();
   if (scenario.type === 'image' && !scenario.blur) await page.screenshot({ path: 'docs/validation/wallpaper-transparency-home.png' });
   await page.evaluate(() => { document.documentElement.classList.remove('has-custom-wallpaper'); document.documentElement.classList.add('no-blur'); });
   if (!scenario.blur) {
    const plain = await measure();
    assert.ok(plain.some(item => item.alpha >= .94), JSON.stringify({plain, classes: await page.evaluate(() => document.documentElement.className)}));
   }
   assert.deepEqual(errors, []);
   console.log(JSON.stringify({ ...scenario, passed: true, panelsChecked: initial.length }));
   await context.close();
  }
 } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exit(1); });





