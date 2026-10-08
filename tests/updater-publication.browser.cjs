const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 try{
  const page=await browser.newPage({viewport:{width:1440,height:1000}});
  await page.addInitScript(()=>{
   window.updateCalls=[];
   const invoke=async(command,args)=>{
    window.updateCalls.push({command,args});
    if(command==='app_init')return{account:null,profiles:[],activeProfileId:null};
    if(['profiles_list','deep_links_take','changelog_get'].includes(command))return[];
    if(command==='plugin:app|version')return'3.0.2';
    if(command==='app_update_environment')return{mode:'windows'};
    if(command==='app_perform_update')throw Error('This verification must never install the update.');
    if(command==='plugin:store|load'||command==='plugin:event|listen')return 1;
    if(command==='plugin:store|get')return[{language:'pt-BR',languageMode:'manual',autoCheckUpdates:false,releaseChannel:'stable'},true];
    if(command==='get_system_specs')return{totalRamMb:8192,osDistro:'Windows',arch:'x86_64'};
    if(command==='java_scan')return{runtimes:[]};
    return null;
   };
   window.electronAPI={invoke,on:()=>()=>{}};
   window.__TAURI_INTERNALS__={invoke,transformCallback:()=>1,convertFileSrc:path=>path};
  });
  await page.goto('http://127.0.0.1:1420/settings');
  await page.getByRole('button',{name:'Verificar Atualizações',exact:true}).click();
  await page.getByRole('dialog',{name:'Uma nova versão está pronta'}).waitFor({timeout:45000});
  const result = await page.evaluate(async()=>{
   const entry=performance.getEntriesByType('resource').find(e=>e.name.includes('/stores/updater.svelte.ts'));
   if(!entry) throw Error("Updater module missing");
   const{updaterStore}=await import(entry.name);
   if(updaterStore.verificationStatus!=='available') throw Error('Public update was not detected');
   return {current:updaterStore.currentVersion,latest:updaterStore.latestVersion,available:updaterStore.updateAvailable,url:updaterStore.downloadUrl,installed:window.updateCalls.some(c=>c.command==='app_perform_update')};
  });
  assert.equal(result.current,'3.0.2');assert.equal(result.latest,'3.1.0');assert.equal(result.available,true);assert.equal(result.installed,false);assert.match(result.url,/^https:\/\/github\.com\/predabr\/luxmc\/releases\/download\/v3\.1\.0\//);
  fs.writeFileSync('docs/validation/v3.1-public-updater.json',JSON.stringify(result,null,2));console.log(JSON.stringify(result));
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
