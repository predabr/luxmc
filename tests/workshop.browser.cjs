const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 fs.mkdirSync('docs/validation/workshop',{recursive:true});
 try {
 for(const theme of ['dark','light']) for(const width of [1440,960]) {
  const page=await browser.newPage({viewport:{width,height:1000},reducedMotion:'reduce'});
  const errors=[];page.on('pageerror',error=>errors.push(String(error)));await setupLauncherDemo(page);
  await page.addInitScript(({theme})=>{
   const state=window.launcherDemo.state;state.settings.theme=theme==='light'?'default-light':'default-dark';state.settings.language='pt-BR';state.settings.languageMode='manual';
   const item={sourceId:'test',source:'modrinth',title:'Queued Pack',slug:'test',description:'Test',downloads:1,author:null,versions:['1.21.1'],categories:['fabric'],iconUrl:null,bannerUrl:null};
   const job={id:'persisted',item,name:'Queued Pack',ramMb:4096,selectedVersion:'1.21.1',status:'running'};
   const original=window.electronAPI.invoke;let rejectDownload;
   const readiness={ready:true,repairable:false,requiredJava:21,blockers:[],warnings:[],conflicts:{hasConflicts:false,conflicts:[],duplicates:[]}};
   window.workshopCalls=[];
   const invoke=async(command,args)=>{
    window.workshopCalls.push({command,args});
    if(command==='settings_get')return {...state.settings,installationQueue:JSON.parse(localStorage.getItem('queue-fixture')||JSON.stringify([job,{...job,id:'secondary',name:'Second Pack',status:'paused'}]))};
    if(command==='settings_set'){Object.assign(state.settings,args.value);if(args.value.installationQueue)localStorage.setItem('queue-fixture',JSON.stringify(args.value.installationQueue));return null;}
    if(command==='performance_history')return [{profileId:state.profiles[0].id,timestamp:'2026-10-07T12:00:00Z',metrics:{kind:'preparation',seconds:4.2,pid:42}},{profileId:state.profiles[0].id,timestamp:'2026-10-07T12:20:00Z',metrics:{kind:'session',peakRamMb:2048,durationSeconds:1200,pid:42}}];
    if(command==='modpack_validate')return readiness;
    if(command==='support_report')return {launcherVersion:'3.1.0',platform:'windows',readiness,diagnosis:{title:'Nenhuma falha detectada',solution:'Nenhuma ação necessária',logSnippet:'[TOKEN] [EMAIL] [USUARIO]'}};
    if(command==='instance_list_world_backups')return [{fileName:'world_20261007_120000.zip',worldName:'Meu mundo',sizeBytes:1024,createdAt:'07/10/2026 12:00'}];
    if(command==='world_restore')return null;
    if(command==='mods_versions')return [{id:'pack',files:[{url:'https://cdn.modrinth.com/test.mrpack',filename:'test.mrpack',size:10,sha1:''}]}];
    if(command==='pack_reference_version')return {id:args.reference.versionId,files:[{url:'https://cdn.modrinth.com/test.mrpack',filename:'test.mrpack',size:10,sha1:''}]};
    if(command==='mods_download_to_temp')return new Promise((resolve,reject)=>{rejectDownload=reject;window.finishQueueDownload=()=>resolve('/cached/test.mrpack');});
    if(command==='instance_cancel_import'){rejectDownload?.(new Error('Importação cancelada'));return null;}
    if(command==='verify_pack_download')return null;
    if(command==='instance_import_mrpack')return {...state.profiles[1],id:'queue-installed',name:'Queued Pack'};
    if(command==='plugin:dialog|open')return '/test.luxtheme';
   if(command==='plugin:dialog|save')return args.options.defaultPath.endsWith('.json')?'/report.json':'/theme.luxtheme';
   if(command==='theme_export'||command==='support_report_export')return null;
    if(command==='theme_import')return {theme:'default-light',accentTheme:'blue',customBackground:'obsidian',customWallpaperUrl:''};
    return original(command,args);
   };window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
  },{theme});
  await page.goto('http://127.0.0.1:1420/workshop');await page.getByRole('heading',{name:'Central de manutenção',exact:true}).waitFor();
  await page.getByText('4.2 s',{exact:true}).first().waitFor();await page.getByText('2.0 GB',{exact:true}).waitFor();
  const queue=page.getByRole('complementary',{name:'Instalação em andamento'});
  await page.getByText('Pausado · pronto para retomar',{exact:true}).first().waitFor();
  await page.getByRole('button',{name:'Priorizar',exact:true}).last().click();await page.waitForFunction(()=>JSON.parse(localStorage.getItem('queue-fixture'))[0].id==='secondary');
  await page.getByRole('button',{name:'Remover',exact:true}).first().click();await page.waitForFunction(()=>JSON.parse(localStorage.getItem('queue-fixture')).length===1);
  await page.getByRole('button',{name:'Retomar',exact:true}).click();await page.waitForFunction(()=>typeof window.finishQueueDownload==='function');
  await page.getByRole('button',{name:'Pausar e preservar arquivos',exact:true}).click();await page.getByRole('button',{name:'Retomar',exact:true}).waitFor();
  await page.reload();await page.getByRole('button',{name:'Retomar',exact:true}).waitFor();
  assert.equal(await page.evaluate(()=>window.workshopCalls.filter(call=>call.command==='mods_download_to_temp').length),0);
  await page.getByRole('button',{name:'Retomar',exact:true}).click();await page.waitForFunction(()=>typeof window.finishQueueDownload==='function');await page.evaluate(()=>window.finishQueueDownload());
  await page.waitForFunction(()=>window.workshopCalls.some(call=>call.command==='instance_import_mrpack'));
  await page.getByRole('complementary',{name:'Fila de instalações'}).getByRole('button',{name:'Fechar',exact:true}).click();
  await page.getByRole('button',{name:'Validar instância',exact:true}).click();await page.getByText('Verificação concluída: nenhuma falha bloqueante identificada',{exact:true}).waitFor();
  await page.getByRole('button',{name:'Preparar diagnóstico',exact:true}).click();await page.getByRole('button',{name:'Exportar diagnóstico',exact:true}).waitFor();await page.getByRole('button',{name:'Exportar diagnóstico',exact:true}).click();await page.waitForFunction(()=>window.workshopCalls.some(call=>call.command==='support_report_export'));
  await page.getByRole('button',{name:'Ver backups',exact:true}).click();await page.getByRole('button',{name:'Restaurar',exact:true}).click();await page.getByRole('alert').filter({hasText:'Restaurar Meu mundo?'}).waitFor();
  await page.getByRole('button',{name:'Cancelar',exact:true}).last().click();
  await page.locator('main').evaluate(main=>main.scrollTop=0);
  await page.screenshot({path:`docs/validation/workshop/center-${theme}-${width}.png`,fullPage:true});
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));assert.deepEqual(errors,[]);
  await page.getByRole('button',{name:'Exportar tema',exact:true}).click();await page.waitForFunction(()=>window.workshopCalls.some(call=>call.command==='theme_export'));
  await page.getByRole('button',{name:'Importar tema',exact:true}).click();await page.getByText('Tema aplicado.',{exact:true}).waitFor();assert.ok(await page.locator('html').evaluate(root=>root.classList.contains('light')));
  await page.close();
 }
 console.log('PASS: history, persisted queue, pause/restart/resume, validation, report preview, restore confirmation, dark/light layouts');
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exit(1)});
