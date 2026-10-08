const {chromium}=require('playwright-core');const assert=require('node:assert/strict');const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
(async()=>{const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});try{
 const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});const errors=[];page.on('pageerror',e=>errors.push(String(e)));await setupLauncherDemo(page,{room:true});
 await page.addInitScript(()=>{
  const state=window.launcherDemo.state;state.settings.language='pt-BR';state.settings.languageMode='manual';
  const compatibility={mcVersion:'1.21.1',loader:'fabric',modFingerprint:'a'.repeat(64),modCount:3};
  state.room.worlds=[{ownerId:'demo-host',ownerUsername:'AlexBuilder',motd:'Mundo compartilhado',localAddress:'127.0.0.1:51234',compatibility},{ownerId:'demo-guest',ownerUsername:'SteveCraft',motd:'Launcher antigo',localAddress:'127.0.0.1:51235',compatibility:null}];
  const original=window.electronAPI.invoke;const invoke=async(command,args)=>{if(command==='instance_compatibility')return {...compatibility};return original(command,args);};window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
 });
 await page.goto('http://127.0.0.1:1420/hosting');await page.getByRole('button',{name:'Verificar antes de entrar',exact:true}).click();await page.getByText('Compatível',{exact:true}).first().waitFor();assert.equal(await page.getByText('Compatível',{exact:true}).count(),3);assert.equal(await page.getByText('Não confirmado',{exact:true}).count(),3);await page.getByText(/Contas Luxmc e locais não possuem/).waitFor();
 await page.evaluate(()=>window.launcherDemo.state.room.worlds[0].compatibility.mcVersion='1.20.1');await page.getByText('Diferente: use uma instância compatível',{exact:true}).waitFor();assert.equal(await page.getByText('Diferente: use uma instância compatível',{exact:true}).count(),1);
 fs.mkdirSync('docs/validation/workshop',{recursive:true});await page.locator('main').evaluate(main=>main.scrollTop=main.scrollHeight);await page.screenshot({path:'docs/validation/workshop/preflight.png'});assert.deepEqual(errors,[]);console.log('PASS: equal versions/loaders/mod hashes, unknown remote metadata, mismatch and account warning');
}finally{await browser.close();}})().catch(error=>{console.error(error);process.exit(1)});
