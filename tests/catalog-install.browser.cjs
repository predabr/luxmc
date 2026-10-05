const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{
  window.calls=[];window.room=null;window.failStorage=false;window.clipboardText='';
  Object.defineProperty(navigator,'clipboard',{value:{writeText:async value=>window.clipboardText=value}});
  const host={id:'host-key',username:'Steve',uuid:'steve',avatarUrl:null,joinedAt:1,isHost:true};
  const invoke=async(command,args)=>{
   window.calls.push({command,args});
   if(command==='app_init')return {account:{id:'offline_test',uuid:'steve',username:'Steve',minecraftToken:''},profiles:[{id:'fabric-test',name:'Modpack de teste',mcVersion:'1.21.1',loader:'fabric',gameDir:'C:\\PRIVATE\\Minecraft',createdAt:'2026-01-01',updatedAt:'2026-01-01',ramMb:4096}],activeProfileId:'fabric-test',devMode:true};
   if(command==='plugin:store|load'||command==='plugin:event|listen')return 1;
   if(command==='plugin:store|get')return [{language:'pt-BR',animations:false,liveWallpaper:false,soundscapesEnabled:false},true];
   if(command==='profiles_list')return [{id:'fabric-test',name:'Modpack de teste',mcVersion:'1.21.1',loader:'fabric',gameDir:'C:\\PRIVATE\\Minecraft',createdAt:'2026-01-01',updatedAt:'2026-01-01',ramMb:4096}];
   if(['deep_links_take','profiles_list','instances_list','screenshots_list','changelog_get'].includes(command))return [];
   if(command==='get_system_specs')return {totalRamMb:8192,osDistro:'Windows 11',arch:'x86_64',launcherVersion:'2.0.2'};
   if(command==='java_scan')return {runtimes:[]};
   if(command==='p2p_scan_lan_worlds')return [{host:'127.0.0.1',port:54321,motd:'Mundo do Steve'}];
   if(command==='p2p_get_local_info')return {ip:'127.0.0.1',port:25575};
   if(command==='tunnel_status'){if(window.hangStatus)return new Promise(()=>{});return window.room?structuredClone(window.room):null;}
   if(command==='join_world'){window.room={mode:'client',invitation:null,localAddress:'127.0.0.1:55555',expiresAt:9999999999,pingMs:12,transport:'Relay criptografado',roomCode:'LUXMC1-1234567890123456',members:[host,{id:'guest',username:'Alex',uuid:'alex',avatarUrl:null,joinedAt:2,isHost:false}],maxPlayers:10,roomLocked:false};return structuredClone(window.room)}
   if(command==='tunnel_save_server')return '127.0.0.1:55555';
   if(command==='host_world'){window.room={mode:'host',invitation:'LUXMC1-1234567890123456',localAddress:null,expiresAt:9999999999,pingMs:null,transport:'QUIC',roomCode:'LUXMC1-1234567890123456',members:[host],maxPlayers:10,roomLocked:false};return structuredClone(window.room)}
   if(command==='tunnel_kick_member'){window.room.members=window.room.members.filter(m=>m.id!==args.memberId);return null}
   if(command==='tunnel_set_locked'){window.room.roomLocked=args.locked;return null}
   if(command==='tunnel_refresh_invitation'){window.room.invitation='LUX-4821|luxmc-world:new';return structuredClone(window.room)}
   if(command==='stop_session'){window.room=null;return null}
   if(command==='storage_full_report'){if(window.failStorage)throw Error('Falha simulada de leitura');return {totalBytes:4194304,categories:[{category:'application',bytes:2097152,path:'launcher.exe'},{category:'java',bytes:1048576,path:'java'},{category:'launcher_data',bytes:1048576,path:'luxmc.db'}],instances:[],logsBytes:0,cacheBytes:0,warnings:[]}}
   if(command==='app_data_directory')return 'C:\\Luxmc\\data';
   if(command==='mods_search')return [{sourceId:'content',source:args.contentType==='world'?'curseforge':'modrinth',slug:'content',title:args.contentType==='world'?'Mundo teste':'Conteúdo teste',description:'Descrição',downloads:100,iconUrl:'/grass_head.png',bannerUrl:null,author:'Author',categories:['fabric'],versions:['1.20.1']}];
   if(command==='mods_versions')return window.noVersions?[]:[{id:'wrong-loader',name:'Forge',versionNumber:'1',loaders:['forge'],files:[]},{id:'fabric-version',name:'Fabric',versionNumber:'1',loaders:['fabric'],files:[]}];
   if(command==='mods_install')return null;
   if(command==='instance_worlds_list')return [{name:'Meu mundo',folderName:'Meu mundo',iconBase64:null,lastPlayed:null,gameMode:'survival',sizeBytes:100}];
   if(command==='curseforge_status')return true;
   if(command==='versions_list')return {versions:[],latestRelease:'1.21.1',latestSnapshot:''};
   if(command==='minecraft_uuid')return null;
   return null;
  };
  window.electronAPI={invoke,on:()=>()=>{}};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},invoke,transformCallback:()=>1,convertFileSrc:path=>path};
 });
 await page.route('https://**/*',route=>route.abort());
 await page.route('https://avatars.test/**',route=>route.fulfill({contentType:'image/png',body:fs.readFileSync('static/grass_head.png')}));
 for (const [type,label] of [['mod','Mod'],['resourcepack','Resource Pack'],['shader','Shader'],['datapack','Data Pack']]) {
   await page.goto(`http://127.0.0.1:1420/mods?type=${type}`);
   await page.getByRole('button',{name:/^Instalar:/}).first().click();
   const dialog=page.getByRole('dialog',{name:'Escolha a instância'});
   await dialog.getByRole('button',{name:/Modpack de teste/}).waitFor();
   assert.equal(await dialog.getByRole('button',{name:/Modpack de teste/}).isEnabled(),true);
   if(type==='datapack') await dialog.getByLabel('Mundo que receberá o datapack').selectOption('Meu mundo');
   await dialog.getByRole('button',{name:'Confirmar e Instalar',exact:true}).click();
   await page.waitForFunction(()=>window.calls.some(c=>c.command==='mods_install'));
   const request=await page.evaluate(()=>window.calls.find(c=>c.command==='mods_install').args.request);
   assert.equal(request.profileId,'fabric-test'); assert.equal(request.contentType,type);
   if(type==='mod')assert.equal(request.versionId,'fabric-version');
   if(type==='datapack')assert.equal(request.worldName,'Meu mundo');
 }
 await page.goto('http://127.0.0.1:1420/mods?type=world');
 await page.getByText('Mundo teste',{exact:true}).first().waitFor();
 assert.equal(await page.getByText('Fabric API',{exact:true}).count(),0);
 await page.waitForFunction(()=>window.calls.some(c=>c.command==='mods_search'&&c.args.contentType==='world'&&c.args.source==='curseforge'));
 fs.mkdirSync('docs/visual/2026-10-04-catalog',{recursive:true});
 await page.screenshot({path:'docs/visual/2026-10-04-catalog/worlds.png'});
 assert.deepEqual(errors,[]);
 console.log('All four content types list modpack instances; compatible loader file chosen; datapack world selected; World requests CurseForge; no frontend errors.');
 await browser.close();
})().catch(error=>{console.error(error);process.exit(1)});
