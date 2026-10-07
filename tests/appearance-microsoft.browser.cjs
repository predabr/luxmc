const { chromium }=require('playwright-core');
const fs=require('node:fs');
const assert=require('node:assert/strict');
(async()=>{
 const skin='data:image/png;base64,'+fs.readFileSync('static/alex.png').toString('base64');
 const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
 const page=await browser.newPage({viewport:{width:1440,height:1000}});
 const errors=[];page.on('pageerror',error=>errors.push(String(error)));
 await page.addInitScript(texture=>{
  window.calls=[];window.failSync=true;
  localStorage.setItem('luxmc.locale','pt-BR');
  localStorage.setItem('luxmc_saved_skins',JSON.stringify([{id:'saved-alex',name:'Alex de teste',url:texture,model:'alex'}]));
  localStorage.setItem('luxmc_selected_skin_id','saved-alex');
  window.fixtureAccount=JSON.parse(sessionStorage.getItem('appearance-account')||'null')||{id:'fixture-ms',uuid:'11111111111111111111111111111111',username:'TestPlayer',accessToken:'x'.repeat(150),skinUrl:texture,skinVariant:'slim',capeUrl:null};
  const invoke=async(command,args)=>{
   window.calls.push({command,args});
   if(command==='auth_accounts')return [window.fixtureAccount];
   if(command==='app_init'){await new Promise(resolve=>setTimeout(resolve,100));return {account:window.fixtureAccount,profiles:[],activeProfileId:null,devMode:true};}
   if(command==='auth_save_appearance'){window.fixtureAccount.skinUrl=args.skinUrl;window.fixtureAccount.skinVariant=args.variant;window.fixtureAccount.capeUrl=args.capeUrl;sessionStorage.setItem('appearance-account',JSON.stringify(window.fixtureAccount));return null;}
   if(command==='plugin:dialog|open')return 'C:/fixture/cape.png';
   if(command==='auth_read_local_texture')return window.fixtureCape;
   if(command==='auth_change_skin'){if(window.failSync){window.failSync=false;throw Error('Falha Microsoft simulada');}return null;}
   if(command==='plugin:store|load'||command==='plugin:event|listen')return 1;
   if(command==='plugin:store|get')return [{animations:false,liveWallpaper:false,soundscapesEnabled:false},true];
   if(['deep_links_take','profiles_list','instances_list','screenshots_list','changelog_get'].includes(command))return [];
   if(command==='java_scan')return {runtimes:[]};
   if(command==='get_system_specs')return {totalRamMb:16384,osDistro:'Windows 11',arch:'x86_64'};
   return null;
  };
  window.electronAPI={invoke,on:()=>()=>{}};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={invoke,transformCallback:()=>1,convertFileSrc:()=>''};
 },skin);
 await page.goto((process.env.LUXMC_BASE_URL||'http://127.0.0.1:1420')+'/skins');
 const apply=page.getByRole('button',{name:'Aplicar e sincronizar com Microsoft',exact:true});
 await apply.waitFor();
 assert.equal(await page.getByRole('button',{name:'Sincronizar com Microsoft',exact:true}).count(),0);
 const cape=page.getByRole('button',{name:'Importar capa PNG',exact:true});
 assert.ok((await cape.boundingBox()).height>=48);
 await page.evaluate(async()=>{const {getFullCapeDataUrl}=await import('/src/lib/utils/capeTextures.ts');window.fixtureCape=getFullCapeDataUrl('luxmc');});
 await cape.click();
 await page.getByText('Sua capa atual',{exact:true}).waitFor();
 await apply.click();
 await page.getByText(/Aparência salva no launcher. Falha na sincronização Microsoft/).first().waitFor();
 assert.equal(await apply.isEnabled(),true);
 let calls=await page.evaluate(()=>window.calls);
 assert.equal(calls.find(call=>call.command==='auth_change_skin').args.skinUrl,skin);
 assert.ok(calls.findIndex(call=>call.command==='auth_save_appearance')<calls.findIndex(call=>call.command==='auth_change_skin'));
 await apply.click();
 await page.getByText(/Aparência salva e skin sincronizada com Minecraft/).waitFor();
 await page.reload();await apply.waitFor();
 await page.getByText('Sua capa atual',{exact:true}).waitFor();
 assert.equal(await page.getByRole('button',{name:'Sem capa',exact:true}).getAttribute('aria-pressed'),'false');
 assert.equal(await page.getByRole('button',{name:'Capa Luxmc Oficial',exact:true}).count(),0);
 assert.ok((await page.getByRole('button',{name:/Adicionar skin/}).boundingBox()).height>=200);
 await page.getByText(/Sua aparência está aplicada/).waitFor();
 assert.deepEqual(errors,[]);
 fs.mkdirSync('docs/visual/2026-10-04-windows-speed',{recursive:true});
 await page.screenshot({path:'docs/visual/2026-10-04-windows-speed/personalizador.png'});
 console.log('Combined Microsoft action preserves local PNG on failure, retries successfully, restores cape after reload, and exposes larger import controls; no UI errors.');
 await browser.close();
})().catch(error=>{console.error(error);process.exit(1)});
