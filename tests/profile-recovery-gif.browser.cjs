const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const { setupLauncherDemo } = require('../scripts/launcher-video-fixture.cjs');

(async () => {
    const browser = await chromium.launch({ headless:true, executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE });
    try {
        for (const plan of ['free','plus','pro']) {
            const page = await browser.newPage();
            await setupLauncherDemo(page);
            await page.addInitScript(plan => {
                window.profileReads = 0;
                const invoke = window.electronAPI.invoke;
                window.electronAPI.invoke = async (command,args={}) => {
                    if (command === 'social_request' && args.request.action === 'profile_get') {
                        if (!args.request.targetId && ++window.profileReads === 1) throw new Error('Falha temporária do serviço');
                        return {profile:{id:args.request.targetId || window.launcherDemo.state.account.uuid,username:'LuxPlayer',displayName:args.request.targetId ? 'Perfil amigo' : 'Nome recuperado',status:'',description:'',banner:'',portrait:'',packs:[],collections:[],membership:{plan,expiresAt:1893456000}}};
                    }
                    return invoke(command,args);
                };
            },plan);
            await page.goto('http://127.0.0.1:1420/friends');
            await page.locator('main').getByText('Nome recuperado',{exact:true}).waitFor({timeout:30000});
            assert.ok(await page.evaluate(()=>window.profileReads >= 2));
            await page.getByRole('button',{name:'Editar meu perfil',exact:true}).click();
            const dialog=page.getByRole('dialog');
            await dialog.getByLabel('Nome de exibição',{exact:true}).waitFor();
            await dialog.getByRole('tab',{name:'Aparência',exact:true}).click();
            const input=dialog.getByLabel('Selecionar avatar do perfil',{exact:true});
            assert.equal((await input.getAttribute('accept')).includes('image/gif'),plan!=='free');
            const gif=Buffer.from('R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7','base64');
            await input.setInputFiles({name:'avatar.gif',mimeType:'image/gif',buffer:gif});
            if (plan==='free') await dialog.getByText('GIFs no perfil requerem Luxmc Plus ou Pro ativo.',{exact:true}).last().waitFor();
            else await dialog.getByText('GIF',{exact:true}).waitFor();
            await page.close();
        }
        console.log('PASS recuperação automática do perfil e GIFs restritos a Plus/Pro na interface real');
    } finally { await browser.close(); }
})().catch(error=>{console.error(error.message);process.exitCode=1;});
