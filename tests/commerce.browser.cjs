const { chromium } = require('playwright-core');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { spawn } = require('node:child_process');
const server = spawn(process.execPath, ['scripts/preview-website.mjs'], {env:{...process.env,LUXMC_SITE_PORT:'8793'},stdio:'ignore',windowsHide:true});
const jsQR=require('jsqr');
const {PNG}=require('pngjs');
const now = Math.floor(Date.now()/1000);
const id = '22222222-2222-4222-8222-222222222222';
const ownerId = '11111111-1111-4111-8111-111111111111';
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aWQAAAABJRU5ErkJggg==','base64');
let orders = [];
let checkoutEnabled = true;
let membership = {plan:'free',expiresAt:null,revision:0,collections:6,presets:0};
let ownerMembership = {plan:'free',expiresAt:null,revision:0,collections:6,presets:0};
let preferences = {preferences:{},presets:[],revision:0};
const apiCalls = [];
let needsUnlock = true;
async function mocks(page, owner, denied = false) {
    await page.route('**/api/commerce/**', async route => {
        const action = route.request().url().split('/').pop(); const body = route.request().postDataJSON(); apiCalls.push(action);
        let result = {ok:true}; let status = 200;
        if (denied) { status = 401; result = {error:'Entre novamente.'}; }
        else if (action === 'status') result = {identity:{id:owner?ownerId:id,username:owner?'Owner':'Player',owner},csrfToken:'d'.repeat(64),security:owner?{twoFactor:false,verifiedUntil:now+300,expiresAt:now+900}:null,membership:owner?ownerMembership:membership,catalog:{},checkout:{enabled:checkoutEnabled,bank:'Nubank',pixKey:'test-only-pix-key'},settings:preferences,orders};
        else if (action === 'owner_unlock') {assert.equal(body.password,'test-unlock-password-123');needsUnlock=false;result={ok:true,csrfToken:'d'.repeat(64)};}
        else if (owner && needsUnlock && action.startsWith('owner_')) {status=428;result={error:'Confirme sua senha para continuar.'};}
        else if (action === 'create') { const {pixPayload,orderTxid}=await import('../website/lib/pix.js'); const order={id:'44444444-4444-4444-8444-444444444444',socialId:id,username:'Player',displayName:'<script>unsafe</script>',plan:body.plan,amount:body.plan==='pro'?1990:990,bank:'Nubank',pixKey:'test-only-pix-key',days:30,status:'awaiting_receipt',version:0,createdAt:now,updatedAt:now,note:'',receiptHash:null,receiptMime:null,receiptUploadedAt:null};order.pixCode=pixPayload({key:'test@example.test',amount:order.amount,txid:orderTxid(order.id)});order.pixReference=orderTxid(order.id);orders=[order];result={order}; }
        else if (action === 'owner_queue') result = {orders:orders.filter(order=>body.status==='all'||order.status===body.status),total:orders.length,stats:{pending:orders.filter(order=>order.status==='pending').length,approved:orders.filter(order=>order.status==='approved').length,approvedAmount:orders.filter(order=>order.status==='approved').reduce((total,order)=>total+order.amount,0)},config:{enabled:true,bank:'Nubank',pixKey:'test-only-pix-key'},audit:[]};
        else if (action === 'owner_review') { assert.equal(body.creditConfirmed,true);assert.equal(body.receivedAmount,orders[0].amount); assert.equal(body.receiptHash,'f'.repeat(64));orders[0].status='approved';membership={plan:orders[0].plan,expiresAt:now+30*86400,revision:1,collections:orders[0].plan==='pro'?24:12,presets:orders[0].plan==='pro'?20:6};result={ok:true,membership}; }
        else if (action === 'preferences') { assert.equal(body.revision,preferences.revision);preferences={preferences:body.preferences,presets:body.presets,revision:preferences.revision+1};result={ok:true,revision:preferences.revision}; }
        else if (action === 'owner_find') result={users:body.query===ownerId?[{id:ownerId,username:'Owner',friendCode:'Owner#11111111',display_name:'Owner',plan:ownerMembership.plan,revision:ownerMembership.revision}]:[{id,username:'Player',friendCode:'Player#22222222',display_name:'Player',plan:membership.plan,revision:membership.revision},{id:'33333333-3333-4333-8333-333333333333',username:'Player',friendCode:'Player#33333333',display_name:'Player',plan:'free',revision:0}]};
        else if (action === 'owner_grant') {const own=body.targetId===ownerId;assert.equal(body.confirmationCode,own?'Owner#11111111':'Player#22222222');assert.equal(body.revision,own?ownerMembership.revision:membership.revision);const next={plan:body.plan,expiresAt:now+body.days*86400,revision:body.revision+1,collections:body.plan==='pro'?24:12,presets:body.plan==='pro'?20:6};if(own)ownerMembership=next;else membership=next;result={ok:true,identity:{id:body.targetId,friendCode:own?'Owner#11111111':'Player#22222222'},membership:next};}
        if(owner&&action.startsWith('owner_')&&!['owner_login'].includes(action))assert.equal(route.request().headers()['x-luxmc-csrf'],'d'.repeat(64));
        await route.fulfill({status,contentType:'application/json',body:JSON.stringify(result)});
    });
    await page.route('**/api/receipts/**', async route => {
        if(route.request().method()==='POST') {orders[0].status='pending';orders[0].version++;orders[0].receiptHash='f'.repeat(64);orders[0].receiptMime='image/png';orders[0].receiptUploadedAt=now;await route.fulfill({contentType:'application/json',body:'{"ok":true}'});}
        else await route.fulfill({contentType:'image/png',body:png});
    });
}
(async()=>{
    let browser;
    try {
        for(let attempt=0;attempt<80;attempt++) {try{if((await fetch('http://127.0.0.1:8793/planos.html')).ok)break;}catch{} await new Promise(resolve=>setTimeout(resolve,100));}
        browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
        fs.mkdirSync('docs/validation/v3.9/plans',{recursive:true});
        const buyer=await browser.newPage({viewport:{width:1440,height:1100},reducedMotion:'reduce'}); const errors=[];buyer.on('pageerror',error=>errors.push(String(error)));await mocks(buyer,false);
        await buyer.goto('http://127.0.0.1:8793/planos.html');await buyer.getByRole('heading',{name:'Player · Luxmc Free'}).waitFor();await buyer.screenshot({path:'docs/validation/v3.9/plans/planos-desktop.png',fullPage:true});
        await buyer.getByRole('button',{name:'Escolher Plus'}).click();await buyer.getByLabel('Enviar comprovante',{exact:false}).setInputFiles({name:'comprovante.png',mimeType:'image/png',buffer:png});const qrPNG=PNG.sync.read(await buyer.locator('.pix-qr-frame canvas').screenshot({path:'docs/validation/v3.9/plans/qr-scan.png'}));assert.equal(jsQR(new Uint8ClampedArray(qrPNG.data),qrPNG.width,qrPNG.height).data,orders[0].pixCode);await buyer.screenshot({path:'docs/validation/v3.9/plans/checkout-pix-desktop.png',fullPage:true});await buyer.setViewportSize({width:390,height:844});assert.equal(await buyer.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);await buyer.screenshot({path:'docs/validation/v3.9/plans/checkout-pix-mobile.png',fullPage:true});await buyer.setViewportSize({width:1440,height:1100});await buyer.getByRole('button',{name:'Enviar para verificação'}).click();await buyer.getByText('Esperando verificação',{exact:true}).waitFor();
        const admin=await browser.newPage({viewport:{width:1440,height:1050},reducedMotion:'reduce'});admin.on('pageerror',error=>errors.push(String(error)));admin.on('dialog',dialog=>dialog.accept());await mocks(admin,true);await admin.goto('http://127.0.0.1:8793/painel-luxmc.html');await admin.locator('#owner-unlock').waitFor({state:'visible'});await admin.locator('#unlock-password').fill('test-unlock-password-123');await admin.getByRole('button',{name:'Confirmar acesso',exact:true}).click();await admin.getByRole('button',{name:'Ver comprovante'}).click();assert.equal(await admin.getByRole('button',{name:'Aprovar e liberar plano'}).isDisabled(),true);await admin.getByLabel('Valor recebido no banco (R$)').fill('9,90');await admin.getByLabel('Identificador da transação Pix (EndToEndId)').fill('E12345678202009091221abcdef12345');await admin.getByLabel('Observação para o comprador / motivo da rejeição').fill('Crédito de exemplo conferido no teste local');await admin.getByLabel('Conferi o valor e o crédito desta transação no extrato do banco.').check();await admin.screenshot({path:'docs/validation/v3.9/plans/painel-revisao.png',fullPage:true});
        assert.equal(await admin.locator('script').count(),1); await admin.getByRole('button',{name:'Aprovar e liberar plano'}).click();await admin.getByText('Plano liberado e disponível para sincronização no launcher.').waitFor();
        await buyer.reload();await buyer.getByRole('heading',{name:'Player · Luxmc Plus'}).waitFor();assert.equal(await buyer.getByRole('link',{name:'Meu recibo Luxmc'}).getAttribute('href'),'/api/purchase-receipts/'+orders[0].id);
        assert.equal(await buyer.locator('#orders,#preferences,#profile-customization').count(),0);
        assert.equal(await buyer.getByRole('button',{name:'Seu plano ativo',exact:true}).isDisabled(),true);
        assert.equal(await buyer.locator('#checkout').isVisible(),false);
        await admin.getByRole('button',{name:'Planos',exact:true}).click();await admin.getByLabel('Usuário, código de amizade ou ID completo').fill('Player');await admin.getByRole('button',{name:'Buscar conta'}).click();await admin.getByRole('button',{name:'Selecionar Player#22222222',exact:true}).waitFor();assert.equal(await admin.getByRole('button',{name:/Selecionar Player#/}).count(),2);await admin.getByRole('button',{name:'Selecionar Player#22222222',exact:true}).click();await admin.getByRole('heading',{name:'Alterar plano de Player'}).waitFor();await admin.getByLabel('Novo plano').selectOption('pro');await admin.getByLabel('Cole o código que o jogador enviou pelo launcher').fill('Player#33333333');await admin.getByRole('button',{name:'Confirmar atribuição'}).click();await admin.getByText('O código informado não corresponde à conta selecionada.',{exact:false}).waitFor();assert.equal(apiCalls.filter(action=>action==='owner_grant').length,0);
        await admin.getByLabel('Cole o código que o jogador enviou pelo launcher').fill('Player#22222222');await admin.getByRole('button',{name:'Confirmar atribuição'}).click();await admin.locator('#message .notice').waitFor();assert.equal(membership.plan,'pro');await admin.getByRole('heading',{name:'Owner · Luxmc Free'}).waitFor();
        await admin.getByRole('button',{name:'Alterar meu próprio plano',exact:true}).click();await admin.getByLabel('Novo plano').selectOption('pro');await admin.getByRole('button',{name:'Confirmar atribuição'}).click();await admin.getByRole('heading',{name:'Owner · Luxmc Pro'}).waitFor();assert.equal(ownerMembership.plan,'pro');await admin.screenshot({path:'docs/validation/v3.9/plans/painel-identidade-teste.png',fullPage:true});
        await admin.getByRole('button',{name:'Segurança',exact:true}).click();await admin.getByRole('heading',{name:'Proteção do acesso'}).waitFor();await admin.getByRole('button',{name:'Configurar duas etapas'}).waitFor();await admin.screenshot({path:'docs/validation/v3.9/plans/painel-seguranca-teste.png',fullPage:true});await admin.setViewportSize({width:390,height:844});assert.equal(await admin.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);await admin.getByRole('button',{name:'Pedidos',exact:true}).click();await admin.screenshot({path:'docs/validation/v3.9/plans/painel-mobile-teste.png',fullPage:true});
        const mobile=await browser.newPage({viewport:{width:390,height:844},reducedMotion:'reduce'});await mocks(mobile,false);await mobile.goto('http://127.0.0.1:8793/planos.html');await mobile.getByRole('heading',{name:'Player · Luxmc Pro'}).waitFor();assert.equal(await mobile.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);await mobile.screenshot({path:'docs/validation/v3.9/plans/planos-mobile.png',fullPage:true});
        checkoutEnabled=false;await buyer.reload();await buyer.getByRole('heading',{name:'Player · Luxmc Pro'}).waitFor();
        await buyer.getByText('Ativo até',{exact:false}).waitFor();
        assert.equal(await buyer.locator('[data-plan=pro]').textContent(),'Seu plano ativo');
        assert.equal(await buyer.locator('[data-plan=pro]').isDisabled(),true);
        assert.equal(await buyer.locator('[data-plan=plus]').textContent(),'Novos pedidos pausados');
        assert.equal(await buyer.locator('[data-plan=plus]').isDisabled(),true);
        assert.equal(await buyer.locator('#identity a[href="/painel-luxmc"]').count(),0);
        assert.equal(await buyer.locator('#message').textContent(),'');
        await buyer.getByRole('heading',{level:1}).click();await buyer.screenshot({path:'docs/validation/v3.9/plans/planos-pro-paused.png',fullPage:true});
        const creates=apiCalls.filter(action=>action==='create').length;
        await buyer.locator('[data-plan=pro]').evaluate(button=>button.click());
        await buyer.waitForTimeout(50);assert.equal(apiCalls.filter(action=>action==='create').length,creates);
        orders=[{...orders[0],plan:'plus',status:'pending'}];await buyer.reload();await buyer.getByRole('button',{name:'Ver pedido',exact:true}).click();
        await buyer.getByText('Esperando verificação',{exact:true}).waitFor();
        await buyer.getByRole('button',{name:'Fechar pedido',exact:true}).click();
        assert.equal(await buyer.locator('#checkout').isVisible(),false);
        const denied=await browser.newPage();await mocks(denied,false,true);await denied.goto('http://127.0.0.1:8793/painel-luxmc.html');await denied.getByRole('heading',{name:'Acesso reservado ao dono'}).waitFor();assert.equal(await denied.locator('#owner-view').isVisible(),false);assert.equal(await denied.locator('#order-list').textContent(),'');
        assert.deepEqual(errors,[]);console.log('Browser: checkout → comprovante → revisão → Plus, plano Pro ativo sem compra duplicada, pedidos pausados sem erro, comprovante acessível em janela, nomes iguais, código incorreto bloqueado, Pro da própria conta, acesso bloqueado e layout móvel verificados.');
    } finally {if(browser)await browser.close();server.kill();}
})().catch(error=>{console.error(error);process.exitCode=1;});
