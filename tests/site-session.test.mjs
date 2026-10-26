import test from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import {readFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {onRequest as session} from '../website/functions/api/session.js';
import {onRequest as account} from '../website/functions/api/account/[action].js';
import {onRequest as commerce} from '../website/functions/api/commerce/[action].js';

const alice='22222222-2222-4222-8222-222222222222',bob='33333333-3333-4333-8333-333333333333';
const sha=value=>createHash('sha256').update(value).digest('hex');
const png='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aWQAAAABJRU5ErkJggg==';
function fixture(t) {
    const sqlite=new DatabaseSync(':memory:');
    for(const file of readdirSync(new URL('../website/migrations/',import.meta.url)).filter(name=>name.endsWith('.sql')).sort())sqlite.exec(readFileSync(new URL(`../website/migrations/${file}`,import.meta.url),'utf8'));
    for(const [id,name] of [[alice,'Alice'],[bob,'Bob']])sqlite.prepare('INSERT INTO social_users(id,username,token_hash) VALUES (?,?,?)').run(id,name,sha(id===alice?'a'.repeat(64):'b'.repeat(64)));
    const db={prepare(sql){const statement=sqlite.prepare(sql);const bound=(values=[])=>({bind:(...next)=>bound(next),async first(){return statement.get(...values)||null;},async all(){return{results:statement.all(...values)};},async run(){return{meta:statement.run(...values)};},execute(){return{meta:statement.run(...values)};}});return bound();},async batch(statements){sqlite.exec('BEGIN');try{const result=statements.map(s=>s.execute());sqlite.exec('COMMIT');return result;}catch(error){sqlite.exec('ROLLBACK');throw error;}}};
    const env={SOCIAL_DB:db,OWNER_SOCIAL_ID:alice,AUTH_PEPPER:'test-only-pepper-not-for-production-123456789'};
    const now=Math.floor(Date.now()/1000),jar=new Map();
    const scoped=(id,token='d'.repeat(64))=>{sqlite.prepare('INSERT INTO lux_commerce_sessions(token_hash,social_id,expires_at,created_at,last_activity_at) VALUES (?,?,?,?,?)').run(sha(token),id,now+3600,now,now);jar.set('__Host-luxmc-commerce',token);};
    async function call(handler=session,{method='GET',path='/api/session',body,headers={}}={}) {
        const request=new Request(`https://luxmc.test${path}`,{method,headers:{'Content-Type':'application/json',Cookie:[...jar].map(([k,v])=>`${k}=${v}`).join('; '),...headers},body:body===undefined?undefined:JSON.stringify(body)});
        const response=await handler({request,env,params:{action:path.split('/').pop()},waitUntil(promise){void promise.catch(()=>{});}});
        for(const cookie of response.headers.getSetCookie()){const [name,value]=cookie.split(';')[0].split('=');if(cookie.includes('Max-Age=0'))jar.delete(name);else jar.set(name,value);}
        return response;
    }
    t.after(()=>sqlite.close());return{sqlite,jar,scoped,call,now};
}
test('site identity comes from the session and never from a requested profile or username',async t=>{
    const f=fixture(t);assert.deepEqual(await(await f.call()).json(),{user:null});f.scoped(alice);
    f.sqlite.prepare('INSERT INTO social_public_profiles(user_id,portrait,display_name,updated_at) VALUES (?,?,?,?)').run(alice,`data:image/png;base64,${png}`,'Creator',f.now);
    f.sqlite.prepare('INSERT INTO lux_memberships(social_id,plan,expires_at) VALUES (?,?,?)').run(alice,'pro',f.now+3600);
    const response=await f.call(session,{path:`/api/session?targetId=${bob}`});const state=await response.json();
    assert.equal(state.user.id,alice);assert.equal(state.user.displayName,'Creator');assert.equal(state.user.membership.plan,'pro');assert.equal(state.user.canManageAccount,false);assert.match(state.user.avatarUrl,/^\/api\/session\?asset=avatar/);assert.doesNotMatch(JSON.stringify(state),/token_hash|pixKey|password|recovery/);assert.match(response.headers.get('cache-control'),/no-store/);
    const image=await f.call(session,{path:state.user.avatarUrl});assert.equal(image.headers.get('content-type'),'image/png');assert.deepEqual(Buffer.from(await image.arrayBuffer()),Buffer.from(png,'base64'));
    assert.equal((await f.call(session,{path:`/api/session?asset=avatar&profile=${bob}`})).status,403);
});
test('a new web login replaces a previous launcher identity and purchases belong to the new account',async t=>{
    const f=fixture(t);f.scoped(alice);t.mock.method(globalThis,'fetch',async()=>new Response(null,{status:404}));
    const registered=await f.call(account,{method:'POST',path:'/api/account/register',body:{username:'NewPlayer',password:'Test-password-123'}});assert.equal(registered.status,200);
    const user=(await registered.json()).account;assert.equal(f.jar.has('__Host-luxmc-commerce'),false);
    const state=await(await f.call()).json();assert.equal(state.user.id,user.socialId);assert.equal(state.user.canManageAccount,true);
    f.sqlite.prepare("UPDATE lux_commerce_config SET enabled=1,pix_key='test@example.test' WHERE id=1").run();
    const created=await f.call(commerce,{method:'POST',path:'/api/commerce/create',body:{plan:'plus',targetId:alice},headers:{'X-Luxmc-CSRF':state.csrfToken}});assert.equal(created.status,200);assert.equal((await created.json()).order.socialId,user.socialId);
});
test('a launcher ticket replaces a browser login without selecting accounts by nickname',async t=>{
    const f=fixture(t);t.mock.method(globalThis,'fetch',async()=>new Response(null,{status:404}));
    await f.call(account,{method:'POST',path:'/api/account/register',body:{username:'WebPlayer',password:'Test-password-123'}});
    const ticket='f'.repeat(64);f.sqlite.prepare('INSERT INTO lux_commerce_tickets(token_hash,social_id,expires_at) VALUES (?,?,?)').run(sha(ticket),bob,f.now+60);
    const response=await f.call(commerce,{method:'POST',path:'/api/commerce/exchange',body:{ticket,targetId:alice}});assert.equal(response.status,200);assert.equal(f.jar.has('__Host-luxmc'),false);assert.equal((await(await f.call()).json()).user.id,bob);
});
test('logout requires CSRF, expires the browser session and keeps profiles and other sessions',async t=>{
    const f=fixture(t);f.scoped(alice);f.sqlite.prepare('INSERT INTO lux_commerce_sessions(token_hash,social_id,expires_at,created_at,last_activity_at) VALUES (?,?,?,?,?)').run(sha('e'.repeat(64)),bob,f.now+3600,f.now,f.now);
    assert.equal((await f.call(session,{method:'POST',body:{}})).status,403);
    const state=await(await f.call()).json();assert.equal((await f.call(session,{method:'POST',body:{},headers:{'X-Luxmc-CSRF':state.csrfToken}})).status,200);
    assert.deepEqual(await(await f.call()).json(),{user:null});assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS count FROM social_users').get().count,2);assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS count FROM lux_commerce_sessions').get().count,1);
});

test('launcher transfer opens the existing account dashboard and keeps tickets single-use',async t=>{
    const f=fixture(t), ticket='e'.repeat(64), id='44444444-4444-4444-8444-444444444444';
    f.sqlite.prepare("INSERT INTO lux_accounts(id,username,password_hash,password_salt,recovery_hash,social_id,created_at,auth_provider) VALUES (?,?,?,?,?,?,?,'microsoft')").run(id,'Bob','not-a-password','salt','recovery',bob,f.now);
    f.sqlite.prepare('INSERT INTO lux_microsoft_accounts(minecraft_uuid,account_id,created_at) VALUES (?,?,?)').run('1234567890abcdef1234567890abcdef',id,f.now);
    f.sqlite.prepare('INSERT INTO lux_commerce_tickets(token_hash,social_id,expires_at) VALUES (?,?,?)').run(sha(ticket),bob,f.now+60);
    const transfer=await f.call(commerce,{method:'POST',path:'/api/commerce/exchange',body:{ticket}});
    assert.equal(transfer.status,200);
    assert.ok(transfer.headers.getSetCookie().some(value=>value.startsWith('__Host-luxmc=')&&value.includes('HttpOnly')&&value.includes('Secure')));
    const state=await(await f.call()).json();
    assert.equal(state.user.id,bob);
    assert.equal(state.user.canManageAccount,true);
    assert.match(state.user.avatarUrl,/1234567890abcdef1234567890abcdef/);
    assert.equal((await f.call(account,{method:'POST',path:'/api/account/me',body:{}})).status,200);
    assert.equal((await f.call(commerce,{method:'POST',path:'/api/commerce/exchange',body:{ticket}})).status,401);
    assert.doesNotMatch(JSON.stringify(state),/recovery|password|token_hash/);
});
