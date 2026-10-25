import test from 'node:test';
import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { exportJWK, generateKeyPair, SignJWT } from 'jose';
import { onRequest } from '../website/functions/api/microsoft/[action].js';
import { onRequest as accountHandler } from '../website/functions/api/account/[action].js';
import { authenticate } from '../website/lib/accounts.js';
import { microsoftNonce } from '../website/lib/microsoft-identity.js';

const origin = 'https://luxmc-r92.pages.dev';
const tenant = '9188040d-6c67-4c5b-b112-36a304b66dad';
const issuer = 'https://login.microsoftonline.com/' + tenant + '/v2.0';
const pair = await generateKeyPair('RS256', {extractable:true});
const otherPair = await generateKeyPair('RS256');
const publicKey = {...await exportJWK(pair.publicKey),kid:'luxmc-fixture-key',alg:'RS256',use:'sig'};
const sha = value => createHash('sha256').update(value).digest('hex');
function fixture(t) {
    const sqlite = new DatabaseSync(':memory:');
    sqlite.exec('PRAGMA foreign_keys=ON');
    for (const name of readdirSync(new URL('../website/migrations/', import.meta.url)).filter(name => name.endsWith('.sql')).sort()) sqlite.exec(readFileSync(new URL('../website/migrations/' + name, import.meta.url), 'utf8'));
    const db = {prepare(sql) {const statement=sqlite.prepare(sql); const bound=(values=[])=>({bind:(...next)=>bound(next),async first(){return statement.get(...values)||null;},async all(){return {results:statement.all(...values)};},async run(){return {meta:statement.run(...values)};},execute(){return {meta:statement.run(...values)};}});return bound();},async batch(statements){sqlite.exec('BEGIN');try{const results=statements.map(statement=>statement.execute());sqlite.exec('COMMIT');return results;}catch(error){sqlite.exec('ROLLBACK');throw error;}}};
    const env={SOCIAL_DB:db,MICROSOFT_CLIENT_ID:'9750ebbe-21e9-4a4d-b808-f451a3e0af7f',MICROSOFT_CLIENT_SECRET:'server-only-test-secret',AUTH_PEPPER:'test-pepper-not-for-production-1234567890123456789'};
    const jar=new Map(), calls=[], logs=[], now=Math.floor(Date.now()/1000);
    let claims={}, signingKey=pair.privateKey;
    t.mock.method(console,'error',(...values)=>logs.push(values));
    t.mock.method(globalThis,'fetch',async (target, options={})=>{
        const url=target instanceof Request?target.url:String(target);
        calls.push(url);
        if(url==='https://login.microsoftonline.com/consumers/discovery/v2.0/keys')return Response.json({keys:[publicKey]});
        assert.equal(url,'https://login.microsoftonline.com/consumers/oauth2/v2.0/token');
        assert.equal(options.body.get('client_secret'),env.MICROSOFT_CLIENT_SECRET);
        assert.match(options.body.get('code_verifier'),/^[a-f0-9]{64}$/);
        const token=await new SignJWT({iss:issuer,aud:env.MICROSOFT_CLIENT_ID,sub:'verified-microsoft-subject',tid:tenant,iat:now,exp:now+3600,nonce:await microsoftNonce(options.body.get('code_verifier')),name:'Test Microsoft',...claims}).setProtectedHeader({alg:'RS256',kid:publicKey.kid}).sign(signingKey);
        return Response.json({id_token:token,access_token:'unused-private-access-token'});
    });
    async function call(action,{method=action==='callback'||action==='status'?'GET':'POST',query='',headers={},handler=onRequest,body}={}) {
        const request=new Request(origin+'/api/microsoft/'+action+query,{method,headers:{Origin:origin,Cookie:[...jar].map(([key,value])=>key+'='+value).join('; '),'Content-Type':'application/json',...headers},body:body===undefined?undefined:JSON.stringify(body)});
        const response=await handler({request,env,params:{action},waitUntil(promise){void promise.catch(()=>{});}});
        for(const value of response.headers.getSetCookie()){const [key,token]=value.split(';')[0].split('=');if(value.includes('Max-Age=0'))jar.delete(key);else jar.set(key,token);}
        return response;
    }
    async function start(){const response=await call('start');assert.equal(response.status,200);return new URL((await response.json()).url);}
    const callback=url=>call('callback',{query:'?state='+url.searchParams.get('state')+'&code=test-authorization-code'});
    function profile(id='original-profile',name='Spect3rBW'){sqlite.prepare('INSERT INTO social_users(id,username,token_hash) VALUES(?,?,?)').run(id,name,sha(id));return id;}
    function scoped(id){const token=sha('session-'+id);sqlite.prepare('INSERT OR REPLACE INTO lux_commerce_sessions(token_hash,social_id,expires_at) VALUES(?,?,?)').run(sha(token),id,now+3600);jar.set('__Host-luxmc-commerce',token);}
    async function pending(){return (await call('link_status')).json();}
    async function confirm(){const status=await pending();return call('link',{body:{confirmation:status.confirmation}});}
    t.after(()=>sqlite.close());
    return {sqlite,db,env,jar,calls,logs,now,call,start,callback,profile,scoped,pending,confirm,setClaims(value){claims=value;},badSignature(){signingKey=otherPair.privateKey;}};
}

test('configuration, method and origin are mandatory',async t=>{
    const f=fixture(t);
    assert.equal((await f.call('start',{headers:{Origin:'https://evil.example'}})).status,403);
    assert.equal((await f.call('start',{method:'GET'})).status,405);
    delete f.env.MICROSOFT_CLIENT_SECRET;
    assert.deepEqual(await (await f.call('status')).json(),{enabled:false});
    assert.equal((await f.call('start')).status,503);
    assert.equal(f.calls.length,0);
});

test('OIDC request binds PKCE, nonce and independent HttpOnly cookies',async t=>{
    const f=fixture(t),url=await f.start();
    const row=f.sqlite.prepare('SELECT * FROM lux_microsoft_pending').get();
    assert.equal(url.searchParams.get('scope'),'openid profile');
    assert.equal(url.searchParams.get('code_challenge_method'),'S256');
    assert.equal(url.searchParams.get('nonce'),await microsoftNonce(row.verifier));
    assert.equal(url.searchParams.get('code_challenge'),createHash('sha256').update(row.verifier).digest('base64url'));
    assert.equal(row.state_hash,sha(url.searchParams.get('state')));
    assert.notEqual(url.searchParams.get('state'),f.jar.get('__Host-luxmc-microsoft-'+url.searchParams.get('state')));
    assert.doesNotMatch(url.href,/client_secret|code_verifier|server-only-test-secret/);
});

test('verified Microsoft identity alone cannot create or claim a Luxmc profile',async t=>{
    const f=fixture(t),id=f.profile();
    f.sqlite.prepare('INSERT INTO lux_memberships(social_id,plan,expires_at) VALUES(?,?,?)').run(id,'pro',f.now+86400);
    assert.equal((await f.callback(await f.start())).headers.get('Location'),'/conta?microsoft_link=1');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_accounts').get().n,0);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n,0);
    assert.equal((await f.pending()).profile,null);
    assert.equal((await f.confirm()).status,401);
    assert.ok(f.calls.every(url=>url.startsWith('https://login.microsoftonline.com/')));
    assert.equal(f.sqlite.prepare('SELECT plan FROM lux_memberships').get().plan,'pro');
});

test('explicit authenticated link preserves Pro, friends and social identity',async t=>{
    const f=fixture(t),id=f.profile(),friend=f.profile('friend','Friend');
    f.sqlite.prepare('INSERT INTO lux_memberships(social_id,plan,expires_at) VALUES(?,?,?)').run(id,'pro',f.now+86400);
    f.sqlite.prepare('INSERT INTO social_relationships(sender,recipient,pair_key,accepted,created_at) VALUES(?,?,?,?,?)').run(id,friend,'pair',1,f.now);
    f.scoped(id);
    await f.callback(await f.start());
    const pending=await f.pending();
    assert.equal(pending.profile.username,'Spect3rBW');
    assert.equal((await f.call('link',{body:{confirmation:'0'.repeat(64)}})).status,409);
    assert.equal((await f.confirm()).status,200);
    const identity=await authenticate(new Request(origin,{headers:{Cookie:'__Host-luxmc='+f.jar.get('__Host-luxmc')}}),f.db);
    assert.equal(identity.social_id,id);
    assert.equal(f.sqlite.prepare('SELECT plan FROM lux_memberships').get().plan,'pro');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM social_relationships').get().n,1);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_subjects').get().n,1);
    assert.equal(f.jar.has('__Host-luxmc-microsoft-link'),false);
    assert.equal(f.jar.has('__Host-luxmc-commerce'),false);
    assert.equal((await f.confirm()).status,401);
});

test('returning Microsoft login reuses the linked account without Xbox or Minecraft',async t=>{
    const f=fixture(t),id=f.profile();f.scoped(id);
    await f.callback(await f.start());await f.confirm();
    f.jar.clear();
    assert.equal((await f.callback(await f.start())).headers.get('Location'),'/conta');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_accounts').get().n,1);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n,2);
    assert.ok(f.calls.every(url=>url.startsWith('https://login.microsoftonline.com/')));
    assert.doesNotMatch(JSON.stringify(f.sqlite.prepare('SELECT * FROM lux_accounts').get()),/unused-private-access-token|test-authorization-code/);
});

for(const [name,overrides] of [
    ['audience',{aud:'another-client'}],['issuer',{iss:'https://evil.example'}],['tenant',{tid:'another-tenant'}],
    ['nonce',{nonce:'stolen-nonce'}],['expiry',{iat:1,exp:2}],['subject',{sub:''}]
]) test('rejects invalid signed claim: '+name,async t=>{
    const f=fixture(t);f.setClaims(overrides);
    assert.equal((await f.callback(await f.start())).headers.get('Location'),'/conta?microsoft_error=unavailable');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_link_pending').get().n,0);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n,0);
    assert.doesNotMatch(JSON.stringify(f.logs),/test-authorization-code|unused-private-access-token|server-only-test-secret/);
});

test('rejects a forged signature even with correct claims',async t=>{
    const f=fixture(t);f.badSignature();
    assert.equal((await f.callback(await f.start())).headers.get('Location'),'/conta?microsoft_error=unavailable');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_subjects').get().n,0);
});

test('state theft, expiry and callback replay fail before token exchange',async t=>{
    const f=fixture(t),url=await f.start(),key='__Host-luxmc-microsoft-'+url.searchParams.get('state'),browser=f.jar.get(key);
    f.jar.set(key,'e'.repeat(64));
    assert.equal((await f.callback(url)).headers.get('Location'),'/conta?microsoft_error=expired');
    assert.equal(f.calls.length,0);
    f.jar.set(key,browser);
    f.sqlite.prepare('UPDATE lux_microsoft_pending SET expires_at=0').run();
    assert.equal((await f.callback(url)).headers.get('Location'),'/conta?microsoft_error=expired');
    const valid=await f.start();await f.callback(valid);const count=f.calls.length;
    assert.equal((await f.callback(valid)).headers.get('Location'),'/conta?microsoft_error=expired');
    assert.equal(f.calls.length,count);
});

test('different Microsoft identities or profiles cannot overwrite a link',async t=>{
    const f=fixture(t),first=f.profile(),second=f.profile('second','OtherProfile');
    f.scoped(first);await f.callback(await f.start());await f.confirm();
    f.jar.clear();f.scoped(second);
    assert.equal((await f.callback(await f.start())).headers.get('Location'),'/conta?microsoft_error=conflict');
    f.jar.clear();f.scoped(first);f.setClaims({sub:'another-microsoft-account'});
    await f.callback(await f.start());
    assert.equal((await f.confirm()).status,409);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_subjects').get().n,1);
});

test('parallel tabs keep login states and reject stale account confirmation',async t=>{
    const f=fixture(t),id=f.profile();f.scoped(id);
    const first=await f.start(),second=await f.start();
    await f.callback(first);const previous=await f.pending();
    f.setClaims({sub:'second-microsoft-account'});
    await f.callback(second);
    assert.equal((await f.call('link',{body:{confirmation:previous.confirmation}})).status,409);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_subjects').get().n,0);
});

test('cancelled and expired link confirmations cannot open sessions',async t=>{
    const f=fixture(t),id=f.profile();f.scoped(id);
    await f.callback(await f.start());const old=await f.pending();
    assert.equal((await f.call('cancel_link')).status,200);
    assert.equal((await f.call('link',{body:{confirmation:old.confirmation}})).status,401);
    await f.callback(await f.start());
    f.sqlite.prepare('UPDATE lux_microsoft_link_pending SET expires_at=0').run();
    assert.deepEqual(await f.pending(),{pending:false});
    assert.equal((await f.confirm()).status,401);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n,0);
});

test('Microsoft-only profiles cannot change or recover a local password',async t=>{
    const f=fixture(t),id=f.profile();f.scoped(id);
    await f.callback(await f.start());await f.confirm();
    const response=await f.call('password',{handler:accountHandler,body:{currentPassword:'wrong-password',password:'New-test-password'}});
    assert.equal(response.status,403);
    const recover=await f.call('recover',{handler:accountHandler,body:{username:'Spect3rBW',password:'New-test-password',recoveryCode:'0'.repeat(64)}});
    assert.equal(recover.status,401);
    assert.match((await recover.json()).error,/Microsoft/);
});
