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
   const world={ownerId:'alex',ownerUsername:'Alex',motd:'Mundo do Alex',localAddress:'127.0.0.1:25580',compatibility:{mcVersion:'1.21.1',loader:'vanilla',modFingerprint:'a'.repeat(64),modCount:0}};
   if(command==='host_world'){await new Promise(resolve=>setTimeout(resolve,200));window.room={mode:'host',invitation:'LUXMC1-1234567890123456',localAddress:null,expiresAt:9999999999,pingMs:null,transport:'QUIC',roomCode:'LUXMC1-1234567890123456',members:[host],maxPlayers:10,roomLocked:false,worldReady:true,worlds:[world]};return structuredClone(window.room)}
   if(command==='join_world'){window.room={mode:'guest',invitation:null,localAddress:'127.0.0.1:25580',expiresAt:9999999999,pingMs:42,transport:'QUIC',roomCode:null,members:[host,{...host,id:'alex',username:'Alex',isHost:false}],maxPlayers:10,roomLocked:false,worldReady:true,worlds:[world]};return structuredClone(window.room)}
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
 assert.equal(await page.getByRole('button',{name:'Instalar componentes de rede',exact:true}).count(),0);
 await page.getByRole('button',{name:'Criar minha sala',exact:true}).click();
 await page.getByText('Sua sala está conectada',{exact:true}).waitFor();
 await page.getByRole('heading',{name:'Mundos da turma',exact:true}).waitFor();
 await page.getByText(/Mundo do Alex/).waitFor();
 assert.equal(await page.getByText('Conexão direta',{exact:false}).count(),0);
 await page.getByRole('button',{name:'Copiar convite',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.clipboardText),'LUXMC1-1234567890123456');
 await page.getByRole('button',{name:'Fechar novas entradas',exact:true}).click();
 await page.getByRole('button',{name:'Permitir entradas',exact:true}).waitFor();
 await page.locator('aside a[href="/skins"]').click();await page.waitForURL('**/skins');
 await page.locator('aside a[href="/friends"]').click();await page.waitForURL('**/friends');
 await page.waitForTimeout(1000);assert.ok(page.url().endsWith('/friends'));
 await page.getByRole('button',{name:'Jogar juntos',exact:true}).first().click();
 await page.getByText('Sua sala está conectada',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Sair',exact:true}).click();
 await page.getByRole('textbox',{name:'Convite da sala'}).fill('LUXMC1-1234567890123456');
 await page.getByRole('button',{name:'Entrar na sala',exact:true}).click();
 await page.getByText('Sua sala está conectada',{exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='join_world').at(-1).args.invitation),'LUXMC1-1234567890123456');
 assert.equal(await page.evaluate(()=>window.calls.some(c=>c.command.startsWith('virtual_lan_'))),false);
 await page.getByRole('button',{name:'Sair',exact:true}).click();
 fs.mkdirSync('docs/visual/2026-10-03-hosting',{recursive:true});
 await page.goto('http://127.0.0.1:1420/settings');await page.getByRole('tab',{name:'Armazenamento',exact:true}).click();await page.getByText('Aplicativo Luxmc',{exact:true}).waitFor();await page.getByText('Java do launcher',{exact:true}).waitFor();assert.ok(await page.getByText('4 MB',{exact:true}).count());assert.equal(await page.locator('.shader-scene').count(),0);
 await page.evaluate(()=>window.failStorage=true);await page.getByRole('button',{name:'Atualizar',exact:true}).click();await page.locator('main').getByRole('alert').filter({hasText:'Falha simulada de leitura'}).waitFor();
 await page.goto('http://127.0.0.1:1420/mods');await page.getByText('Zombie Invade 100 Days',{exact:true}).waitFor();assert.equal(await page.locator('.shader-scene').count(),0);
 for(const title of ['Zombie Invade 100 Days','SkyFactory 4']){const card=page.locator('.catalog-card').filter({has:page.getByText(title,{exact:true})});const image=card.locator('img.relative').first();const imageBox=await image.boundingBox();const cardBox=await card.boundingBox();assert.ok(imageBox.width>250&&imageBox.height>=180&&Math.abs(imageBox.width-cardBox.width)<=3);await image.evaluate(i=>i.decode());assert.equal(await image.evaluate(i=>getComputedStyle(i).objectFit),'contain');assert.equal(await image.evaluate(i=>i.style.opacity),'1')}
 await page.screenshot({path:'docs/visual/2026-10-03-hosting/covers.png'});
 assert.deepEqual(errors,[]);console.log('Native room and storage: setup, invite, navigation persistence, stop, join, real categories, read errors and uncropped fallback artwork passed');await browser.close();
})().catch(e=>{console.error(e);process.exit(1)});
