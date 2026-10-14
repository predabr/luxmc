const {chromium}=require('playwright-core');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:900}});
  page.setDefaultTimeout(20000);
  await setupLauncherDemo(page);
  await page.addInitScript(()=>{
   const invoke=window.electronAPI.invoke;
   window.artworkCalls=[];
   window.electronAPI.invoke=async(command,args={})=>{
    if(command==='mods_search')return Array.from({length:36},(_,i)=>({source:'modrinth',sourceId:`viewport-${i}`,slug:`viewport-${i}`,title:`Viewport Pack ${i}`,description:'Viewport artwork test',downloads:1000-i,iconUrl:'/modpack_fo_icon.png',bannerUrl:null,author:'Fixture',categories:['fabric'],versions:['1.20.1']}));
    if(command==='mods_project_details'){window.artworkCalls.push(args.projectId);return{id:args.projectId,source:'modrinth',gallery:[],body:'',iconUrl:'/modpack_fo_icon.png'};}
    return invoke(command,args);
   };
  });
  await page.goto('http://127.0.0.1:1420/mods');
  await page.waitForFunction(()=>window.artworkCalls.length>0);
  await page.waitForTimeout(500);
  const initial=await page.evaluate(()=>[...window.artworkCalls]);
  const rendered=await page.locator('.catalog-card').count();
  assert.ok(initial.length<rendered,`Overscan galleries loaded: ${initial.length}/${rendered}`);
  assert.ok(!initial.includes('viewport-35'));
  await page.evaluate(()=>{const root=document.querySelector('[data-catalog-results]').closest('main');root.scrollTop=root.scrollHeight;});
  await page.locator('.catalog-card[aria-label="Viewport Pack 35"]').scrollIntoViewIfNeeded();
  await page.waitForFunction(()=>window.artworkCalls.includes('viewport-35'));
  console.log('PASS offscreen cards defer gallery requests and load on scroll',{initialGalleries:initial.length,initialRenderedCards:rendered,totalCards:36});
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
