const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{
  if (!localStorage.getItem('fixtureInitialized')) { localStorage.setItem('fixtureInitialized','1'); localStorage.setItem('luxmc_custom_wallpaper','luxmc-wallpaper:cherry'); localStorage.setItem('luxmc_custom_wallpaper_type','video'); localStorage.setItem('luxmc_background','custom'); localStorage.setItem('luxmc_wallpaper_library',JSON.stringify([{url:'luxmc-wallpaper:cherry',type:'video',name:'Cerejeiras'}])); }
  window.calls=[];window.room=null;window.failStorage=false;window.clipboardText='';
  Object.defineProperty(navigator,'clipboard',{value:{writeText:async value=>window.clipboardText=value}});
  const host={id:'host-key',username:'Steve',uuid:'steve',avatarUrl:null,joinedAt:1,isHost:true};
  const invoke=async(command,args)=>{
   window.calls.push({command,args});
   if(command==='app_init')return {account:{id:'offline_test',uuid:'steve',username:'Steve',minecraftToken:''},profiles:[{id:'fabric-test',name:'Modpack de teste',mcVersion:'1.21.1',loader:'fabric',gameDir:'C:\\PRIVATE\\Minecraft',createdAt:'2026-01-01',updatedAt:'2026-01-01',ramMb:4096}],activeProfileId:'fabric-test',devMode:true};
   if(command==='plugin:store|load'||command==='plugin:event|listen')return 1;
   if(command==='plugin:store|get')return [{language:'pt-BR',animations:true,liveWallpaper:false,soundscapesEnabled:false,pauseWallpaperOnBlur:false},true];
   if(command==='profiles_list')return [{id:'fabric-test',name:'Modpack de teste',mcVersion:'1.21.1',loader:'fabric',gameDir:'C:\\PRIVATE\\Minecraft',createdAt:'2026-01-01',updatedAt:'2026-01-01',ramMb:4096}];
   if(['deep_links_take','profiles_list','instances_list','screenshots_list','changelog_get'].includes(command))return [];
   if(command==='get_system_specs')return {totalRamMb:8192,osDistro:'Windows 11',arch:'x86_64',launcherVersion:'2.0.2'};
   if(command==='java_scan')return {runtimes:[]};
   if(command==='p2p_scan_lan_worlds')return [{host:'127.0.0.1',port:54321,motd:'Mundo do Steve'}];
   if(command==='p2p_get_local_info')return {ip:'127.0.0.1',port:25575};
   if(command==='tunnel_status'){if(window.hangStatus)return new Promise(()=>{});return window.room?structuredClone(window.room):null;}
   if(command==='join_world'){window.room={mode:'client',invitation:null,localAddress:'127.0.0.1:55555',expiresAt:9999999999,pingMs:12,transport:'Relay criptografado',roomCode:'LUX-4821',members:[host,{id:'guest',username:'Alex',uuid:'alex',avatarUrl:null,joinedAt:2,isHost:false}],maxPlayers:10,roomLocked:false};return structuredClone(window.room)}
   if(command==='tunnel_save_server')return '127.0.0.1:55555';
   if(command==='host_world'){window.room={mode:'host',invitation:'LUX-4821|luxmc-world:test',localAddress:null,expiresAt:9999999999,pingMs:null,transport:'QUIC',roomCode:'LUX-4821',members:[host],maxPlayers:10,roomLocked:false};return structuredClone(window.room)}
   if(command==='tunnel_kick_member'){window.room.members=window.room.members.filter(m=>m.id!==args.memberId);return null}
   if(command==='tunnel_set_locked'){window.room.roomLocked=args.locked;return null}
   if(command==='tunnel_refresh_invitation'){window.room.invitation='LUX-4821|luxmc-world:new';return structuredClone(window.room)}
   if(command==='stop_session'){window.room=null;return null}
   if(command==='storage_full_report'){if(window.failStorage)throw Error('Falha simulada de leitura');return {totalBytes:4194304,categories:[{category:'application',bytes:2097152,path:'launcher.exe'},{category:'java',bytes:1048576,path:'java'},{category:'launcher_data',bytes:1048576,path:'luxmc.db'}],instances:[],logsBytes:0,cacheBytes:0,warnings:[]}}
   if(command==='app_data_directory')return 'C:\\Luxmc\\data';
   if(command==='mods_search')return ['Zombie Invade 100 Days','SkyFactory 4'].map((title,i)=>({sourceId:String(i),source:'modrinth',slug:'unique-'+i,title,description:'Modpack sem galeria',downloads:1000,iconUrl:'/grass_head.png',bannerUrl:null,author:'Author',categories:['forge'],versions:['1.21.1']}));
   if(command==='curseforge_status')return true;
   if(command==='versions_list')return {versions:[],latestRelease:'1.21.1',latestSnapshot:''};
   if(command==='minecraft_uuid')return null;
   if(command==='instance_export_share_code')return 'LUXMC1-1234567890123456';
   if(command==='importer_detect_launchers')return [];
   return null;
  };
  window.electronAPI={invoke,on:()=>()=>{}};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},invoke,transformCallback:()=>1,convertFileSrc:path=>path};
 });
 await page.route('https://**/*',route=>route.abort());
 await page.route('**/test-wallpaper.mp4',route=>route.fulfill({contentType:'video/mp4',body:fs.readFileSync('website/assets/cinema/cherry-loop.mp4')}));
 await page.route('https://avatars.test/**',route=>route.fulfill({contentType:'image/png',body:fs.readFileSync('static/grass_head.png')}));
 await page.goto('http://127.0.0.1:1420/skins');
 await page.locator('.skin-preview-stage').waitFor();
 assert.equal(await page.locator('.skin-preview-stage').getByText('Clássico (4 px)',{exact:true}).count(),0);
 assert.equal(await page.locator('.skin-preview-stage span[title]').count(),0);
 assert.equal(await page.locator('.skin-preview-stage button').count(),1);
 assert.equal(await page.evaluate(()=>localStorage.getItem('luxmc_custom_wallpaper')),null);
 assert.equal(await page.evaluate(()=>JSON.parse(localStorage.getItem('luxmc_wallpaper_library')).length),0);
 await page.evaluate(()=>{localStorage.setItem('luxmc_custom_wallpaper','http://127.0.0.1:1420/test-wallpaper.mp4');localStorage.setItem('luxmc_custom_wallpaper_type','video');localStorage.setItem('luxmc_background','custom');});
 await page.goto('http://127.0.0.1:1420/settings');
 await page.getByRole('button',{name:'Aparência',exact:false}).click();
 assert.equal(await page.getByRole('button',{name:'Usar wallpaper animado de Minecraft com shaders',exact:true}).count(),0);
 await page.waitForFunction(()=>document.documentElement.classList.contains('has-custom-wallpaper'));
 const video=page.locator('video').first();
 await video.waitFor();
 await page.waitForFunction(()=>{const v=document.querySelector('video');return v&&v.readyState>=2&&v.currentTime>0.1});
 assert.equal(await video.evaluate(v=>v.muted&&v.loop),true);
 const opacity = page.getByLabel('Opacidade da interface com wallpaper',{exact:false});
 await opacity.fill('27');
 await opacity.dispatchEvent('input');
 await page.waitForFunction(()=>Math.abs(Number(getComputedStyle(document.documentElement).getPropertyValue('--interface-opacity'))-.27)<.001);
 await page.locator('aside button[title="Início"]').click();
 const triggers=page.locator('.luxmc-control .glass-select-trigger');
 await triggers.first().waitFor();
 assert.equal(await triggers.first().evaluate(el=>getComputedStyle(el).backgroundColor),'rgba(0, 0, 0, 0)');
 fs.mkdirSync('docs/visual/2026-10-04-editorial',{recursive:true});
 await page.screenshot({path:'docs/visual/2026-10-04-editorial/launcher-wallpaper.png'});
 await page.waitForFunction(()=>{const v=document.querySelector('video');return v&&!v.paused&&v.currentTime>0.1});
 await page.evaluate(()=>document.dispatchEvent(new Event('visibilitychange')));
 await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
 await page.waitForFunction(()=>document.querySelector('video').paused);
 await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:false});document.dispatchEvent(new Event('visibilitychange'));});
 await page.waitForFunction(()=>!document.querySelector('video').paused);
 await page.goto('http://127.0.0.1:1420/instances');
 await page.getByRole('button',{name:'Migrar Launchers',exact:true}).click();
 const importer=page.getByRole('dialog');
 await importer.getByRole('heading',{name:'Migrar launchers e compartilhar instâncias'}).waitFor();
 await page.waitForFunction(()=>{const dialog=document.querySelector('[role=dialog]');return dialog&&getComputedStyle(dialog).opacity==='1'&&getComputedStyle(dialog.firstElementChild).opacity==='1';});
 assert.ok((await importer.locator(':scope > div').first().boundingBox()).width >= 850);
 assert.equal(await importer.locator(':scope > div').first().evaluate(el=>getComputedStyle(el).backgroundColor),'rgba(17, 18, 22, 0.94)');
 await importer.getByLabel('Instância para compartilhar').selectOption('fabric-test');
 await importer.getByRole('button',{name:'Gerar código',exact:true}).click();
 await importer.getByRole('status').filter({hasText:'LUXMC1-1234567890123456'}).waitFor();
 assert.equal(await page.evaluate(()=>window.clipboardText),'LUXMC1-1234567890123456');
 await page.screenshot({path:'docs/visual/2026-10-04-editorial/migration-codes.png'});
 assert.deepEqual(errors,[]);
 console.log('Removed preset migration, imported video playback, adjustable opacity, transparent filters, skin labels, pause/resume passed');
 await browser.close();
})().catch(error=>{console.error(error);process.exit(1)});
