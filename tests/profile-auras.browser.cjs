const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        for(const plan of ['plus','pro']) {
            const page=await browser.newPage({viewport:{width:1440,height:1050},reducedMotion:'reduce'});
            const errors=[];page.on('pageerror',error=>errors.push(error.message));
            await setupLauncherDemo(page);
            await page.addInitScript(plan=>{
                const original=window.electronAPI.invoke;
                window.electronAPI.invoke=async(command,args={})=>{
                    if(command==='social_request'&&args.request.action==='profile_get')return {profile:{id:window.launcherDemo.state.account.uuid,username:'LuxPlayer',displayName:'Explorer',status:'',description:'',banner:'',portrait:'',packs:[],collections:[],profileStyle:{accent:'rose',frame:'double',effect:'pulse',layout:'classic'},membership:{plan,expiresAt:1893456000}}};
                    return original(command,args);
                };
            },plan);
            await page.goto('http://127.0.0.1:1420/friends');
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).click();
            const dialog=page.getByRole('dialog');
            await dialog.getByRole('tab',{name:'Aparência',exact:true}).click();
            const aura=dialog.getByRole('button',{name:'Aura do avatar: Aurora',exact:true});
            await aura.click();
            if(plan==='pro') {
                assert.equal(await aura.getAttribute('aria-pressed'),'true');
                const preview=dialog.locator('.signature-preview');
                assert.equal(await preview.getAttribute('data-profile-ambient-effect'),'aurora');
                assert.equal(await preview.getAttribute('data-profile-effect'),'none');
                assert.equal(await preview.locator('[data-profile-avatar]').evaluate(element=>getComputedStyle(element,'::before').animationName),'none');
                await dialog.getByRole('button',{name:'Aura do avatar: Sinal luminoso',exact:true}).click();
                assert.equal(await preview.getAttribute('data-profile-ambient-effect'),'beacon');
            } else {
                assert.equal(await aura.getAttribute('aria-pressed'),'false');
                await page.getByText('Disponível com Pro',{exact:true}).first().waitFor();
            }
            assert.deepEqual(errors,[]);
            await page.close();
        }
        console.log('Pro auras select distinctly, respect reduced motion and keep Plus gated.');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
