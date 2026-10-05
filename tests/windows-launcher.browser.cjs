const { chromium } = require('playwright-core');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({ locale: 'en-US',viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',error=>errors.push(String(error)));
 await page.addInitScript(()=>{
  const profile=(id,name,favorite,lastPlayed)=>({id,name,favorite,lastPlayed,icon:'grass_block',mcVersion:'1.21.1',loader:'fabric',gameDir:'C:\\Users\\PRIVATE\\Minecraft\\'+id,createdAt:new Date().toISOString(),updatedAt:new Date().toISOString(),ramMb:4096});
  const list=[profile('old','Instância antiga',false,'2025-01-01T00:00:00Z'),profile('favorite','Exploração favorita',true,'2026-01-01T00:00:00Z')];
  window.testCalls=[];window.failJava=false;window.testClipboard='';
  Object.defineProperty(navigator,'clipboard',{value:{writeText:async text=>window.testClipboard=text}});
  const invoke=async(command,args)=>{
   window.testCalls.push({command,args});
   if(command==='app_init')return {account:{id:'offline_test',uuid:'offline_test',username:'Steve',accessToken:''},profiles:list,activeProfileId:'old',devMode:true};
   if(command==='profiles_list')return list;
   if(command==='plugin:store|get')return [{language:'pt-BR',languageMode:'manual'},true];
   if(command==='plugin:store|load')return 1;
   if(command==='get_system_specs')return {osDistro:'Windows 11',arch:'x86_64',kernelVersion:'10.0',totalRamMb:8192,launcherVersion:'2.0.2',gpuVendor:'NVIDIA',gpuRenderer:'Test GPU',gpuSupportsZink:false};
   if(command==='java_scan'){if(window.failJava)throw Error('Java check failed');return {runtimes:[{major:21,installed:true,path:'C:\\PRIVATE\\Java\\java.exe',versionString:'21.0.9',isSystem:true}]}}
   if(command==='env_check')return {ok:true,issues:[]};
   if(command==='doctor_instance_readiness')return {ready:true,requiredJava:21,allocatedRamMb:4096,recommendedRamMb:4096,blockers:[],warnings:[],conflicts:{hasConflicts:false,conflicts:[],duplicates:[]}};
   if(command==='deep_links_take'||command==='changelog_get')return [];
   if(command==='minecraft_uuid')return null;
   return null;
  };
  window.electronAPI={invoke,on:()=>()=>{}};window.__TAURI_INTERNALS__={invoke,transformCallback:()=>1,convertFileSrc:()=>'/grass_head.png'};
 });
 await page.route('https://**/*',route=>route.abort());await page.goto('http://127.0.0.1:1420/');assert.equal(await page.locator('.shader-scene').count(),0);
 await page.locator('aside nav a[href="/mods"]').waitFor();await page.locator('button[title="Adicionar amigo"]').click();await page.getByRole('heading',{name:'Adicionar Novo Amigo'}).waitFor();assert.equal(new URL(page.url()).searchParams.get('tab'),'add');await page.locator('aside button[title="Início"]').first().click();await page.waitForURL(url=>url.pathname==='/');await page.keyboard.press('Control+k');await page.getByRole('combobox').fill('diagnostico');await page.getByRole('dialog',{name:'Paleta de comandos'}).getByText(/Diagn.*tico do launcher/).click();const dialog=page.getByRole('dialog',{name:'Diagnóstico do launcher'});
 await dialog.getByText('Windows 11 · x86_64').waitFor();await dialog.getByText('Java 21 · Disponível').waitFor();
 await dialog.getByRole('button',{name:'Fechar diagnóstico'}).focus();await page.keyboard.press('Shift+Tab');assert.equal(await dialog.getByRole('button',{name:'Copiar relatório'}).evaluate(e=>e===document.activeElement),true);
 require('node:fs').mkdirSync('docs/visual/2026-10-03-windows',{recursive:true});await page.screenshot({path:'docs/visual/2026-10-03-windows/diagnostics.png'});
 await dialog.locator('select').selectOption('favorite');await page.waitForFunction(()=>window.testCalls.filter(c=>c.command==='doctor_instance_readiness').at(-1).args.profileId==='favorite');assert.equal(await dialog.locator('select').inputValue(),'favorite');
 await dialog.getByRole('button',{name:'Copiar relatório'}).click();const report=await page.evaluate(()=>window.testClipboard);assert.ok(report.includes('Windows 11'));assert.ok(!report.includes('PRIVATE'));assert.ok(!report.includes('Exploração favorita'));
 await page.evaluate(()=>window.failJava=true);await dialog.getByRole('button',{name:'Verificar novamente'}).click();await dialog.getByRole('alert').filter({hasText:'Java check failed'}).waitFor();assert.ok(await dialog.getByText('Windows 11 · x86_64').count());
 await page.keyboard.press('Escape');assert.equal(await dialog.count(),0);
 await page.keyboard.press('Control+k');await page.getByRole('combobox').fill('exploracao');const palette=page.getByRole('dialog',{name:'Paleta de comandos'});await palette.getByText('Exploração favorita',{exact:true}).waitFor();assert.equal(await palette.getByText('Instância antiga',{exact:true}).count(),0);await page.getByRole('combobox').fill('diagnostico');await palette.getByText('Diagnóstico do launcher',{exact:true}).click();await dialog.getByText('Windows 11 · x86_64').waitFor();await page.keyboard.press('Escape');
 await page.goto('http://127.0.0.1:1420/settings');await page.getByRole('tab',{name:'Java',exact:true}).click();assert.equal(await page.getByRole('switch',{name:'Toggle Wayland Mode',exact:true}).count(),0);await page.getByRole('button',{name:'Recomendar RAM',exact:true}).click();await page.getByText('RAM recomendada aplicada: 4 GB.',{exact:false}).waitFor();
 assert.deepEqual(errors,[]);console.log('Windows frontend: diagnostics, instance selection, private report, partial failure, command search, RAM recommendation passed');await browser.close();
})().catch(error=>{console.error(error);process.exit(1)});
