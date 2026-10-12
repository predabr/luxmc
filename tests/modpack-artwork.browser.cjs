const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
(async () => {
 const browser = await chromium.launch({ headless: true, executablePath: process.env.LUXMC_CHROMIUM_EXECUTABLE });
 try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 }, reducedMotion: 'reduce' });
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await setupLauncherDemo(page);
  await page.addInitScript(() => {
   const profile = { ...window.launcherDemo.state.profiles.find(item=>item.name==='Vanilla Perfected'), id:'vp', banner:undefined };
   window.launcherDemo.state.profiles=[profile];
   window.launcherDemo.state.selectedProfileId='vp';
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



