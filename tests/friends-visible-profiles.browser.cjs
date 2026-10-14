const { chromium }=require('playwright-core');
const { setupLauncherDemo }=require('../scripts/launcher-video-fixture.cjs');
const assert=require('node:assert/strict');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        const page=await browser.newPage({viewport:{width:1440,height:1050}});
        await setupLauncherDemo(page);
        await page.addInitScript(()=>{
            const state=window.launcherDemo.state,base=state.contacts[0];
            state.contacts=Array.from({length:100},(_,i)=>({...base,id:`00000000-0000-4000-8000-${String(i+100).padStart(12,'0')}`,username:`Player${String(i).padStart(3,'0')}`,status:'online',incoming:false}));
            window.friendProfileRequests=[];window.friendProfileActive=0;window.friendProfileMaximum=0;
            const invoke=window.electronAPI.invoke;
            window.electronAPI.invoke=async(command,args={})=>{
                if(command==='social_request'&&args.request.action==='profile_get'){
                    const target=args.request.targetId;
                    if(target){window.friendProfileRequests.push(target);window.friendProfileMaximum=Math.max(window.friendProfileMaximum,++window.friendProfileActive);await new Promise(r=>setTimeout(r,80));window.friendProfileActive--;}
                    return {profile:{id:target||state.account.uuid,username:'LuxPlayer',displayName:'',description:'',status:'',banner:'',portrait:'/grass_head.png',packs:[],collections:[]}};
                }
                return invoke(command,args);
            };
        });
        await page.goto('http://127.0.0.1:1420/friends');
        await page.locator('.friend-row').first().waitFor();
        await page.waitForFunction(()=>window.friendProfileRequests.length>0);
        await page.waitForTimeout(800);
        const initial=await page.evaluate(()=>new Set(window.friendProfileRequests).size);
        assert.ok(initial>0&&initial<50,`Buscou ${initial} perfis de 100 fora da tela`);
        const last='00000000-0000-4000-8000-000000000199';
        assert.equal(await page.evaluate(id=>window.friendProfileRequests.includes(id),last),false);
        await page.getByRole('button',{name:'Ver perfil: Player099',exact:true}).last().scrollIntoViewIfNeeded();
        await page.waitForFunction(id=>window.friendProfileRequests.includes(id),last);
        assert.ok(await page.evaluate(()=>window.friendProfileMaximum<=3));
        console.log(JSON.stringify({totalFriends:100,initialProfileQueries:initial,scrollLoadsLastProfile:true,maximumConcurrent:await page.evaluate(()=>window.friendProfileMaximum)}));
    } finally {await browser.close();}
})().catch(error=>{console.error(error.message);process.exitCode=1;});
