const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const {setupLauncherDemo}=require('../scripts/launcher-video-fixture.cjs');

(async()=>{
    const browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE});
    try {
        for(const mode of ['prepare','repair','blocked']) {
            const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'});
            await setupLauncherDemo(page);
            await page.addInitScript(mode=>{
                window.preparationCalls=[];
                let repaired=false;
                const original=window.electronAPI.invoke;
                const invoke=async(command,args={})=>{
                    window.preparationCalls.push(command);
                    if(command==='doctor_instance_readiness')return {ready:repaired,repairable:mode!=='blocked',preparationRequired:mode==='prepare',requiredJava:21,allocatedRamMb:4096,recommendedRamMb:4096,gpuVendor:'Fixture',gpuDriver:'Fixture',blockers:repaired?[]:['Arquivos do Minecraft ausentes'],warnings:[],conflicts:{hasConflicts:false,conflicts:[],duplicates:[]}};
                    if(command==='doctor_repair_all'){repaired=true;return {repairedMods:0,warnings:[]};}
                    if(command==='launch_game')return {pid:4242};
                    if(command==='game_launch_status')return {running:true,windowReady:true};
                    return original(command,args);
                };
                window.electronAPI.invoke=invoke;
                window.__TAURI_INTERNALS__.invoke=invoke;
            },mode);
            await page.goto('http://127.0.0.1:1420/instances/demo-fo');
            await page.getByRole('button',{name:'JOGAR MINECRAFT',exact:true}).click();
            if(mode==='blocked')await page.getByText(/Arquivos do Minecraft ausentes/).waitFor();
            else {
                try {await page.waitForFunction(()=>window.preparationCalls.includes('launch_game'));}
                catch(error) {console.error(JSON.stringify({mode,calls:await page.evaluate(()=>window.preparationCalls),text:(await page.locator('main').innerText()).slice(-1400)}));throw error;}
            }
            const calls=await page.evaluate(()=>window.preparationCalls);
            assert.equal(calls.filter(command=>command==='doctor_repair_all').length,mode==='repair'?1:0);
            assert.equal(calls.includes('launch_game'),mode!=='blocked');
            await page.close();
        }
        console.log('First preparation uses the launch pipeline; explicit repair remains available; blockers still prevent launch.');
    } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1;});
