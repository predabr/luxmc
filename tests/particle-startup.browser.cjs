const {chromium}=require('playwright-core');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 try{
  const page=await browser.newPage();page.setDefaultTimeout(20000);await setupLauncherDemo(page);
  await page.addInitScript(()=>{window.particleFrames=0;const clear=CanvasRenderingContext2D.prototype.clearRect;CanvasRenderingContext2D.prototype.clearRect=function(...args){if(this.canvas.classList.contains('fixed'))window.particleFrames++;return clear.apply(this,args);};});
  await page.goto('http://127.0.0.1:1420/');
  await page.evaluate(async()=>{const {settings}=await import('/src/lib/stores/settings.svelte.ts');window.motionSettings=settings;window.motionState=(await import('/src/lib/stores/app.svelte.ts')).appState;settings.value={...settings.value,liveWallpaper:true,wallpaperFps:30,pauseWallpaperOnBlur:false};});
  await page.waitForFunction(()=>window.particleFrames>5);
  const sample=async()=>{const first=await page.evaluate(()=>window.particleFrames);await page.waitForTimeout(1000);return await page.evaluate(()=>window.particleFrames)-first;};
  const normal=await sample();await page.evaluate(()=>window.motionSettings.value={...window.motionSettings.value,wallpaperFps:10});const low=await sample();
  assert.ok(normal>15&&low>4&&low<normal*.7,JSON.stringify({normal,low}));
  await page.evaluate(()=>window.motionState.isLaunching=true);await page.waitForTimeout(100);assert.equal(await sample(),0);await page.evaluate(()=>window.motionState.isLaunching=false);assert.ok(await sample()>4);
  await page.evaluate(()=>window.motionState.isGameRunning=true);await page.waitForTimeout(100);const paused=await sample();assert.equal(paused,0);
  await page.evaluate(()=>window.motionState.isGameRunning=false);await page.waitForFunction(()=>window.particleFrames>0);assert.ok(await sample()>4);
  console.log('PASS real particle canvas follows FPS setting, pauses during launch and game, then resumes',{normal,low,paused});
 }finally{await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
