const { chromium } = require('playwright-core');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.LUXMC_BROWSER_BASE_URL || 'http://127.0.0.1:1420';
const output = 'docs/validation/artwork-personalizer';

(async () => {
    const browser = await chromium.launch({ headless:true, executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE });
    fs.mkdirSync(output,{recursive:true});
    try {
        const page = await browser.newPage({viewport:{width:1600,height:1000}});
        const errors=[];
        page.on('pageerror',error=>errors.push(String(error)));
        await setupLauncherDemo(page);
        await page.addInitScript(()=>{
            const invoke=window.electronAPI.invoke;
            window.launcherDemo.state.contacts=window.launcherDemo.state.contacts.filter(friend=>friend.status!=='pending');
            window.galleryCalls=[];
            const pack=(id,title,bannerUrl=null)=>({source:'modrinth',sourceId:id,slug:id,title,description:'Galeria do próprio projeto, sem trocar a identidade do modpack.',downloads:12345,iconUrl:'/modpack_vanilla_perfected_icon.webp',bannerUrl,author:'Fixture',categories:['fabric'],versions:['Forge','Fabric','1.20.1']});
            window.electronAPI.invoke=async(command,args={})=>{
                if(command==='mods_search') return [
                    {...pack('1KVo5zza','Fabulously Optimized'),iconUrl:'https://cdn.modrinth.com/data/1KVo5zza/test-icon.webp'},
                    pack('gallery-fixture','Pack com galeria','/modpack_vanilla_perfected_icon.webp'),
                    pack('empty-fixture','Pack só com logo','/missing-cover.png'),
                    pack('wrong-fixture','Resposta de outro projeto')
                ];
                if(command==='mods_project_details'){
                    window.galleryCalls.push(args);
                    return {id:args.projectId==='wrong-fixture'?'foreign-id':args.projectId,source:'modrinth',gallery:args.projectId==='empty-fixture'?[]:[{url:'/modpack_vanilla_perfected_icon.webp'},{url:'/modpack_vanilla_perfected.png'}]};
                }
                if(command==='instance_pack_reference') return {source:'curseforge',projectId:'396246',versionId:'fixture',name:'Fabulously Optimized'};
                return invoke(command,args);
            };
        });
        await page.route('https://cdn.modrinth.com/data/1KVo5zza/test-icon.webp',route=>route.fulfill({path:'static/modpack_fo_icon.png',contentType:'image/png'}));
        await page.goto(`${base}/mods`);
        const cover=title=>page.locator('.catalog-card').filter({has:page.getByText(title,{exact:true})}).locator('.catalog-cover img.relative');
        await cover('Fabulously Optimized').waitFor();
        for(const close of await page.locator('.notification button[aria-label]').all()) await close.click();
        await page.waitForFunction(()=>[...document.querySelectorAll('.catalog-cover img.relative')].every(image=>image.complete&&image.naturalWidth&&Number(image.style.opacity)>0));
        await page.waitForFunction(()=>[...document.querySelectorAll('.catalog-card')].find(card=>card.textContent.includes('Pack com galeria'))?.querySelector('img.relative')?.getAttribute('src')==='/modpack_vanilla_perfected.png');
        assert.equal(await cover('Fabulously Optimized').getAttribute('src'),'/modpack_fo.webp');
        assert.equal(await cover('Pack só com logo').getAttribute('src'),'/modpack_vanilla_perfected_icon.webp');
        assert.equal(await cover('Resposta de outro projeto').getAttribute('src'),'/modpack_vanilla_perfected_icon.webp');
        assert.equal(await cover('Pack só com logo').evaluate(image=>image.style.opacity),'0.4');
        assert.equal(await cover('Pack com galeria').evaluate(image=>image.style.opacity),'1');
        assert.equal(await page.getByText('MC Forge',{exact:true}).count(),0);
        assert.equal(await page.evaluate(()=>window.galleryCalls.filter(call=>call.projectId==='gallery-fixture').length),1);
        await page.screenshot({path:`${output}/catalog-dark.png`,animations:'disabled'});
        await page.evaluate(async()=>{
            const {profiles}=await import('/src/lib/stores/profiles.svelte.ts');
            profiles.list=[{id:'fo-old',name:'Fabulously Optimized',icon:'https://media.forgecdn.net/avatars/test-fo.png',mcVersion:'1.20.1',loader:'fabric',gameDir:'C:/Fixture/FO',createdAt:1,updatedAt:1}];
        });
        await page.waitForFunction(()=>localStorage.getItem('luxmc_catalog_banner_fo-old')==='/modpack_fo.webp');
        await page.goto(`${base}/skins`);
        await page.getByRole('region',{name:'Visualizador 3D de Skin'}).locator('canvas').first().waitFor({state:'visible'});
        for(const close of await page.locator('.notification button[aria-label]').all()) await close.click();
        await page.getByRole('button',{name:'Modelo e jogador',exact:true}).click();
        const modelEditor=page.locator('.fixed.inset-0').filter({has:page.getByText('Editar Configurações da Skin',{exact:true})});
        await modelEditor.getByRole('button',{name:/Steve/}).click();
        await modelEditor.getByRole('button',{name:/Alex/}).click();
        await modelEditor.locator('button').first().click();
        for(const name of ['Parado','Andar','Correr','Voar']){
            await page.getByRole('button',{name,exact:true}).click();
            assert.equal(await page.getByRole('button',{name,exact:true}).getAttribute('aria-pressed'),'true');
        }
        for(const name of ['Frente','Costas','3D']) await page.getByRole('button',{name,exact:true}).click();
        await page.getByRole('button',{name:'Parado',exact:true}).click();
        await page.getByTitle('Pausar rotação',{exact:true}).click();
        await page.getByRole('button',{name:'3D',exact:true}).click();
        await page.waitForTimeout(200);
        await page.screenshot({path:`${output}/personalizer-dark.png`,animations:'disabled'});
        await page.evaluate(async()=>{
            const {themeStore}=await import('/src/lib/stores/theme.svelte.ts');
            themeStore.setCustomWallpaper(new URL('/bg_day.jpg',location.href).href,'image','Fixture',false);
        });
        await page.waitForFunction(()=>document.documentElement.classList.contains('has-custom-wallpaper'));
        await page.waitForTimeout(600);
        await page.screenshot({path:`${output}/personalizer-wallpaper.png`,animations:'disabled'});
        for(const width of [1080,900,640]){
            await page.setViewportSize({width,height:950});
            await page.waitForTimeout(150);
            const overflow=await page.evaluate(()=>{
                const main=document.querySelector('main').getBoundingClientRect();
                return [...document.querySelectorAll('.studio-preview,.studio-panel,.appearance-apply,.studio-controls button')].some(node=>{const box=node.getBoundingClientRect();return box.left<main.left-1||box.right>main.right+1;});
            });
            assert.equal(overflow,false,`personalizer overflow at ${width}`);
        }
        await page.screenshot({path:`${output}/personalizer-compact.png`,animations:'disabled'});
        await page.setViewportSize({width:1600,height:1000});
        await page.evaluate(async()=>(await import('/src/lib/stores/theme.svelte.ts')).themeStore.setTheme('light',false));
        await page.waitForTimeout(200);
        await page.screenshot({path:`${output}/personalizer-light.png`,animations:'disabled'});
        assert.deepEqual(errors,[]);
        console.log('Gallery identity, square cover recovery, original logo fallback, Minecraft version badges, installed FO recovery and responsive personalizer verified.');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
