const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 fs.mkdirSync('docs/validation/social-download-polish',{recursive:true});
 try {
 for(const theme of ['dark','light']) {
 const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await setupLauncherDemo(page);
 await page.addInitScript(theme=>{
  const state=window.launcherDemo.state;state.settings.theme=theme==='light'?'default-light':'default-dark';state.settings.language='pt-BR';state.settings.languageMode='manual';
  const original=window.electronAPI.invoke;
  window.contentCalls=[];window.profileSaved=null;window.publicFixture={id:'00000000-0000-4000-8000-000000000004',username:'PixelExplorer',avatarUrl:'/grass_head.png',description:'Eu construo mundos.',banner:'',portrait:'',packs:['Homestead']};
  const invoke=async(command,args)=>{
   if(command==='social_request'&&args.request.action==='profile_get') return {profile: args.request.targetId?window.publicFixture:{...window.publicFixture,id:'00000000-0000-4000-8000-000000000001',username:'LuxPlayer',description:'',packs:[]}};
   if(command==='social_request'&&args.request.action==='profile_save'){window.profileSaved=args.request;return {ok:true};}
   if(command==='instance_file_tree')return Array.from({length:1000},(_,i)=>({name:`Mod${i}.jar`,path:`/mods/Mod${i}.jar`,isDir:false,size:1234,icon:null,iconKey:`content:v2:${i}`,iconResolved:false}));
   if(command==='instance_content_icons'){window.contentCalls.push(args);return args.fileNames.map(name=>({name,icon:'/grass_head.png',iconKey:`content:v2:${name.match(/\d+/)[0]}`,resolved:true}));}
   if(command==='mods_versions')return [{id:'pack',files:[{url:'https://cdn.modrinth.com/test.mrpack',filename:'test.mrpack'}]}];
   if(command==='mods_download_to_temp')return new Promise(resolve=>{window.finishDownload=()=>resolve('/test.mrpack');});
   if(command==='instance_import_mrpack')return {...state.profiles[1],id:'new-test',name:'Persistent Pack'};
   return original(command,args);
  };
  window.electronAPI.invoke=invoke;window.__TAURI_INTERNALS__.invoke=invoke;
 },theme);
 await page.goto('http://127.0.0.1:1420/friends');
 await page.getByRole('button',{name:'PixelExplorer',exact:true}).first().click();
 const dialog=page.getByRole('dialog',{name:'PixelExplorer'});await dialog.getByText('Eu construo mundos.',{exact:true}).waitFor();
 assert.equal(await dialog.getByText('Homestead',{exact:true}).count(),1);
 await page.screenshot({path:`docs/validation/social-download-polish/profile-${theme}.png`});
 await dialog.getByRole('button',{name:'Fechar',exact:true}).click();
 await page.getByRole('button',{name:'Editar meu perfil',exact:true}).first().click();
 await page.getByRole('textbox',{name:'Sobre',exact:true}).fill('Meu perfil editado.');
 await page.getByRole('button',{name:'Salvar',exact:true}).click();
 await page.waitForFunction(()=>window.profileSaved?.description==='Meu perfil editado.');
 await page.getByRole('dialog').getByRole('button',{name:'Fechar',exact:true}).click();
 await page.screenshot({path:`docs/validation/social-download-polish/friends-${theme}.png`});
 await page.goto('http://127.0.0.1:1420/instances/demo-vp');
 await page.waitForFunction(()=>window.contentCalls.length>0);
 assert.ok(await page.evaluate(()=>window.contentCalls.flatMap(call=>call.fileNames).length<100),'Only visible content is decoded');
 assert.ok(await page.evaluate(()=>window.contentCalls.every(call=>call.fileNames.length<=32)));
 await page.screenshot({path:`docs/validation/social-download-polish/content-${theme}.png`});
 await page.evaluate(async()=>{
  const {modpackInstallation}=await import(performance.getEntriesByType('resource').find(entry=>entry.name.includes('/stores/modpackInstallation.svelte.ts')).name);
  window.modpackTask=modpackInstallation.start({title:'Persistent Pack',sourceId:'pack',source:'modrinth',versions:['1.21.1'],categories:['fabric'],iconUrl:'/modpack_fo_icon.png'},'Persistent Pack',4096,'Qualquer Versão');
 });
 await page.getByRole('complementary',{name:'Fila de instalações'}).waitFor();
 await page.getByRole('link',{name:'Personalização',exact:true}).first().click();
 await page.getByRole('complementary',{name:'Fila de instalações'}).waitFor();
 await page.screenshot({path:`docs/validation/social-download-polish/download-${theme}.png`});
 await page.evaluate(()=>window.finishDownload());
 await page.getByRole('complementary',{name:'Fila de instalações'}).getByRole('button',{name:'Abrir instância',exact:true}).waitFor();
 await page.getByRole('complementary',{name:'Fila de instalações'}).getByRole('button',{name:'Fechar',exact:true}).click();
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({theme,offlineProfile:true,saveProfile:true,visibleOnlyIcons:true,persistentInstall:true}));
 await page.close();
 }
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
