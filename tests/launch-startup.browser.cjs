const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const errors=[];const results=[];
 try {
  const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});page.on('pageerror',e=>errors.push(String(e)));
  await setupLauncherDemo(page);
  await page.addInitScript(()=>{
   window.launcherDemo.state.settings.launcherActionOnLaunch='hide_reopen';
   const original=window.electronAPI.invoke;
   window.launchCases=[];window.gameReady=false;window.earlyExit=false;window.gameAlive=false;
   const invoke=async(command,args)=>{
    window.launchCases.push(command);
    if(command==='launch_game') {
     if(window.earlyExit) { window.launcherDemo.emit('game-exit',{versionId:'1.21.1',code:1,success:false,errorMessage:'Java falhou no início'});return {pid:321}; }
     window.gameAlive=true;return {pid:321};
    }
    if(command==='game_launch_status')return {running:window.gameAlive,windowReady:window.gameReady};
    if(command==='crash_doctor_diagnose')return {hasError:false};
    return original(command,args);
   };
   window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
  });
  await page.goto('http://127.0.0.1:1420/');await page.getByRole('button',{name:'Jogar',exact:true}).first().waitFor();
  await page.getByRole('button',{name:'Jogar',exact:true}).first().click();
  await page.waitForFunction(()=>window.launchCases.includes('game_launch_status'));
  assert.equal(await page.evaluate(()=>window.launchCases.includes('plugin:window|hide')),false);
  await page.locator('aside nav a[href="/mods"]').click();await page.waitForURL(/\/mods/);
  await page.evaluate(()=>{window.gameReady=true;});
  await page.waitForFunction(()=>window.launchCases.includes('plugin:window|hide'));
  assert.equal(await page.evaluate(()=>window.launchCases.includes('plugin:window|close')),false);
  await page.evaluate(()=>{window.gameAlive=false;window.launcherDemo.emit('game-exit',{versionId:'1.21.1',code:1,success:false,errorMessage:'Java falhou'});});
  await page.waitForFunction(()=>window.launchCases.includes('plugin:window|show'));
  results.push({windowReadiness:true,monitorSurvivesNavigation:true,errorRestoresWindow:true,launcherWindowNotDestroyed:true});
  await page.locator('aside button[title="Início"]').first().click();await page.waitForURL(url=>url.pathname==='/');
  await page.evaluate(()=>{window.earlyExit=true;window.launchCases=[];});
  await page.getByRole('button',{name:'Jogar',exact:true}).first().click();
  await page.getByText(/encerrou durante o início/).first().waitFor();
  assert.equal(await page.evaluate(()=>window.launchCases.includes('plugin:window|hide')),false);
  assert.equal(await page.evaluate(()=>window.launchCases.includes('versions_download')),false);
  results.push({earlyExitDoesNotHide:true,duplicateDownloadRemoved:true});
  assert.deepEqual(errors,[]);console.log(JSON.stringify({results,errors}));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
