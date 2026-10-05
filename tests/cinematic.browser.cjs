const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({viewport:{width:1440,height:1000}});
 const dir='docs/visual/2026-10-03-shaders/'+Date.now();const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{localStorage.setItem('luxmc_lang','pt');localStorage.setItem('luxmc_cookie_consent','accepted')});
 await page.route('https://**/*',r=>new URL(r.request().url()).hostname === 'luxmc-r92.pages.dev' ? r.continue() : r.abort());
 await page.goto(process.env.LUXMC_SITE_URL || 'http://127.0.0.1:4174/');
 const stage=page.locator('#worldStage');
 await page.waitForFunction(()=>document.querySelector('#worldStage').dataset.sceneActive==='true');
 assert.equal(await page.locator('.scenic-panorama').count(),2);
 await page.locator('[data-skin-ready="true"]').first().waitFor();assert.equal(await page.locator('[data-skin-scene]').count(),3);
 assert.ok(await page.locator('#luxmcWorld').evaluate(img=>img.complete&&img.naturalWidth===1920));
 await stage.hover({position:{x:350,y:200}});await page.waitForTimeout(800);
 assert.notEqual(await stage.evaluate(n=>n.style.getPropertyValue('--scene-x')),'0');
 await page.locator('#worldMotion').click();assert.equal(await stage.getAttribute('data-scene-active'),'false');
 await page.locator('#worldRotate').click();await page.waitForFunction(()=>document.querySelector('#luxmcWorld').src.endsWith('city.webp')&&document.querySelector('#luxmcWorld').complete);
 await page.locator('#worldMotion').click();
 await page.waitForTimeout(1000);fs.mkdirSync(dir,{recursive:true});
 await page.screenshot({path:dir+'/desktop.png'});
 await page.locator('.scenic-panorama').first().scrollIntoViewIfNeeded();await page.waitForFunction(()=>document.querySelector('#worldStage').dataset.sceneActive==='false');
 assert.equal(await page.locator('.scenic-panorama').first().getAttribute('data-scene-active'),'true');await page.locator('.scenic-panorama').first().locator('[data-skin-ready="true"]').waitFor();await page.screenshot({path:dir+'/middle.png'});
 await page.locator('.scenic-panorama').last().scrollIntoViewIfNeeded();await page.locator('.scenic-panorama').last().locator('[data-skin-ready="true"]').waitFor();await page.waitForTimeout(400);await page.screenshot({path:dir+'/final.png'});
 await page.emulateMedia({reducedMotion:'reduce'});await page.waitForFunction(()=>[...document.querySelectorAll('.shader-scene')].every(n=>n.dataset.sceneActive==='false'));assert.equal(await page.locator('.scenic-panorama').last().getAttribute('data-scene-active'),'false');
 for(const width of [1024,768,390]){await page.setViewportSize({width,height:900});await page.evaluate(()=>scrollTo({top:0,behavior:'instant'}));await page.waitForTimeout(400);assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),'Overflow '+width)}
 await page.screenshot({path:dir+'/mobile.png'});
 assert.deepEqual(errors,[]);console.log('Shader screenshots: local assets, scene switch, perspective, pause, offscreen suspension, reduced motion, responsive layouts passed');await browser.close();
})().catch(e=>{console.error(e);process.exit(1)});

