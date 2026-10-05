const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

const rgb = value => value.match(/[\d.]+/g).slice(0, 3).map(Number);
const luminance = channels => channels.map(channel => channel / 255).map(channel => channel <= .04045 ? channel / 12.92 : ((channel + .055) / 1.055) ** 2.4).reduce((value, channel, index) => value + channel * [.2126, .7152, .0722][index], 0);
const contrast = (foreground, background) => {
 const values = [luminance(rgb(foreground)), luminance(rgb(background))].sort((a,b)=>b-a);
 return (values[0]+.05)/(values[1]+.05);
};

(async () => {
 const browser = await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 try {
  const page = await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
  const errors=[];page.on('pageerror',error=>errors.push(String(error)));
  await setupLauncherDemo(page);
  await page.addInitScript(()=>{
   const original=window.electronAPI.invoke;
   const wrapped=async(command,args)=>{
    if(command==='plugin:dialog|open')return 'C:/Downloads/Wallpaper.png';
    if(command==='wallpaper_import')return window.launcherDemo.textures.grass;
    return original(command,args);
   };
   window.electronAPI.invoke=wrapped;window.__TAURI_INTERNALS__.invoke=wrapped;
  });
  await page.goto('http://127.0.0.1:1420/');await page.locator('.home-instance-card').first().waitFor();
  assert.ok((await page.locator('.home-instance-card').first().boundingBox()).height>=280);
  assert.ok((await page.locator('.home-instance-card img[src="/grass_block.png"]').boundingBox()).width>=110);
  for (const theme of ['light','dark']) for (const wallpaper of [false,true]) {
   await page.locator('a[href="/settings"]').first().click();
   await page.getByRole('tab',{name:/Aparência/}).click();
   await page.getByRole('button',{name:theme==='light'?/Claro.*Neve/:/Escuro.*Obsidiana/}).click();
   if(wallpaper)await page.getByTitle('Importe qualquer vídeo MP4/WebM ou imagem PNG/JPG/GIF/WebP').click();
   else await page.getByRole('button',{name:'Fortaleza de Obsidiana',exact:true}).click();
   await page.getByRole('button',{name:'Início',exact:true}).click();
   await page.waitForTimeout(450);
   for(const close of await page.locator('[aria-label="Fechar"]').all())if(await close.isVisible())await close.click();
   await page.screenshot({path:`docs/validation/home-${theme}-${wallpaper?'wallpaper':'plain'}.png`});
   await page.getByRole('button',{name:/Nova instância/i}).last().click();
   const dialog=page.getByRole('dialog');await dialog.waitFor();
   await page.waitForTimeout(400);
   const selected=dialog.locator('.selection-control').last();
   await selected.waitFor();
   const colors=await selected.evaluate(node=>({foreground:getComputedStyle(node).color,background:getComputedStyle(node).backgroundColor}));
   assert.ok(contrast(colors.foreground,colors.background)>=4.5,`${theme}/${wallpaper} selected contrast: ${JSON.stringify(colors)}`);
   const create=dialog.getByRole('button',{name:'Criar instância',exact:true});
   assert.ok(await create.isEnabled());
   if(wallpaper){const colors=await create.evaluate(node=>({foreground:getComputedStyle(node).color,background:getComputedStyle(node).backgroundColor}));assert.ok(contrast(colors.foreground,colors.background)>=4.5);}
   await page.screenshot({path:`docs/validation/create-${theme}-${wallpaper?'wallpaper':'plain'}.png`});
   await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'});
  }
  await page.setViewportSize({width:960,height:900});
  await page.getByRole('button',{name:/Nova instância/i}).last().click();
  const create=page.getByRole('dialog').getByRole('button',{name:'Criar instância',exact:true});
  await create.click();await page.waitForFunction(()=>window.launcherDemo.calls.some(call=>call.command==='profiles_create'));
  const created=await page.evaluate(()=>window.launcherDemo.calls.find(call=>call.command==='profiles_create').args.input);
  assert.equal(created.loader,'vanilla');assert.equal(created.mcVersion,'1.21.1');
  await page.waitForTimeout(1500);
  await page.locator('a[href="/settings"]').first().click();
  await page.getByRole('tab',{name:/Aparência/}).click();
  await page.getByRole('button',{name:/Claro.*Neve/}).click();
  for(const route of ['friends','mods','skins','instances']){
   await page.locator(`a[href="/${route}"]`).first().click();
   await page.waitForTimeout(500);
   assert.ok(await page.locator('html').evaluate(node=>node.classList.contains('light')));
   await page.screenshot({path:`docs/validation/light-review-${route}.png`});
  }
  const news=await browser.newPage({viewport:{width:1440,height:1000}});
  news.on('pageerror',error=>errors.push(String(error)));
  await news.clock.install();await setupLauncherDemo(news);
  await news.addInitScript(()=>{
   localStorage.removeItem('luxmc_minecraft_news_v2');window.newsCalls=0;window.newsFailure=false;
   const original=window.electronAPI.invoke;
   const wrapped=async(command,args)=>{
    if(command==='minecraft_news'){
     window.newsCalls++;
     if(window.newsFailure)throw Error('offline');
     return {entries:[{id:'live-news-'+window.newsCalls,title:'Notícia oficial '+window.newsCalls,date:'2026-10-05',readMoreLink:'https://www.minecraft.net/article/latest',newsPageImage:{url:'/v2/images/latest.jpg'}}]};
    }
    return original(command,args);
   };
   window.electronAPI.invoke=wrapped;window.__TAURI_INTERNALS__.invoke=wrapped;
  });
  await news.goto('http://127.0.0.1:1420/');await news.waitForFunction(()=>window.newsCalls===1);
  await news.getByText('Notícia oficial 1',{exact:true}).waitFor();
  await news.clock.fastForward(29*60*1000);assert.equal(await news.evaluate(()=>window.newsCalls),1);
  await news.clock.fastForward(2*60*1000);await news.waitForFunction(()=>window.newsCalls===2);
  await news.getByText('Notícia oficial 2',{exact:true}).waitFor();
  await news.evaluate(()=>{window.newsFailure=true;});await news.clock.fastForward(31*60*1000);
  await news.waitForFunction(()=>window.newsCalls===3);await news.getByText('Notícia oficial 2',{exact:true}).waitFor();
  await news.clock.fastForward(2*60*1000);assert.equal(await news.evaluate(()=>window.newsCalls),3);
  await news.evaluate(()=>{window.newsFailure=false;window.dispatchEvent(new Event('online'));});await news.waitForFunction(()=>window.newsCalls===4);
  await news.getByText('Notícia oficial 4',{exact:true}).waitFor();
  assert.deepEqual(errors,[]);
  console.log('PASS: larger home cards and vanilla icon, light/dark with and without wallpaper, readable selected versions and create action, responsive Vanilla creation, automatic news refresh, offline retention/backoff and online recovery.');
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1);});
