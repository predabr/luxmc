const {chromium}=require(process.env.LUXMC_PLAYWRIGHT_MODULE || 'playwright');
const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});
 const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>{errors.push(String(e));console.error('PAGE ERROR',String(e))});
 await page.addInitScript(()=>{
   const profile={id:'fabric-profile',name:'Fabric de teste',icon:'grass_block',mcVersion:'1.20.1',loader:'fabric',loaderVersion:'0.16.0',gameDir:'/tmp/luxmc-test',createdAt:new Date().toISOString(),updatedAt:new Date().toISOString(),ramMb:4096,modCount:2};
   const account={id:'offline_test',uuid:'offline_test',username:'Steve',accessToken:'',skinUrl:'https://mineskin.eu/skin/Steve'};
   let tunnel=null; localStorage.setItem("luxmc.locale","pt-BR");
   const events={};const queue=[];window.testCalls=[];
   const invoke=async(command,args)=>{
     window.testCalls.push({command,args});
     if(command==='tunnel_status') return tunnel;
     if(command==='p2p_scan_lan_worlds') return [{port:54321,host:'127.0.0.1',motd:'World'}];
     if(command==='p2p_get_local_info') return {ip:'127.0.0.1',port:54321};
     if(command==='host_world') return tunnel={mode:'host',invitation:'luxmc-world:test',localAddress:null,expiresAt:9999999999,pingMs:null,transport:'QUIC'};
     if(command==='join_world') return tunnel={mode:'client',invitation:null,localAddress:'127.0.0.1:45678',expiresAt:9999999999,pingMs:10,transport:'QUIC'};
     if(command==='stop_session') {tunnel=null;return null;}
     if(command==='launch_game') return {pid:123};
     if(command==='app_init')return {account,profiles:[profile],activeProfileId:profile.id,devMode:true};
     if(command==='deep_links_take')return queue.splice(0);
     if(command==='mods_project_details')return {id:args.projectId,slug:args.projectId,title:args.projectId==='sodium'?'Sodium':'Fabulously Optimized',description:'Projeto de teste',body:'',bodyType:'markdown',iconUrl:null,downloads:1000000,categories:['fabric'],loaders:['fabric'],gameVersions:['1.20.1'],source:args.source,gallery:[]};
     if(command==='curseforge_status')return true;
     if(command==='profiles_list'||command==='instances_list')return [profile];
     if(command==='mods_versions')return [{id:'v1',name:'v1',versionNumber:'1',files:[{filename:'pack.mrpack',url:'https://cdn.modrinth.com/pack.mrpack',size:1,sha1:''}]}];
     if(command==='mods_download_to_temp')return '/tmp/pack.mrpack';
     if(command==='instance_import_mrpack')return {...profile,id:'new-profile',name:'Fabulously Optimized'};
     if(command==='plugin:store|load')return 1;
     if(command==='plugin:store|get')return [{animations:false,liveWallpaper:false,soundscapesEnabled:false,soundEnabled:false},true];
     if(command==='plugin:event|listen')return 1;
     if(command==='mods_search'||command==='java_scan'||command==='screenshots_list'||command==='skins_list')return [];
     if(command==='get_system_specs')return {totalRamMb:16384,osDistro:'Linux',arch:'x86_64'};
     if(command==='minecraft_uuid')return null;
     if(command==='auth_resolve_texture')return window.testSkinData;
     if(command==='social_request' && args.request.action==='stream_ticket') return {url:null};
     if(command==='mesh_status')return {available:false,state:'Tailscale não instalado',ip:null,peers:[]};
     if(command==='auth_save_appearance') return null;
     if(command==='social_request')return {me:{id:'me',username:'Steve'},friends:[{id:'friend-id',username:'Alex',status:'in_game',incoming:false,lastSeen:null,activity:'Fabric de teste',mcVersion:'1.20.1',loader:'fabric',serverIp:'friend.example.org',serverPort:25567}]};
     return null;
   };
   window.electronAPI={invoke,on:(event,callback)=>{(events[event]??=[]).push(callback);return ()=>events[event]=events[event].filter(v=>v!==callback)}};
   window.__TAURI_INTERNALS__={invoke,transformCallback:()=>1,convertFileSrc:()=>'/grass_head.png'};
   window.testEvent=(event,payload)=>{for(const callback of events[event]||[])callback(payload)};
   window.testLink=url=>{queue.push(url);for(const callback of events['deep-link-pending']||[])callback(null)};
 });

 const fs = require('node:fs');
 fs.mkdirSync('docs/visual/2026-09-23', {recursive:true});
 const skin = fs.readFileSync('static/steve.png');
 await page.route(/https:\/\/(mineskin\.eu|mc-heads\.net|minotar\.net)\//, route => route.fulfill({status:200,contentType:'image/png',body:skin}));
 await page.addInitScript(data=>window.testSkinData=data,'data:image/png;base64,'+require('node:fs').readFileSync('static/steve.png').toString('base64'));
 await page.goto('http://127.0.0.1:1420/');

 await page.locator('aside').last().waitFor();
 const sidebar = await page.locator('aside').last().boundingBox();
 assert.ok(Math.abs(sidebar.y + sidebar.height - 988) <= 2, JSON.stringify(sidebar));
 await page.evaluate(()=>document.documentElement.classList.add('has-custom-wallpaper'));
 for (const filter of [page.getByRole('button',{name:'Ordenar instâncias'}),page.getByRole('button',{name:'Grupo de instâncias'})]) assert.equal(await filter.evaluate(e=>getComputedStyle(e).backgroundColor), 'rgba(0, 0, 0, 0)');
 await page.screenshot({path:'docs/visual/2026-09-23/audit-home.png'});

 const news = page.locator('aside').last().locator('div').filter({has:page.getByRole('link',{name:'Ver todas',exact:true})}).first();
 const newsBottom = await page.locator('aside').last().locator('a[href="/news"]').last().evaluate(e=>e.parentElement.getBoundingClientRect().bottom);
 assert.ok(newsBottom>=968, String(newsBottom));
 await page.route('https://mineskin.eu/skin/Broken',route=>route.abort());
 await page.evaluate(()=>window.testLink('luxmc://skin/apply?url=https%3A%2F%2Fmineskin.eu%2Fskin%2FBroken&model=slim'));
 await page.waitForURL('**/skins');
 await page.getByRole('region',{name:'Visualizador 3D de Skin'}).locator('canvas').first().waitFor({state:'visible',timeout:30000});
 assert.equal(await page.evaluate(()=>window.testCalls.filter(c=>c.command==='auth_save_appearance').length),0);
 await page.getByRole('button',{name:'Aplicar skin e capa',exact:true}).click();
 await page.waitForFunction(()=>window.testCalls.some(c=>c.command==='auth_save_appearance' && c.args.skinUrl.startsWith('data:image/png;base64,')));
 await page.screenshot({path:'docs/visual/2026-09-23/audit-skins.png'});
 const region=page.getByRole('region',{name:'Visualizador 3D de Skin'});
 await region.locator('canvas').first().evaluate(canvas=>{
   const gl=canvas.getContext('webgl2') || canvas.getContext('webgl');
   window.contextLossExtension=gl.getExtension('WEBGL_lose_context');
   window.contextLossExtension.loseContext();
 });
 await region.getByText('A prévia 3D foi interrompida. Sua seleção está preservada.').waitFor();
 await page.waitForTimeout(1000);
 await page.evaluate(()=>window.contextLossExtension.restoreContext());
 await region.locator('canvas').first().waitFor({state:'visible',timeout:15000});


 const capeChecks=await page.evaluate(async()=>{
   const {getFullCapeDataUrl,migrateGeneratedCape,getCapePreviewDataUrl}=await import('/src/lib/utils/capeTextures.ts');
   const {loadTextureImage,normalizeCape}=await import('/src/lib/utils/textureImage.ts');
   const signal=new AbortController().signal;
   const canonical=getFullCapeDataUrl('vanilla');
   const texture=await loadTextureImage(canonical,signal);
   const full=normalizeCape(texture);const ctx=full.getContext('2d');
   const outside=ctx.getImageData(1,1,10,16);const inside=ctx.getImageData(12,1,10,16);
   ctx.putImageData(outside,12,1);ctx.putImageData(inside,1,1);
   const corrected=await migrateGeneratedCape(full.toDataURL(),signal);
   const untouched=await migrateGeneratedCape(canonical,signal);
   const hd=document.createElement('canvas');hd.width=128;hd.height=64;const h=hd.getContext('2d');h.fillStyle='rgb(10,20,30)';h.fillRect(24,2,1,1);
   const imported=await loadTextureImage(hd.toDataURL(),signal);const normalized=normalizeCape(imported);
   return {legacy:corrected===canonical,canonical:untouched===canonical,hdPixel:[...normalized.getContext('2d').getImageData(24,2,1,1).data],preview:getCapePreviewDataUrl('vanilla').startsWith('data:')};
 });
 assert.deepEqual(capeChecks,{legacy:true,canonical:true,hdPixel:[10,20,30,255],preview:true});
 const canvas=region.locator('canvas').first();
 await canvas.evaluate(c=>window.originalSkinCanvas=c);
 const sidebarAvatar=page.locator('aside').first().locator('img').last();
 for (const name of ['Alex','Steve','Alex','Steve']) {
   const saves=await page.evaluate(()=>window.testCalls.filter(c=>c.command==='auth_save_appearance').length);
   await page.getByRole('button',{name:new RegExp('^'+name+' ')}).first().click();
   await page.waitForTimeout(150);
   assert.equal(await page.evaluate(()=>window.testCalls.filter(c=>c.command==='auth_save_appearance').length),saves);
   await page.getByRole('button',{name:'Aplicar skin e capa',exact:true}).click();
   await page.waitForFunction(count=>window.testCalls.filter(c=>c.command==='auth_save_appearance').length>count,saves);
   await page.waitForFunction(()=>document.querySelector('[aria-label="Visualizador 3D de Skin"] canvas')===window.originalSkinCanvas && !window.originalSkinCanvas.classList.contains('invisible'));
 }
 await page.waitForFunction(()=>document.querySelector('aside img:last-of-type')?.src.startsWith('data:') || [...document.querySelectorAll('aside img')].some(i=>i.src.startsWith('data:')));
 await page.evaluate(async()=>{const {appState}=await import('/src/lib/stores/app.svelte.ts');appState.isGameRunning=true});
 await page.setViewportSize({width:1280,height:720});
 await page.waitForTimeout(150);
 assert.ok(await canvas.evaluate(c=>c.width>100 && c.height>100 && !c.classList.contains('invisible')));
 const left=await page.locator('aside').first().boundingBox();
 assert.equal(left.y,0);assert.equal(left.height,720);
 await page.evaluate(async()=>{const {appState}=await import('/src/lib/stores/app.svelte.ts');appState.isGameRunning=false});
 await page.getByRole('link',{name:'Amigos'}).first().click();
 await page.getByRole('button',{name:'Jogar com amigos'}).click();
 const p2p=page.getByRole('generic',{name:'Jogar com amigos'});
 assert.equal(await page.locator('[aria-label="Jogar com amigos"] > section').count(),2);
 assert.equal(await page.getByText('Hospedar Sessão P2P (UPnP)').count(),0);
 await page.getByRole('button',{name:'Abrir para amigos'}).click();
 await page.getByRole('button',{name:'Copiar código'}).waitFor();
 await page.getByRole('button',{name:'Encerrar partida'}).click();
 await page.getByPlaceholder('Cole o código de convite aqui…').fill('luxmc-world:test');
 await page.getByRole('button',{name:'Entrar no mundo',exact:true}).click();
 await page.waitForFunction(()=>window.testCalls.some(c=>c.command==='launch_game' && c.args.request.serverIp==='127.0.0.1'));
 await page.screenshot({path:'docs/visual/2026-09-23/audit-p2p.png'});
 assert.deepEqual(errors, []);
 console.log(JSON.stringify({sidebar,transparentFilters:true,skinFallback:true,stableCanvas:true,capeChecks,p2pOneClick:true,errors}));
 await browser.close();
})().catch(e=>{console.error(e);process.exit(1)});
