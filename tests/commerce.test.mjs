import test from 'node:test';
import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { onRequest as commerce } from '../website/functions/api/commerce/[action].js';
import { onRequest as receipts } from '../website/functions/api/receipts/[id].js';
import { onRequest as social } from '../website/functions/api/social/[action].js';
import { base32, decryptSecret, otpAt } from '../website/lib/owner-security.js';

const owner = '11111111-1111-4111-8111-111111111111';
const alice = '22222222-2222-4222-8222-222222222222';
const bob = '33333333-3333-4333-8333-333333333333';
const tokens = { [owner]: 'a'.repeat(64), [alice]: 'b'.repeat(64), [bob]: 'c'.repeat(64) };
const hash = value => createHash('sha256').update(value).digest('hex');
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aWQAAAABJRU5ErkJggg==','base64');
function fixture() {
    const sqlite = new DatabaseSync(':memory:'); sqlite.exec('PRAGMA foreign_keys=ON');
    for (const file of readdirSync(new URL('../website/migrations/', import.meta.url)).filter(name => name.endsWith('.sql')).sort()) sqlite.exec(readFileSync(new URL(`../website/migrations/${file}`, import.meta.url), 'utf8'));
    for (const [id, token] of Object.entries(tokens)) sqlite.prepare('INSERT INTO social_users(id, username, token_hash) VALUES (?, ?, ?)').run(id, id === owner ? 'Owner' : id === alice ? 'Alice' : 'Bob', hash(token));
    const db = { prepare(sql) { const statement = sqlite.prepare(sql); const bound = (input = []) => { const values = input.map(value => value instanceof ArrayBuffer ? new Uint8Array(value) : value); return { bind: (...next) => bound(next), first: async () => statement.get(...values) || null, all: async () => ({ results: statement.all(...values) }), run: async () => ({ meta: statement.run(...values) }), execute: () => ({ meta: statement.run(...values) }) }; }; return bound(); }, async batch(statements) { sqlite.exec('BEGIN'); try { const results = statements.map(statement => statement.execute()); sqlite.exec('COMMIT'); return results; } catch(error) { sqlite.exec('ROLLBACK'); throw error; } } };
    const env = { SOCIAL_DB: db, OWNER_SOCIAL_ID: owner, AUTH_PEPPER:'test-only-pepper-not-for-production-123456789' };
    const auth = id => ({ Authorization: `Bearer ${tokens[id] || 'd'.repeat(64)}` });
    let ownerCookie = `__Host-luxmc-commerce=${'9'.repeat(64)}`;
    const cookieHeaders = cookie => ({Authorization:'',Cookie:cookie,'X-Luxmc-CSRF':hash(`luxmc-commerce-csrf:${cookie.split('=')[1]}`)});
    sqlite.prepare('INSERT INTO lux_commerce_sessions(token_hash,social_id,expires_at,verified_at,created_at,last_activity_at) VALUES (?,?,?,?,?,?)').run(hash('9'.repeat(64)),owner,Math.floor(Date.now()/1000)+900,Math.floor(Date.now()/1000),Math.floor(Date.now()/1000),Math.floor(Date.now()/1000));
    async function call(action, id = alice, body = {}, extra = {}) { const response = await commerce({ request: new Request(`https://luxmc.test/api/commerce/${action}`, { method:'POST', headers: { 'Content-Type':'application/json', ...auth(id), ...(id===owner?cookieHeaders(ownerCookie):{}), ...extra }, body:JSON.stringify(body) }), env, params:{action}, waitUntil(promise) { void promise.catch(()=>{}); } }); if(id===owner&&response.headers.get('set-cookie'))ownerCookie=response.headers.get('set-cookie').split(';')[0];return { status:response.status, headers:response.headers, body:await response.json() }; }
    async function upload(id, order, bytes = png, type = 'image/png') { const response = await receipts({ request:new Request(`https://luxmc.test/api/receipts/${order}`,{method:'POST',headers:{...auth(id),'Content-Type':type},body:bytes}), env, params:{id:order}, waitUntil(){} }); return {status:response.status, body:await response.json()}; }
    async function receipt(id, order, extra={}) { return receipts({ request:new Request(`https://luxmc.test/api/receipts/${order}`,{headers:{...auth(id),...(id===owner?cookieHeaders(ownerCookie):{}),...extra}}), env, params:{id:order}, waitUntil(){} }); }
    const approve = async (order, transactionId = 'E12345678901234567890123456789012') => { const row = sqlite.prepare('SELECT o.version, r.sha256 FROM lux_purchase_orders o JOIN lux_purchase_receipts r ON r.order_id = o.id WHERE o.id = ?').get(order); return call('owner_review', owner, { id:order, version:row.version, receiptHash:row.sha256, decision:'approve', creditConfirmed:true, transactionId, note:'Crédito conferido' }); };
    sqlite.prepare('UPDATE lux_commerce_config SET enabled = 1 WHERE id = 1').run();
    return {sqlite, db, env, call, upload, receipt, approve, cookieHeaders};
}
test('owner panel rejects another account, spoofed role, missing credentials and cross-site requests', async t => {
    const f=fixture(); t.after(()=>f.sqlite.close());
    for(const action of ['owner_queue','owner_review','owner_grant','owner_config','owner_find']) assert.equal((await f.call(action,alice,{owner:true,targetId:owner,username:'Owner',role:'owner'})).status,403);
    assert.equal((await f.call('status','unknown')).status,401);
    assert.equal((await f.call('status',owner,{}, {'Sec-Fetch-Site':'cross-site',Origin:'https://evil.test'})).status,403);
    const ticket=(await f.call('portal_ticket',alice,{owner:true})).body; assert.equal(ticket.path,'/planos');
    const competing=await Promise.all([f.call('exchange','unknown',{ticket:ticket.ticket}),f.call('exchange','unknown',{ticket:ticket.ticket})]);assert.deepEqual(competing.map(item=>item.status).sort(),[200,401]);
    const exchanged=competing.find(item=>item.status===200); assert.match(exchanged.headers.get('set-cookie'),/HttpOnly; Secure; SameSite=Strict/);
    const cookie=exchanged.headers.get('set-cookie').split(';')[0];assert.equal((await f.call('status','unknown',{}, f.cookieHeaders(cookie))).body.identity.id,alice);assert.equal((await f.call('owner_queue','unknown',{}, f.cookieHeaders(cookie))).status,403);
    assert.equal((await f.call('exchange','unknown',{ticket:ticket.ticket})).status,401);
    assert.equal((await f.call('exchange','unknown',{ticket:'z'.repeat(64)})).status,401);
    const expired=(await f.call('portal_ticket',owner,{owner:true})).body;
    f.sqlite.prepare('UPDATE lux_commerce_tickets SET expires_at = 1 WHERE token_hash = ?').run(hash(expired.ticket));
    assert.equal((await f.call('exchange','unknown',{ticket:expired.ticket})).status,401);
});
test('checkout trusts server prices, is idempotent and preserves the payment destination', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());
    const created=await f.call('create',alice,{plan:'plus',amount:1,socialId:bob}); assert.equal(created.status,200); assert.equal(created.body.order.amount,990);assert.equal(created.body.order.socialId,alice);
    assert.equal((await f.call('create',alice,{plan:'plus'})).body.order.id,created.body.order.id);
    assert.equal((await f.call('create',alice,{plan:'pro'})).status,409);
    f.sqlite.prepare("UPDATE lux_commerce_config SET pix_key = 'other@example.test'").run();
    assert.notEqual((await f.call('status')).body.orders[0].pixKey,'other@example.test');
    assert.equal((await f.call('create',bob,{plan:'owner'})).status,400);
    f.sqlite.prepare('UPDATE lux_commerce_config SET enabled = 0').run(); assert.equal((await f.call('create',bob,{plan:'pro'})).status,503);
});
test('owner password is hashed, scoped to the owner, rate limited and changing it revokes other sessions', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const secret='test-owner-password-strong-123';
    assert.equal((await f.call('owner_password',alice,{password:secret})).status,403);
    assert.equal((await f.call('owner_password',owner,{password:'weak'})).status,400);
    assert.equal((await f.call('owner_password',owner,{password:secret})).status,200);
    const stored=f.sqlite.prepare('SELECT owner_password_hash, owner_password_salt FROM lux_commerce_config').get();assert.notEqual(stored.owner_password_hash,secret);assert.match(stored.owner_password_hash,/^[a-f0-9]{64}$/);assert.match(stored.owner_password_salt,/^[a-f0-9]{64}$/);
    assert.equal((await f.call('owner_login','unknown',{password:'invalid-password-123'})).status,401);
    const login=await f.call('owner_login','unknown',{password:secret});assert.equal(login.status,200);const cookie=login.headers.get('set-cookie').split(';')[0];assert.match(login.headers.get('set-cookie'),/Max-Age=900/);
    assert.equal((await f.call('owner_queue','unknown',{}, f.cookieHeaders(cookie))).status,200);
    assert.equal((await f.call('owner_password',owner,{password:'replacement-owner-password-456'})).status,200);
    assert.equal((await f.call('owner_queue','unknown',{}, f.cookieHeaders(cookie))).status,401);
    for(let i=0;i<4;i++) await f.call('owner_login','unknown',{password:'invalid-password-123'});
    assert.equal((await f.call('owner_login','unknown',{password:secret})).status,429);
});
test('receipts stay private, validate actual file type, and uploads cannot revive cancelled orders', async t => {
    const f=fixture();t.after(()=>f.sqlite.close()); const order=(await f.call('create',alice,{plan:'pro'})).body.order.id;
    assert.equal((await f.upload(bob,order)).status,404);
    assert.equal((await f.upload(alice,order,Buffer.from('<html>not a receipt</html>'))).status,415);
    assert.equal((await f.upload(alice,order,png,'application/pdf')).status,415);
    assert.equal((await f.upload(alice,order,Buffer.alloc(1048577))).status,413);
    assert.equal((await f.upload(alice,order)).status,200);
    assert.equal((await f.receipt(bob,order)).status,404); assert.equal((await f.receipt('unknown',order)).status,401);
    const own=await f.receipt(alice,order);assert.equal(own.headers.get('cache-control'),'no-store'); assert.deepEqual(Buffer.from(await own.arrayBuffer()),png); assert.equal((await f.receipt(owner,order)).status,200);
    assert.equal((await f.call('cancel',bob,{id:order})).status,409);
    assert.equal((await f.call('cancel',alice,{id:order})).status,200); assert.equal((await f.upload(alice,order)).status,409); assert.equal((await f.approve(order)).status,409);
    assert.equal((await f.call('status')).body.membership.plan,'free');
});
test('approval is atomic, needs bank confirmation, blocks duplicate Pix transactions and renews only once', async t => {
    const f=fixture();t.after(()=>f.sqlite.close()); const order=(await f.call('create',alice,{plan:'plus'})).body.order.id;await f.upload(alice,order);
    const row=f.sqlite.prepare('SELECT version FROM lux_purchase_orders WHERE id = ?').get(order);
    assert.equal((await f.call('owner_review',owner,{id:order,version:row.version,receiptHash:hash(png),decision:'approve',note:''})).status,400);
    const competing=await Promise.all([f.approve(order),f.approve(order)]);assert.deepEqual(competing.map(item=>item.status).sort(),[200,409]);const result=competing.find(item=>item.status===200);assert.equal(result.body.membership.plan,'plus');const expiry=result.body.membership.expiresAt;
    assert.equal((await f.approve(order)).status,409);assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS total FROM lux_purchase_audit').get().total,1);
    const next=(await f.call('create',alice,{plan:'plus'})).body.order.id;await f.upload(alice,next);assert.equal((await f.approve(next)).status,409);assert.equal((await f.call('status')).body.membership.expiresAt,expiry);
    const renewed=await f.approve(next,'E98765432109876543210987654321098');assert.equal(renewed.status,200);assert.equal(renewed.body.membership.expiresAt,expiry+30*86400);
    const other=(await f.call('create',bob,{plan:'plus'})).body.order.id;await f.upload(bob,other);assert.equal((await f.approve(other)).status,409);assert.equal((await f.call('status',bob)).body.membership.plan,'free');
});
test('a replaced receipt invalidates the old review and rejection grants no membership', async t => {
    const f=fixture();t.after(()=>f.sqlite.close()); const order=(await f.call('create',alice,{plan:'pro'})).body.order.id;await f.upload(alice,order);const stale=(await f.call('status')).body.orders[0]; await f.upload(alice,order);
    assert.equal((await f.call('owner_review',owner,{id:order,version:stale.version,receiptHash:stale.receiptHash,decision:'approve',note:'',transactionId:'E12345678901234567890123456789012',creditConfirmed:true})).status,409);
    const fresh=(await f.call('status')).body.orders[0];assert.equal((await f.call('owner_review',owner,{id:order,version:fresh.version,receiptHash:fresh.receiptHash,decision:'reject',note:'Crédito não localizado'})).status,200);
    assert.equal((await f.call('status')).body.membership.plan,'free');assert.equal((await f.call('status')).body.orders[0].note,'Crédito não localizado');
});
test('paid preferences use revisions, reject paths and preserve presets after a downgrade', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());
    assert.equal((await f.call('preferences',alice,{revision:0,preferences:{density:'compact'},presets:[]})).status,403);
    assert.equal((await f.call('owner_grant',owner,{targetId:alice,confirmationCode:alice,plan:'pro',days:30,revision:0,note:'Teste de perfil'})).status,200);
    const presets=Array.from({length:8},(_,i)=>({id:`44444444-4444-4444-8444-${String(i).padStart(12,'0')}`,name:`Perfil ${i}`,preferences:{density:'compact'}}));
    let result=await f.call('preferences',alice,{revision:0,preferences:{density:'compact',javaPath:'C:/secret',accountToken:'secret'},presets});assert.equal(result.status,200);
    assert.deepEqual((await f.call('status')).body.settings.preferences,{density:'compact'});
    assert.equal((await f.call('preferences',alice,{revision:0,preferences:{density:'spacious'},presets})).status,409);
    assert.equal((await f.call('preferences',alice,{revision:1,preferences:{density:'bad'},presets})).status,400);
    assert.equal((await f.call('preferences',alice,{revision:1,preferences:{motionStyle:'subtle'},presets})).status,200);
    assert.equal((await f.call('owner_grant',owner,{targetId:alice,confirmationCode:alice,plan:'plus',days:30,revision:1,note:'Alteração de plano'})).status,200);
    assert.equal((await f.call('preferences',alice,{revision:2,preferences:{density:'spacious'},presets})).status,200);
    assert.equal((await f.call('preferences',alice,{revision:3,preferences:{},presets:[...presets,{id:'99999999-9999-4999-8999-999999999999',name:'Novo',preferences:{}}]})).status,400);
    f.sqlite.prepare('UPDATE lux_memberships SET expires_at = 1 WHERE social_id = ?').run(alice);assert.equal((await f.call('status')).body.membership.plan,'free');assert.equal((await f.call('status')).body.settings.presets.length,8);
});
test('Microsoft and registered Luxmc identities keep their membership independently of nickname', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const accountId='55555555-5555-4555-8555-555555555555';const session='e'.repeat(64);
    f.sqlite.prepare("INSERT INTO lux_accounts(id,username,social_id,password_hash,password_salt,recovery_hash,created_at) VALUES (?,'LuxUser',?,'hash','salt','recovery',1)").run(accountId,bob);
    f.sqlite.prepare("INSERT INTO lux_sessions(token_hash, account_id, expires_at, created_at, client) VALUES (?, ?, ?, 1, 'launcher')").run(hash(session),accountId,Math.floor(Date.now()/1000)+3600);
    await f.call('owner_grant',owner,{targetId:bob,confirmationCode:bob,plan:'plus',days:30,revision:0,note:'Conta registrada'});
    const status=await f.call('status','unknown',{}, {Authorization:`Bearer ${session}`});assert.equal(status.status,200);assert.equal(status.body.identity.id,bob);assert.equal(status.body.membership.plan,'plus');
    f.sqlite.prepare("UPDATE social_users SET username = 'Different' WHERE id = ?").run(bob);assert.equal((await f.call('status','unknown',{}, {Authorization:`Bearer ${session}`})).body.membership.plan,'plus');
    const response=await social({request:new Request('https://luxmc.test/api/social/profile_get',{method:'POST',headers:{Authorization:`Bearer ${tokens[bob]}`,'Content-Type':'application/json'},body:'{}'}),env:f.env,params:{action:'profile_get'},waitUntil(){}});assert.equal((await response.json()).profile.membership.plan,'plus');
});
test('manual grants distinguish duplicate nicknames and reject a mismatched account code', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());
    f.sqlite.prepare("UPDATE social_users SET username = 'SamePlayer' WHERE id IN (?, ?)").run(alice,bob);
    const found=await f.call('owner_find',owner,{query:'SamePlayer'});assert.equal(found.status,200);assert.equal(found.body.users.length,2);assert.notEqual(found.body.users[0].friendCode,found.body.users[1].friendCode);
    const exact=await f.call('owner_find',owner,{query:'sameplayer#33333333'});assert.equal(exact.body.users.length,1);assert.equal(exact.body.users[0].id,bob);
    assert.equal((await f.call('owner_find',owner,{query:'SamePlayer#12345678'})).body.users.length,0);
    const grant={targetId:bob,plan:'pro',days:30,revision:0};
    assert.equal((await f.call('owner_grant',owner,grant)).status,400);
    assert.equal((await f.call('owner_grant',owner,{...grant,confirmationCode:'SamePlayer#22222222'})).status,400);
    assert.equal((await f.call('owner_grant',owner,{...grant,confirmationCode:alice})).status,400);
    assert.equal((await f.call('status',bob)).body.membership.plan,'free');
    const updated=await f.call('owner_grant',owner,{...grant,confirmationCode:'SamePlayer#33333333'});assert.equal(updated.status,200);assert.equal(updated.body.identity.id,bob);assert.equal(updated.body.membership.plan,'pro');
    assert.equal((await f.call('status',alice)).body.membership.plan,'free');
    assert.equal((await f.call('owner_grant',owner,{...grant,confirmationCode:bob})).status,409);
    const own=await f.call('owner_grant',owner,{targetId:owner,confirmationCode:owner,plan:'pro',days:30,revision:0,note:''});assert.equal(own.status,200);assert.equal((await f.call('status',owner)).body.membership.plan,'pro');
    const queue=await f.call('owner_queue',owner);assert.equal(queue.body.audit.length,2);assert.equal(queue.body.stats.approvedAmount,null);assert.deepEqual(new Set(queue.body.audit.map(entry=>entry.social_id)),new Set([bob,owner]));
});
test('a paid collection limit is enforced on the server and expired collections are kept', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const collections=Array.from({length:12},(_,i)=>({id:`collection-${i}`,title:`Pack ${i}`,description:'',entries:[]}));
    const save=async items=>{const response=await social({request:new Request('https://luxmc.test/api/social/profile_save',{method:'POST',headers:{Authorization:`Bearer ${tokens[alice]}`,'Content-Type':'application/json'},body:JSON.stringify({description:'',banner:'',portrait:'',packs:[],collections:items})}),env:f.env,params:{action:'profile_save'},waitUntil(){}});return {status:response.status,body:await response.json()};};
    assert.equal((await save(collections)).status,403);await f.call('owner_grant',owner,{targetId:alice,confirmationCode:alice,plan:'plus',days:30,revision:0,note:'Coleções ampliadas'});assert.equal((await save(collections)).status,200);
    f.sqlite.prepare('UPDATE lux_memberships SET expires_at = 1 WHERE social_id = ?').run(alice);assert.equal((await save(collections)).status,200);assert.equal((await save([...collections,{id:'extra',title:'extra',description:'',entries:[]}])).status,403);
    const read = async limit => { const response=await social({request:new Request('https://luxmc.test/api/social/profile_get',{method:'POST',headers:{Authorization:`Bearer ${tokens[alice]}`,'Content-Type':'application/json'},body:JSON.stringify({collectionsLimit:limit})}),env:f.env,params:{action:'profile_get'},waitUntil(){}});return (await response.json()).profile; };
    assert.equal((await read()).collections.length,6);assert.equal((await read(24)).collections.length,12);
    assert.equal((await save(collections.slice(0,6))).status,200);assert.equal((await read(24)).collections.length,12);
});
test('owner APIs and private receipts reject a native token without password confirmation', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const native={Authorization:`Bearer ${tokens[owner]}`,Cookie:''};
    assert.equal((await f.call('owner_queue',owner,{},native)).status,428);
    assert.equal((await f.call('owner_grant',owner,{targetId:alice,confirmationCode:alice,plan:'pro',days:30,revision:0},native)).status,428);
    const order=(await f.call('create',alice,{plan:'plus'})).body.order.id;await f.upload(alice,order);assert.equal((await f.receipt(owner,order,native)).status,428);
    assert.equal((await f.call('status',alice)).body.membership.plan,'free');
    await f.call('owner_password',owner,{password:'existing-test-password-123'});
    assert.equal((await f.call('owner_password',owner,{password:'attacker-replacement-password-123'},native)).status,428);
});
test('owner cookie requests require a CSRF token and sensitive actions need a recent confirmation', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());
    assert.equal((await f.call('owner_find',owner,{query:'Alice'},{'X-Luxmc-CSRF':''})).status,403);
    assert.equal((await f.call('owner_config',owner,{bank:'Nubank',pixKey:'safe@example.test',enabled:false},{'X-Luxmc-CSRF':'wrong'})).status,403);
    assert.equal((await f.call('owner_find',owner,{query:'Alice'}, {Origin:'http://localhost:1420'})).status,403);
    const state=(await f.call('status',owner)).body;assert.match(state.csrfToken,/^[a-f0-9]{64}$/);
    f.sqlite.prepare('UPDATE lux_commerce_sessions SET verified_at = ? WHERE social_id = ?').run(Math.floor(Date.now()/1000)-301,owner);
    assert.equal((await f.call('owner_queue',owner)).status,200);
    assert.equal((await f.call('owner_grant',owner,{targetId:alice,confirmationCode:alice,plan:'plus',days:30,revision:0})).status,428);
    f.sqlite.prepare('UPDATE lux_commerce_sessions SET last_activity_at = ? WHERE social_id = ?').run(Math.floor(Date.now()/1000)-301,owner);
    assert.equal((await f.call('owner_queue',owner)).status,428);
    f.sqlite.prepare('UPDATE lux_commerce_sessions SET expires_at = 1 WHERE social_id = ?').run(owner);
    assert.equal((await f.call('owner_queue',owner)).status,401);
});
test('buyers using browser cookies cannot create orders without a session-bound CSRF token', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const ticket=(await f.call('portal_ticket',alice)).body.ticket;
    const exchanged=await f.call('exchange','unknown',{ticket});const cookie=exchanged.headers.get('set-cookie').split(';')[0];
    assert.equal((await f.call('create','unknown',{plan:'plus'}, {...f.cookieHeaders(cookie),'X-Luxmc-CSRF':''})).status,403);
    assert.equal((await f.call('create','unknown',{plan:'plus'}, {...f.cookieHeaders(cookie),'X-Luxmc-CSRF':'forged'})).status,403);
    assert.equal((await f.call('create','unknown',{plan:'plus'}, f.cookieHeaders(cookie))).status,200);
});
test('closing other sessions revokes scoped sessions and one-use links', async t => {
    const f=fixture();t.after(()=>f.sqlite.close());const other='f'.repeat(64);
    f.sqlite.prepare('INSERT INTO lux_commerce_sessions(token_hash,social_id,expires_at,verified_at,last_activity_at) VALUES (?,?,?,?,?)').run(hash(other),owner,Math.floor(Date.now()/1000)+900,Math.floor(Date.now()/1000),Math.floor(Date.now()/1000));
    const ticket=(await f.call('portal_ticket',owner,{owner:true})).body.ticket;
    assert.equal((await f.call('owner_revoke_sessions',owner)).status,200);
    assert.equal((await f.call('status','unknown',{}, f.cookieHeaders(`__Host-luxmc-commerce=${other}`))).status,401);
    assert.equal((await f.call('exchange','unknown',{ticket})).status,401);
    assert.equal((await f.call('owner_queue',owner)).status,200);
});
test('TOTP matches RFC 6238 vectors, stores an encrypted secret and rejects reused codes', async t => {
    const secret=base32(new TextEncoder().encode('12345678901234567890'));
    for(const [seconds,code] of [[59,'94287082'],[1111111109,'07081804'],[1111111111,'14050471'],[1234567890,'89005924'],[2000000000,'69279037'],[20000000000,'65353130']]) assert.equal(await otpAt(secret,Math.floor(seconds/30),8),code);
    const f=fixture();t.after(()=>f.sqlite.close());const password='test-authenticator-password-123';await f.call('owner_password',owner,{password});
    const begin=await f.call('owner_totp_begin',owner,{password});assert.equal(begin.status,200);assert.match(begin.body.secret,/^[A-Z2-7]{32}$/);assert.match(begin.body.uri,/^otpauth:\/\/totp\/Luxmc:/);
    const encrypted=f.sqlite.prepare('SELECT owner_totp_pending FROM lux_commerce_config').get().owner_totp_pending;assert.notEqual(encrypted,begin.body.secret);assert.equal(await decryptSecret(encrypted,f.env.AUTH_PEPPER),begin.body.secret);
    const step=Math.floor(Date.now()/30000);const otp=await otpAt(begin.body.secret,step);
    const enabled=await f.call('owner_totp_enable',owner,{otp});assert.equal(enabled.status,200);assert.equal(enabled.body.recoveryCodes.length,8);
    assert.equal((await f.call('status',owner)).body.security.twoFactor,true);
    assert.equal((await f.call('owner_login','unknown',{password})).status,401);
    assert.equal((await f.call('owner_login','unknown',{password,otp})).status,401);
    const stored=f.sqlite.prepare('SELECT owner_totp_secret,owner_recovery_hashes FROM lux_commerce_config').get();assert.notEqual(stored.owner_totp_secret,begin.body.secret);for(const code of enabled.body.recoveryCodes)assert.ok(!stored.owner_recovery_hashes.includes(code));
    const recovered=await f.call('owner_login','unknown',{password,otp:enabled.body.recoveryCodes[0]});assert.equal(recovered.status,200);
    f.sqlite.prepare('DELETE FROM social_limits').run();assert.equal((await f.call('owner_login','unknown',{password,otp:enabled.body.recoveryCodes[0]})).status,401);
    f.sqlite.prepare('UPDATE lux_commerce_config SET owner_totp_last_step = ?').run(step-1);const login=await f.call('owner_login','unknown',{password,otp});assert.equal(login.status,200);assert.equal((await f.call('owner_login','unknown',{password,otp})).status,401);
    f.sqlite.prepare('DELETE FROM social_limits').run();const loginCookie=login.headers.get('set-cookie').split(';')[0];
    assert.equal((await f.call('owner_totp_disable','unknown',{password:'wrong-password-123',otp:enabled.body.recoveryCodes[1]},f.cookieHeaders(loginCookie))).status,401);
    const disabled=await f.call('owner_totp_disable','unknown',{password,otp:enabled.body.recoveryCodes[1]},f.cookieHeaders(loginCookie));assert.equal(disabled.status,200);const disabledCookie=disabled.headers.get('set-cookie').split(';')[0];assert.equal((await f.call('status','unknown',{},f.cookieHeaders(disabledCookie))).body.security.twoFactor,false);
});
