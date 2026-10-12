const {chromium}=require('playwright-core');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const {spawn}=require('node:child_process');
const server=spawn(process.execPath,['scripts/preview-website.mjs'],{env:{...process.env,LUXMC_SITE_PORT:'8794'},stdio:'ignore',windowsHide:true});
(async()=>{
    let browser;
    try {
        for(let attempt=0;attempt<80;attempt++){try{if((await fetch('http://127.0.0.1:8794/')).ok)break;}catch{}await new Promise(resolve=>setTimeout(resolve,100));}
        browser=await chromium.launch({headless:true,executablePath:process.env.LUXMC_CHROMIUM_EXECUTABLE || 'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
        const page=await browser.newPage({viewport:{width:1440,height:1000},reducedMotion:'reduce'}),errors=[];
        page.on('pageerror',error=>errors.push(String(error)));
        let user={id:'test-profile',username:'Player',displayName:'Player',avatarUrl:null,skinUrl:'/api/appearance/Player?asset=skin',canManageAccount:true,membership:{plan:'pro',expiresAt:1900000000}};
        await page.route('**/api/session*',async route=>{if(route.request().method()==='POST'){assert.equal(route.request().headers()['x-luxmc-csrf'],'test-csrf');user=null;await route.fulfill({contentType:'application/json',body:'{"ok":true}'});}else await route.fulfill({contentType:'application/json',body:JSON.stringify({user,csrfToken:'test-csrf'})});});
        await page.route('**/api/appearance/**',route=>route.fulfill({contentType:'image/png',body:fs.readFileSync('static/steve.png')}));
        await page.route('**/api/commerce/status',route=>route.fulfill({contentType:'application/json',body:JSON.stringify({identity:{id:'test-profile',username:'Player',owner:false},csrfToken:'test-csrf',membership:user.membership,checkout:{enabled:false},orders:[],settings:{preferences:{},presets:[],revision:0}})}));
        await page.route('**/api/latest-release*',route=>route.fulfill({contentType:'application/json',body:JSON.stringify({tag_name:'v3.9.0',assets:[]})}));
        await page.goto('http://127.0.0.1:8794/');
        await page.locator('.site-account-menu summary').waitFor();
        await page.waitForFunction(()=>document.querySelector('.site-account-menu img')?.naturalWidth===64);
        assert.equal(await page.locator('.site-account-menu img').getAttribute('data-skin-head'),'true');
        await page.locator('.site-account-menu summary').click();await page.getByRole('link',{name:'Meu plano',exact:true}).click();
        await page.getByRole('heading',{name:'Player · Luxmc Pro',exact:true}).waitFor();
        assert.equal(await page.locator('.plans-emblem').count(),0);assert.equal(await page.locator('[data-plan=pro]').isDisabled(),true);
        fs.mkdirSync('docs/validation/v3.9/identity',{recursive:true});
        await page.screenshot({path:'docs/validation/v3.9/identity/planos-desktop.png',fullPage:true});
        user.avatarUrl='/assets/avatar-steve.png';user.skinUrl=null;await page.evaluate(()=>window.dispatchEvent(new Event('luxmc-session-changed')));
        await page.waitForFunction(()=>document.querySelector('.site-account-menu img')?.dataset.skinHead==='false');
        await page.setViewportSize({width:390,height:844});
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
        await page.screenshot({path:'docs/validation/v3.9/identity/planos-mobile.png',fullPage:true});
        await page.goto('http://127.0.0.1:8794/');await page.locator('.site-account-menu summary').waitFor();
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
        await page.screenshot({path:'docs/validation/v3.9/identity/navigation-mobile.png'});
        await page.locator('.site-account-menu summary').click();await page.getByRole('button',{name:'Sair do site',exact:true}).click();await page.locator('[data-site-account] > a').waitFor();
        assert.deepEqual(errors,[]);console.log('Skin head cropping, launcher photo, account-bound plan link, logout, desktop and mobile verified.');
    } finally {if(browser)await browser.close();server.kill();}
})().catch(error=>{console.error(error);process.exitCode=1;});
