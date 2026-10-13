const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 try {
 const page=await browser.newPage({viewport:{width:1440,height:1000}});
 await setupLauncherDemo(page);
 await page.route('https://luxmc-r92.pages.dev/api/player?*',route=>route.continue());
 await page.route('https://www.youtube-nocookie.com/**',route=>route.continue());
 await page.addInitScript(()=>{
  window.calls=[];window.versionsSettled=false;
  localStorage.setItem('luxmc.locale','pt-BR');
  const item={sourceId:'test-pack',source:'modrinth',slug:'test-pack',title:'Pack com vídeos',description:'Descrição',downloads:12,iconUrl:'/grass_head.png',bannerUrl:null,author:'Author',categories:['fabric'],versions:['1.21.1']};
  const original=window.electronAPI.invoke;
  const invoke=async(command,args)=>{
   window.calls.push({command,args});
   if(command==='mods_search')return [item];
   if(command==='mods_project_details')return {...item,id:item.sourceId,bodyType:'html',body:'<h2>Trailer do pacote</h2><iframe src="https://www.youtube.com/embed/M7lc1UVf-VE"></iframe><video src="/fixture.mp4" controls></video>',loaders:['fabric'],gameVersions:['1.21.1'],gallery:[],author:{name:'Author'}};
   if(command==='mods_versions'){await new Promise(resolve=>setTimeout(resolve,1200));window.versionsSettled=true;return [];}
   return original(command,args);
  };
  window.electronAPI.invoke=invoke;
 });
 await page.route('**/fixture.mp4',route=>route.fulfill({path:'website/assets/cinema/cherry-loop.mp4',contentType:'video/mp4'}));
 await page.goto((process.env.LUXMC_BASE_URL||'http://127.0.0.1:1420')+'/mods?type=modpack');
 await page.getByText('Pack com vídeos',{exact:true}).click();
 await page.getByRole('heading',{name:'Trailer do pacote'}).waitFor();
 assert.equal(await page.evaluate(()=>window.versionsSettled),false,'overview should not wait for all versions');
 await page.locator('video').evaluate(async video=>{video.muted=true;await video.play()});
 await page.waitForFunction(()=>document.querySelector('video').currentTime>.1);
 const youtube=page.waitForRequest(request=>request.url().startsWith('https://www.youtube-nocookie.com/embed/M7lc1UVf-VE'),{timeout:30000});
 youtube.catch(()=>{});
 await page.locator('.btn-play-video').click();
 const request=await youtube;
 assert.equal((await request.allHeaders()).referer,'https://luxmc-r92.pages.dev/');
 const iframe=page.locator('.video-player-container iframe');
 assert.equal(await iframe.getAttribute('src'),'https://luxmc-r92.pages.dev/api/player?video=M7lc1UVf-VE');
 console.log('Overview loads before versions; direct MP4 advances; embedded YouTube is requested through the live HTTPS player with the correct Referer. External YouTube playback restrictions are not bypassed.');
 } finally { await browser.close(); }
})().catch(error=>{console.error(error);process.exitCode=1;});
