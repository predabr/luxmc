const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({ locale: 'en-US',viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{
  window.calls=[];window.room=null;window.failStorage=false;window.clipboardText='';
  Object.defineProperty(navigator,'clipboard',{value:{writeText:async value=>window.clipboardText=value}});
  const host={id:'host-key',username:'Steve',uuid:'steve',avatarUrl:null,joinedAt:1,isHost:true};
  const invoke=async(command,args)=>{
   window.calls.push({command,args});
   if(command==='app_init')return {account:{id:'offline_test',uuid:'steve',username:'Steve',minecraftToken:''},profiles:[],activeProfileId:null,devMode:true};
   if(command==='plugin:store|load'||command==='plugin:event|listen')return 1;
   if(command==='plugin:store|get')return [{language:'pt-BR',languageMode:'manual',animations:false,liveWallpaper:false,soundscapesEnabled:false},true];
   if(['deep_links_take','profiles_list','instances_list','screenshots_list','changelog_get'].includes(command))return [];
   if(command==='get_system_specs')return {totalRamMb:8192,osDistro:'Windows 11',arch:'x86_64',launcherVersion:'2.0.2'};
   if(command==='java_scan')return {runtimes:[]};
   if(command==='p2p_scan_lan_worlds')return [{host:'127.0.0.1',port:54321,motd:'Mundo do Steve'}];
   if(command==='p2p_get_local_info')return {ip:'127.0.0.1',port:25575};
   if(command==='tunnel_status')return window.room?structuredClone(window.room):null;
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
   return null;
  };
  window.electronAPI={invoke,on:()=>()=>{}};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},invoke,transformCallback:()=>1,convertFileSrc:path=>path};
 });
 await page.route('https://**/*',route=>route.abort());
 await page.route('https://avatars.test/**',route=>route.fulfill({contentType:'image/png',body:fs.readFileSync('static/grass_head.png')}));
 await page.goto('http://127.0.0.1:1420/hosting');
 assert.equal(await page.locator('aside button[aria-label="Minha hospedagem"]').count(),0);
 assert.equal(await page.locator('aside button[aria-label="Diagnóstico do launcher"]').count(),0);
 await page.getByRole('button',{name:'Hospedar meu mundo',exact:true}).click();
 await page.getByRole('heading',{name:'Sua sala, sua turma.',exact:true}).waitFor();
 assert.equal(await page.locator('[data-room-member]').count(),1);
 const hostCall=await page.evaluate(()=>window.calls.find(c=>c.command==='host_world'));
 assert.equal(hostCall.args.port,undefined);assert.equal(hostCall.args.identity.username,'Steve');
 await page.locator('aside a[href="/skins"]').click();await page.waitForURL('**/skins');await page.locator('aside a[href="/friends"]').click();await page.waitForURL('**/hosting');await page.getByRole('heading',{name:'Sua sala, sua turma.',exact:true}).waitFor();
 await page.evaluate(()=>window.room.members.push({id:'guest-key',username:'Alex',uuid:'alex',avatarUrl:'https://avatars.test/alex.png',joinedAt:2,isHost:false}));
 await page.getByText('Alex',{exact:true}).waitFor();
 assert.equal(await page.locator('[data-room-member]').count(),2);
 await page.getByRole('button',{name:'Copiar convite',exact:true}).click();assert.equal(await page.evaluate(()=>window.clipboardText),'LUX-4821|luxmc-world:test');
 await page.getByRole('button',{name:'Fechar novas entradas',exact:true}).click();await page.getByRole('button',{name:'Liberar novas entradas',exact:true}).waitFor();
 await page.getByRole('button',{name:'Novo convite',exact:true}).click();assert.equal(await page.locator('[data-room-member]').count(),2);
 fs.mkdirSync('docs/visual/2026-10-03-hosting',{recursive:true});await page.screenshot({path:'docs/visual/2026-10-03-hosting/room.png'});
 await page.getByRole('button',{name:'Expulsar Alex',exact:true}).click();const dialog=page.getByRole('dialog',{name:'Remover participante'});await dialog.getByRole('button',{name:'Expulsar participante',exact:true}).click();await page.waitForFunction(()=>document.querySelectorAll('[data-room-member]').length===1);
 assert.equal(await page.evaluate(()=>window.calls.find(c=>c.command==='tunnel_kick_member').args.memberId),'guest-key');
 await page.getByRole('button',{name:'Encerrar hospedagem',exact:true}).click();await page.getByRole('dialog',{name:'Encerrar hospedagem'}).getByRole('button',{name:'Encerrar sala',exact:true}).click();await page.waitForURL('**/friends');assert.equal(await page.evaluate(()=>window.room),null);
 await page.goto('http://127.0.0.1:1420/settings');await page.getByRole('tab',{name:'Armazenamento',exact:true}).click();await page.getByText('Aplicativo Luxmc',{exact:true}).waitFor();await page.getByText('Java do launcher',{exact:true}).waitFor();assert.ok(await page.getByText('4 MB',{exact:true}).count());assert.equal(await page.locator('.shader-scene').count(),0);
 await page.evaluate(()=>window.failStorage=true);await page.getByRole('button',{name:'Atualizar',exact:true}).click();await page.getByRole('alert').filter({hasText:'Falha simulada de leitura'}).waitFor();
 await page.goto('http://127.0.0.1:1420/mods');await page.getByText('Zombie Invade 100 Days',{exact:true}).waitFor();assert.equal(await page.locator('.shader-scene').count(),0);
 for(const title of ['Zombie Invade 100 Days','SkyFactory 4']){const card=page.locator('.catalog-card').filter({has:page.getByText(title,{exact:true})});const image=card.locator('img').first();const imageBox=await image.boundingBox();const cardBox=await card.boundingBox();assert.ok(imageBox.width>250&&imageBox.height>=180&&Math.abs(imageBox.width-cardBox.width)<=3);assert.equal(await image.evaluate(i=>getComputedStyle(i).objectFit),'cover')}
 await page.screenshot({path:'docs/visual/2026-10-03-hosting/covers.png'});
 assert.deepEqual(errors,[]);console.log('Hosting and storage: automatic LAN selection, account identity, live roster, avatars, invite, lock, renewal, kick, stop, real categories, read errors, full covers, contextual visuals passed');await browser.close();
})().catch(e=>{console.error(e);process.exitCode=1});
