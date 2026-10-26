interface SiteUser { id:string; username:string; displayName:string; avatarUrl:string|null; skinUrl:string|null; canManageAccount:boolean; membership:{plan:string;expiresAt:number|null} }
let sessionGeneration = 0;
let sessionCsrf = '';
let launcherConnection: Promise<boolean> | null = null;
let lastLauncherAttempt = 0;
const launcherFragment = new URLSearchParams(location.hash.slice(1));
const launcherTicket = launcherFragment.get('launcher_ticket');
if (launcherTicket) {
    const section = launcherFragment.get('section');
    history.replaceState(null, '', location.pathname + location.search + (section && ['mods','changelog'].includes(section) ? `#${section}` : ''));
}
async function connectLauncher(explicit = false, suppliedTicket: string | null = null): Promise<boolean> {
    if (launcherConnection) return launcherConnection;
    try { if (!explicit && localStorage.getItem('luxmc.launcher-autologin') === 'off') return false; } catch {}
    if (!explicit && !suppliedTicket && Date.now() - lastLauncherAttempt < 30000) return false;
    lastLauncherAttempt = Date.now();
    launcherConnection = (async () => {
        try {
            let ticket = suppliedTicket;
            if (!ticket) {
                const local = await fetch('http://127.0.0.1:38457/v1/session', {method:'POST', credentials:'omit', mode:'cors', headers:{'Content-Type':'application/json','X-Luxmc-Site':'1'}, body:'{}', cache:'no-store', signal:AbortSignal.timeout(4000)});
                if (!local.ok) return false;
                ticket = (await local.json()).ticket;
            }
            if (typeof ticket !== 'string' || !/^[a-f0-9]{64}$/.test(ticket)) return false;
            const response = await fetch('/api/commerce/exchange', {method:'POST',credentials:'same-origin',headers:{'Content-Type':'application/json'},body:JSON.stringify({ticket}),signal:AbortSignal.timeout(10000)});
            if (!response.ok) return false;
            try { localStorage.removeItem('luxmc.launcher-autologin'); } catch {}
            window.dispatchEvent(new Event('luxmc-session-changed'));
            return true;
        } catch { return false; }
    })().finally(() => { launcherConnection = null; });
    return launcherConnection;
}
window.LuxLauncherSession = {connect: () => connectLauncher(true)};
const sessionSlots = () => document.querySelectorAll<HTMLElement>('[data-site-account]');
function sessionNode<K extends keyof HTMLElementTagNameMap>(tag:K,text='',className='') { const result=document.createElement(tag);result.textContent=text;result.className=className;return result; }
function accountLink(text:string,path:string) { const link=sessionNode('a',text);link.href=path;return link; }
function safeProfileImage(value:string|null) { if (!value) return false; if (/^data:image\/(png|jpeg|webp|gif);base64,/.test(value)) return value.length <= 960000; try { const url=new URL(value,location.origin);return ['https:','http:'].includes(url.protocol) && !url.username && !url.password && (url.protocol==='https:' || url.origin===location.origin); } catch { return false; } }
async function skinHead(path:string) {
    return new Promise<string>((resolve,reject) => {
        const image=new Image();
        image.onload=()=>{ try { if (image.naturalWidth!==64 || ![32,64].includes(image.naturalHeight)) throw Error('Invalid skin'); const canvas=document.createElement('canvas');canvas.width=64;canvas.height=64;const context=canvas.getContext('2d');if(!context)throw Error('Canvas unavailable');context.imageSmoothingEnabled=false;context.drawImage(image,8,8,8,8,0,0,64,64);context.drawImage(image,40,8,8,8,0,0,64,64);resolve(canvas.toDataURL('image/png')); } catch(error) {reject(error);} };
        image.onerror=()=>reject(Error('Skin unavailable'));image.src=path;
    });
}
function renderSiteSession(user:SiteUser|null,avatar='/assets/avatar-steve.png',isHead=true) {
    for (const slot of sessionSlots()) {
        slot.replaceChildren();slot.dataset.accountId=user?.id || '';
        if (!user) {const link=accountLink('Entrar','/conta');link.dataset.i18n='nav.login';const text=window.LuxI18n?.t('nav.login');if(typeof text==='string'&&text!=='nav.login')link.textContent=text;slot.append(link);continue;}
        const menu=sessionNode('details','','site-account-menu');
        const trigger=sessionNode('summary');trigger.setAttribute('aria-label',`Conta de ${user.displayName}`);
        const image=sessionNode('img');image.src=avatar;image.alt='';image.width=36;image.height=36;image.dataset.skinHead=String(isHead);image.onerror=()=>{image.onerror=null;image.src='/assets/avatar-steve.png';image.dataset.skinHead='true';};
        trigger.append(image,sessionNode('span',user.displayName,'site-account-name'),sessionNode('span','⌄','site-account-chevron'));
        const content=sessionNode('nav','','site-account-dropdown');content.setAttribute('aria-label','Sua conta');
        content.append(sessionNode('strong',user.displayName),sessionNode('p',`Luxmc ${user.membership.plan==='pro'?'Pro':user.membership.plan==='plus'?'Plus':'Free'}`),translatedAccountLink('nav.plan','Meu plano','/planos'));
        if(user.canManageAccount)content.append(translatedAccountLink('nav.account','Minha conta','/conta'));
        const logout=sessionNode('button','Sair do site');logout.type='button';
        logout.onclick=async()=>{logout.disabled=true;try { const response=await fetch('/api/session',{method:'POST',credentials:'same-origin',headers:{'Content-Type':'application/json','X-Luxmc-CSRF':sessionCsrf},body:'{}',signal:AbortSignal.timeout(10000)});if(!response.ok)throw Error('Não foi possível sair. Tente novamente.');try {localStorage.setItem('luxmc.launcher-autologin','off');}catch{}sessionGeneration++;sessionCsrf='';renderSiteSession(null);window.dispatchEvent(new Event('luxmc-session-changed')); } catch {logout.textContent='Tentar sair novamente';logout.disabled=false;} };
        content.append(logout);menu.append(trigger,content);slot.append(menu);
    }
}
async function refreshSiteSession() {
    const run=++sessionGeneration;
    try {
        const response=await fetch('/api/session',{credentials:'same-origin',signal:AbortSignal.timeout(10000)});
        if(!response.ok) return;
        const state=await response.json() as {user:SiteUser|null;csrfToken?:string};
        if(run!==sessionGeneration)return;
        sessionCsrf=state.csrfToken || '';
        const user=state.user;
        if (!user && await connectLauncher()) return;
        if(run!==sessionGeneration)return;
        let avatar='/assets/avatar-steve.png',isHead=true;
        if(user && safeProfileImage(user.avatarUrl)) {avatar=user.avatarUrl!;isHead=false;}
        else if(user?.skinUrl && user.skinUrl.startsWith('/api/appearance/')) {try {avatar=await skinHead(user.skinUrl);}catch{}}
        if(run!==sessionGeneration)return;
        renderSiteSession(user,avatar,isHead);
        window.dispatchEvent(new CustomEvent('luxmc-session-ready',{detail:{user,avatar}}));
    } catch {}
}
window.addEventListener('luxmc-session-changed',()=>void refreshSiteSession());
window.addEventListener('pageshow',()=>void refreshSiteSession());
document.addEventListener('visibilitychange',()=>{if(!document.hidden)void refreshSiteSession();});
document.addEventListener('pointerdown',event=>{for(const menu of document.querySelectorAll<HTMLDetailsElement>('.site-account-menu[open]'))if(!menu.contains(event.target as Node))menu.open=false;});
document.addEventListener('keydown',event=>{if(event.key==='Escape')for(const menu of document.querySelectorAll<HTMLDetailsElement>('.site-account-menu[open]')){menu.open=false;menu.querySelector('summary')?.focus();}});
setInterval(()=>{if(!document.hidden && !document.querySelector('.site-account-menu[open]'))void refreshSiteSession();},60000);
if (launcherTicket) void connectLauncher(true, launcherTicket).then(() => refreshSiteSession());
else if(sessionSlots().length)void refreshSiteSession();

function translatedAccountLink(key:string,fallback:string,url:string) { const link=accountLink(fallback,url);link.dataset.i18n=key;const text=window.LuxI18n?.t(key);if(typeof text==='string'&&text!==key)link.textContent=text;return link; }
