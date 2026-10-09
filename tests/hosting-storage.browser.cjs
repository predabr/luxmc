const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({ locale: 'en-US',viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(String(e)));
 await page.addInitScript(()=>{
  window.calls=[];window.room=null;window.failStorage=false;window.clipboardText='';
  window.lan={installed:false,supported:true,active:false,invitation:null,address:null,peers:[],error:null};
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
   if(command==='virtual_lan_status')return structuredClone(window.lan);
   if(command==='virtual_lan_worlds')return {worlds:[{id:'10.100.1.2',owner:'Alex',name:'Mundo do Alex',localAddress:'127.0.0.1:25580',version:'1.21.1',latencyMs:42}],localWorld:{motd:'Mundo do Steve',port:54321},error:null};
   if(command==='virtual_lan_world_port')return null;
   if(command==='virtual_lan_install'){window.lan.installed=true;return null}
   if(command==='virtual_lan_connect'){window.lan={...window.lan,preparation:{stage:'downloading',downloadedBytes:5242880,totalBytes:10485760}};await new Promise(resolve=>setTimeout(resolve,4500));window.lan={...window.lan,installed:true,preparation:null,active:true,invitation:args.invite||'luxmc-lan:private_payload',address:'10.100.1.1',peers:[{name:'Alex',address:'10.100.1.2'}],error:null};return null}
   if(command==='virtual_lan_stop'){window.lan={...window.lan,active:false,invitation:null,address:null,peers:[]};return null}
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
 assert.equal(await page.getByRole('button',{name:'Instalar componentes de rede',exact:true}).count(),0);
 await page.getByRole('button',{name:'Criar sala',exact:true}).click();
 await page.getByRole('heading',{name:'Baixando componentes',exact:true}).waitFor();
 assert.equal(await page.getByRole('progressbar',{name:'Download dos componentes de rede'}).getAttribute('aria-valuenow'),'50');
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='virtual_lan_install').length),0);
 await page.getByRole('heading',{name:'Sua sala está pronta',exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Preparar minha instância',exact:true}).count(),0);
 assert.equal(await page.getByText('10.100.1.1',{exact:true}).count(),0);
 await page.getByRole('heading',{name:'Mundos para jogar',exact:true}).waitFor();
 await page.getByText('Mundo do Alex',{exact:true}).waitFor();
 await page.evaluate(()=>{window.lan.active=false;window.lan.error='Reconectando os computadores';});
 await page.getByRole('heading',{name:'A conexão precisa de atenção',exact:true}).waitFor();
 await page.getByText('Mundo do Alex',{exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Sair da sala',exact:true}).isEnabled(),true);
 await page.evaluate(()=>{window.lan.active=true;window.lan.error=null;});
 await page.getByRole('heading',{name:'Sua sala está pronta',exact:true}).waitFor();
 await page.getByRole('button',{name:'Ajuda de conexão',exact:true}).click();
 await page.getByText('10.100.1.1',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Meu mundo não foi encontrado',exact:true}).click();
 await page.getByRole('textbox',{name:'Porta do meu mundo'}).fill('54321');
 await page.getByRole('button',{name:'Verificar e compartilhar',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='virtual_lan_world_port').at(-1).args.port),54321);
 fs.mkdirSync('docs/validation/revision6',{recursive:true});
 await page.screenshot({path:'docs/validation/revision6/friends-room.png'});
 await page.getByRole('button',{name:'Convidar amigo',exact:true}).click();
 assert.equal(await page.evaluate(()=>window.clipboardText),'luxmc://join/lan?invitation=luxmc-lan%3Aprivate_payload');
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='host_world').length),0);
 await page.locator('aside a[href="/skins"]').click();await page.waitForURL('**/skins');
 await page.locator('aside a[href="/friends"]').click();await page.waitForURL('**/hosting');
 await page.getByRole('heading',{name:'Sua sala está pronta',exact:true}).waitFor();
 await page.getByRole('button',{name:'Sair da sala',exact:true}).click();
 await page.getByRole('button',{name:'Criar sala',exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.lan.active),false);
 await page.getByRole('textbox',{name:'Convite de um amigo'}).fill('luxmc://join/lan?invitation=luxmc-lan%3Afriend_payload');
 await page.getByRole('button',{name:'Entrar na sala',exact:true}).click();
 await page.getByRole('heading',{name:'Sua sala está pronta',exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>window.calls.filter(c=>c.command==='virtual_lan_connect').at(-1).args.invite),'luxmc-lan:friend_payload');
 await page.getByRole('button',{name:'Sair da sala',exact:true}).click();
 fs.mkdirSync('docs/visual/2026-10-03-hosting',{recursive:true});
 await page.goto('http://127.0.0.1:1420/settings');await page.getByRole('tab',{name:'Armazenamento',exact:true}).click();await page.getByText('Aplicativo Luxmc',{exact:true}).waitFor();await page.getByText('Java do launcher',{exact:true}).waitFor();assert.ok(await page.getByText('4 MB',{exact:true}).count());assert.equal(await page.locator('.shader-scene').count(),0);
 await page.evaluate(()=>window.failStorage=true);await page.getByRole('button',{name:'Atualizar',exact:true}).click();await page.locator('main').getByRole('alert').filter({hasText:'Falha simulada de leitura'}).waitFor();
 await page.goto('http://127.0.0.1:1420/mods');await page.getByText('Zombie Invade 100 Days',{exact:true}).waitFor();assert.equal(await page.locator('.shader-scene').count(),0);
 for(const title of ['Zombie Invade 100 Days','SkyFactory 4']){const card=page.locator('.catalog-card').filter({has:page.getByText(title,{exact:true})});const image=card.locator('img.relative').first();const imageBox=await image.boundingBox();const cardBox=await card.boundingBox();assert.ok(imageBox.width>250&&imageBox.height>=180&&Math.abs(imageBox.width-cardBox.width)<=3);assert.equal(await image.evaluate(i=>getComputedStyle(i).objectFit),'cover')}
 await page.screenshot({path:'docs/visual/2026-10-03-hosting/covers.png'});
 assert.deepEqual(errors,[]);console.log('Virtual LAN and storage: setup, invite, navigation persistence, stop, join, real categories, read errors and covers passed');await browser.close();
})().catch(e=>{console.error(e);process.exit(1)});
