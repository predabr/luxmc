const {chromium} = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
 const browser = await chromium.launch({headless:true, executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const evidence=[];
 try {
  fs.mkdirSync('docs/validation/general-ui', {recursive:true});
  for (const theme of ['dark','light']) for (const wallpaper of [false,true]) {
   const page=await browser.newPage({viewport:{width:1440,height:960},reducedMotion:'reduce'});
   const errors=[]; page.on('pageerror', error=>errors.push(String(error)));
   await setupLauncherDemo(page);
   await page.addInitScript(({theme,wallpaper})=>{
    const config={...window.launcherDemo.state.settings,theme:theme==='light'?'default-light':'default-dark',language:'en',languageMode:'manual',animations:false,blur:false,interfaceOpacity:18};
    if(wallpaper){localStorage.setItem('luxmc_background','custom');localStorage.setItem('luxmc_custom_wallpaper','/modpack_vanilla_perfected.png');localStorage.setItem('luxmc_custom_wallpaper_type','image');config.customBackground='custom';config.customWallpaperUrl='/modpack_vanilla_perfected.png';}
    window.launcherDemo.state.settings=config;
    const original=window.electronAPI.invoke;
    const invoke=async(command,args)=>{
     if(command==='plugin:store|get' && args.key==='app')return[config,true];
     if(command==='settings_get')return config;
     return original(command,args);
    };
    window.electronAPI.invoke=invoke;
    window.__TAURI_INTERNALS__.invoke=invoke;
   },{theme,wallpaper});
   await page.goto('http://127.0.0.1:1420/');
   try { await page.locator('.home-instance-grid').waitFor(); } catch(error) { await page.screenshot({path:'docs/validation/general-ui/debug.png'}); console.log(await page.locator('body').innerText()); throw error; }
   await page.waitForFunction(theme=>document.documentElement.classList.contains('light')===(theme==='light'),theme);
   if(wallpaper){
    await page.evaluate(async()=>{const {themeStore}=await import('/src/lib/stores/theme.svelte.ts');themeStore.setCustomWallpaper('/modpack_vanilla_perfected.png','image',undefined,false);});
    await page.waitForFunction(()=>document.documentElement.classList.contains('has-custom-wallpaper'));
   }
   const select=page.locator('.glass-select-trigger').first();
   await select.click();
   await page.getByRole('listbox').waitFor();
   await page.keyboard.press('End');
   assert.equal(await page.evaluate(()=>document.activeElement?.getAttribute('role')),'option');
   await page.keyboard.press('Escape');
   assert.equal(await select.getAttribute('aria-expanded'),'false');
   await page.screenshot({path:`docs/validation/general-ui/home-${theme}-${wallpaper?'wallpaper':'plain'}.png`});
   for(const route of ['mods','friends','settings']){
    await page.goto(`http://127.0.0.1:1420/${route}`);
    await page.locator('main').waitFor(); await page.waitForTimeout(800);
    const overflow=await page.locator('main').evaluate(element=>element.scrollWidth-element.clientWidth);
    assert.ok(overflow<=1,`${route} overflow: ${overflow}`);
    await page.screenshot({path:`docs/validation/general-ui/${route}-${theme}-${wallpaper?'wallpaper':'plain'}.png`});
   }
   assert.deepEqual(errors,[]);
   evidence.push({theme,wallpaper,keyboardSelection:true,overflow:false,errors});
   await page.close();
  }
  fs.writeFileSync('docs/validation/general-ui-2026-10-06.json',JSON.stringify(evidence,null,2));
  console.log(JSON.stringify(evidence));
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
