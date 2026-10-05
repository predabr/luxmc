const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({locale:'pt-BR',viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',error=>errors.push(String(error)));
 const siteUrl=process.env.LUXMC_SITE_URL || 'http://127.0.0.1:8765/';
 await page.route('https://**/*',route=>new URL(route.request().url()).origin===new URL(siteUrl).origin?route.continue():route.abort());
 await page.route('**/api/latest-release',route=>route.fulfill({json:{tag_name:'v2.0.2',assets:[]}}));
 await page.addInitScript(()=>localStorage.setItem('luxmc_cookie_consent','accepted'));
 await page.goto(siteUrl);
 const group=page.getByRole('group',{name:'Conheça o launcher'});
 for(const label of ['01 · Sua biblioteca','02 · Seus mods','03 · Seu personagem']){
  const button=group.getByRole('button',{name:label,exact:true});await button.click();
  assert.equal(await button.getAttribute('aria-pressed'),'true');
  await page.waitForFunction(()=>{const image=document.getElementById('productTourImage');return image.complete&&image.naturalWidth>0;});
 }
 fs.mkdirSync('docs/visual/2026-10-04-catalog',{recursive:true});
 await page.locator('.product-tour').screenshot({path:'docs/visual/2026-10-04-catalog/site-tour.png'});
 for(const width of [390,768,1440]){await page.setViewportSize({width,height:900});assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));}
 assert.deepEqual(errors,[]);console.log('Real launcher tour: all screenshots loaded; selection works; no horizontal overflow at mobile, tablet and desktop; no JS errors.');
 await browser.close();
})().catch(error=>{console.error(error);process.exit(1)});
