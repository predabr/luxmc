const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const luminance = value => value.match(/[\d.]+/g).slice(0,3).map(Number).map(channel=>channel/255).map(channel=>channel<=.04045?channel/12.92:((channel+.055)/1.055)**2.4).reduce((sum,channel,index)=>sum+channel*[.2126,.7152,.0722][index],0);

(async () => {
 const browser = await chromium.launch({headless:true, executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const evidence = [];
 try {
  fs.mkdirSync('docs/validation/final-polish', {recursive:true});
  for (const theme of ['light', 'dark']) {
   const page = await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
   const errors = []; page.on('pageerror', error => errors.push(String(error)));
   await setupLauncherDemo(page);
   await page.addInitScript(theme => {
    const state = window.launcherDemo.state;
    const config = {...state.settings,theme:theme === 'light' ? 'default-light' : 'default-dark',language:'en',languageMode:'manual',animations:false};
    state.settings = config;
    state.profiles[1].name = 'Vulkan Optimized'; state.profiles[1].icon = 'grass_block';
    const original = window.electronAPI.invoke;
    const invoke = async (command, args) => {
     if(command === 'settings_get') return config;
     if(command === 'plugin:store|get' && args.key === 'app') return [config,true];
     if(command === 'mods_search' && args.query === 'Vulkan Optimized') return [{title:'Vulkan Optimized',slug:'vulkan-optimized',iconUrl:'https://cdn.modrinth.com/data/u76hn1pF/742ca1e06c3c27f3b61be05135cf857e78420aad_96.webp'}];
     if(command === 'mods_search') return Array.from({length:36},(_,index)=>({slug:`pack-${index}`,sourceId:`pack-${index}`,source:'modrinth',title:`Adventure pack ${index}: a complete exploration experience`,description:'Explore and build together with a carefully curated collection of mods.',downloads:12000,iconUrl:'/modpack_fo_icon.png',bannerUrl:index === 1 ? null : '/modpack_fo.webp',author:'Pack author',categories:['fabric'],versions:['1.21.1']}));
     return original(command,args);
    };
    window.electronAPI.invoke = invoke; window.__TAURI_INTERNALS__.invoke = invoke;
   },theme);
   await page.goto('http://127.0.0.1:1420/');
   await page.waitForFunction(()=>window.launcherDemo.state.profiles[1].icon.startsWith('https://cdn.modrinth.com/'));
   assert.equal(await page.evaluate(()=>window.launcherDemo.state.profiles[0].icon), '/grass_block.png');
   await page.getByRole('button',{name:/New instance/i}).last().click();
   const create = page.getByRole('dialog'); await create.waitFor();
   assert.ok(await create.evaluate(node=>node.scrollWidth<=node.clientWidth));
   await page.screenshot({path:`docs/validation/final-polish/create-${theme}.png`});
   await page.keyboard.press('Escape');
   await page.goto('http://127.0.0.1:1420/mods');
   await page.locator('[data-catalog-logo]').first().waitFor();
   await page.waitForTimeout(500);
   const selectedColors = await page.getByRole('button',{name:'Modpack',exact:true}).evaluate(node=>({fg:getComputedStyle(node).color,bg:getComputedStyle(node).backgroundColor}));
   const values=[luminance(selectedColors.fg),luminance(selectedColors.bg)].sort((a,b)=>b-a);
   assert.ok((values[0]+.05)/(values[1]+.05)>=4.5);
   const geometry = await page.locator('.catalog-card').first().evaluate(card=>{
    const logo=card.querySelector('[data-catalog-logo]').getBoundingClientRect();
    const rect=card.getBoundingClientRect(); const title=card.querySelector('h3').getBoundingClientRect();
    return {inside:logo.top>=rect.top && logo.bottom<=rect.bottom,clearTitle:logo.bottom<=title.top,footer:card.querySelector('button').getBoundingClientRect().bottom<=rect.bottom};
   });
   assert.deepEqual(geometry,{inside:true,clearTitle:true,footer:true});
   assert.equal(await page.locator('.catalog-card').nth(1).locator('img').first().evaluate(node=>getComputedStyle(node).objectFit),'contain');
   assert.ok(await page.locator('aside').last().evaluate(node=>node.scrollWidth<=node.clientWidth));
   await page.screenshot({path:`docs/validation/final-polish/catalog-${theme}.png`});
   const initial = await page.locator('main').evaluate(node=>node.scrollTop);
   await page.locator('.catalog-card').first().hover(); await page.mouse.wheel(0,1600);
   await page.waitForTimeout(350);
   const scrolled = await page.locator('main').evaluate(node=>node.scrollTop);
   assert.ok(scrolled > initial + 500);
   assert.ok(await page.locator('.catalog-card').count() <= 27);
   await page.locator('main').evaluate(node=>node.scrollTop=0); await page.waitForTimeout(150);
   await page.locator('.catalog-card button').first().click();
   const installer=page.getByRole('dialog'); await installer.waitFor();
   await page.waitForTimeout(250);
   assert.ok(await installer.evaluate(node=>node.scrollWidth<=node.clientWidth));
   assert.ok(await installer.locator('h2').evaluate(node=>!node.classList.contains('truncate')));
   await page.screenshot({path:`docs/validation/final-polish/installer-${theme}.png`});
   await page.setViewportSize({width:960,height:720});
   const installButton=installer.getByRole('button',{name:'Install modpack',exact:true});
   await installButton.scrollIntoViewIfNeeded(); assert.ok(await installButton.isVisible());
   await page.screenshot({path:`docs/validation/final-polish/installer-small-${theme}.png`});
   await page.goto('http://127.0.0.1:1420/news');
   await page.getByRole('button',{name:/Luxmc.*\(/i}).click();
   await page.getByText('Library and appearance refinements',{exact:true}).waitFor();
   assert.deepEqual(errors,[]);
   evidence.push({theme,artworkRecovered:true,geometry,scrollDelta:scrolled-initial,errors});
   await page.close();
  }
  fs.writeFileSync('docs/validation/final-polish-ui.json',JSON.stringify(evidence,null,2));
  console.log(JSON.stringify(evidence));
 } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exit(1)});
