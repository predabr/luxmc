import test from 'node:test';
import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { onRequest } from '../website/functions/api/microsoft/[action].js';
import { onRequest as account } from '../website/functions/api/account/[action].js';
import { authenticate } from '../website/lib/accounts.js';

const origin = 'https://luxmc-r92.pages.dev';
const uuid = '8a5bec034d584f96908ae56015586a6d';
const sha = value => createHash('sha256').update(value).digest('hex');
function fixture(t) {
    const sqlite = new DatabaseSync(':memory:');
    sqlite.exec('PRAGMA foreign_keys=ON');
    for (const file of readdirSync(new URL('../website/migrations/', import.meta.url)).filter(name => name.endsWith('.sql')).sort()) sqlite.exec(readFileSync(new URL(`../website/migrations/${file}`, import.meta.url), 'utf8'));
    const db = { prepare(sql) { const statement = sqlite.prepare(sql); const bound = (values = []) => ({ bind: (...next) => bound(next), async first() { return statement.get(...values) || null; }, async all() { return { results: statement.all(...values) }; }, async run() { return { meta: statement.run(...values) }; }, execute() { return { meta: statement.run(...values) }; } }); return bound(); }, async batch(statements) { sqlite.exec('BEGIN'); try { const result = statements.map(item => item.execute()); sqlite.exec('COMMIT'); return result; } catch (error) { sqlite.exec('ROLLBACK'); throw error; } } };
    const env = { SOCIAL_DB: db, MICROSOFT_CLIENT_ID: '9750ebbe-21e9-4a4d-b808-f451a3e0af7f', MICROSOFT_CLIENT_SECRET: 'server-only-test-secret', AUTH_PEPPER: 'test-pepper-not-for-production-1234567890123456789' };
    const jar = new Map(), calls = [], now = Math.floor(Date.now() / 1000);
    let profile = { id: uuid, name: 'Spect3rBW' }, missingGame = false;
    t.mock.method(globalThis, 'fetch', async (url, options) => {
        calls.push({ url, options });
        if (url === 'https://login.microsoftonline.com/consumers/oauth2/v2.0/token') {
            assert.equal(options.body.get('client_secret'), env.MICROSOFT_CLIENT_SECRET);
            assert.match(options.body.get('code_verifier'), /^[a-f0-9]{64}$/);
            return Response.json({ access_token: 'test-microsoft-token' });
        }
        if (url === 'https://user.auth.xboxlive.com/user/authenticate') return Response.json({ Token: 'test-xbox-token' });
        if (url === 'https://xsts.auth.xboxlive.com/xsts/authorize') return Response.json({ Token: 'test-xsts-token', DisplayClaims: { xui: [{ uhs: '123456789' }] } });
        if (url === 'https://api.minecraftservices.com/authentication/login_with_xbox') return Response.json({ access_token: 'test-minecraft-token' });
        if (url === 'https://api.minecraftservices.com/minecraft/profile') return missingGame ? new Response(null, { status: 404 }) : Response.json(profile);
        throw new Error('Unexpected authentication URL');
    });
    async function call(action, { method = action === 'start' ? 'POST' : 'GET', query = '', headers = {}, handler = onRequest, body } = {}) {
        const request = new Request(`${origin}/api/microsoft/${action}${query}`, { method, headers: { Origin: origin, Cookie: [...jar].map(([key, value]) => `${key}=${value}`).join('; '), 'Content-Type': 'application/json', ...headers }, body: body === undefined ? undefined : JSON.stringify(body) });
        const response = await handler({ request, env, params: { action }, waitUntil(promise) { void promise.catch(() => {}); } });
        for (const cookie of response.headers.getSetCookie()) { const [key, value] = cookie.split(';')[0].split('='); if (cookie.includes('Max-Age=0')) jar.delete(key); else jar.set(key, value); }
        return response;
    }
    async function start() { const response = await call('start'); assert.equal(response.status, 200); return new URL((await response.json()).url); }
    const callback = url => call('callback', { query: `?state=${url.searchParams.get('state')}&code=test-authorization-code` });
    function legacy() {
        const id = '43d26fda-6fbd-4727-8680-f96aca47dcf3';
        sqlite.prepare('INSERT INTO social_users(id,username,token_hash) VALUES(?,?,?)').run(id, 'Spect3rBW', sha('legacy-owner-token'));
        sqlite.prepare('INSERT INTO lux_memberships(social_id,plan,expires_at) VALUES(?,?,?)').run(id, 'pro', now + 86400);
        return id;
    }
    function scoped(id) { const token = 'd'.repeat(64); sqlite.prepare('INSERT INTO lux_commerce_sessions(token_hash,social_id,expires_at) VALUES(?,?,?)').run(sha(token), id, now + 3600); jar.set('__Host-luxmc-commerce', token); }
    t.after(() => sqlite.close());
    return { sqlite, db, env, jar, calls, call, start, callback, legacy, scoped, setProfile(value) { profile = value; }, missingGame() { missingGame = true; } };
}

test('Microsoft login requires explicit web configuration and a matching origin', async t => {
    const f = fixture(t);
    assert.equal((await f.call('start', { headers: { Origin: 'https://evil.example' } })).status, 403);
    assert.equal((await f.call('start', { method: 'GET' })).status, 405);
    delete f.env.MICROSOFT_CLIENT_SECRET;
    assert.deepEqual(await (await f.call('status')).json(), { enabled: false });
    assert.equal((await f.call('start')).status, 503);
    assert.equal(f.jar.size, 0);
    assert.equal(f.calls.length, 0);
});

test('authorization uses PKCE and a separate HttpOnly browser binding', async t => {
    const f = fixture(t), url = await f.start();
    assert.equal(url.origin, 'https://login.microsoftonline.com');
    assert.equal(url.searchParams.get('code_challenge_method'), 'S256');
    assert.equal(url.searchParams.get('redirect_uri'), `${origin}/api/microsoft/callback`);
    assert.equal(url.searchParams.get('scope'), 'XboxLive.signin');
    assert.notEqual(url.searchParams.get('state'), f.jar.get('__Host-luxmc-microsoft'));
    assert.doesNotMatch(url.href, /server-only-test-secret|code_verifier/);
    const pending = f.sqlite.prepare('SELECT * FROM lux_microsoft_pending').get();
    assert.equal(pending.state_hash, sha(url.searchParams.get('state')));
    assert.equal(pending.browser_hash, sha(f.jar.get('__Host-luxmc-microsoft')));
    assert.equal(url.searchParams.get('code_challenge'), createHash('sha256').update(pending.verifier).digest('base64url'));
});

test('provider failures log a safe stage without credentials or provider descriptions', async t => {
    const f = fixture(t), logs = [], url = await f.start();
    t.mock.method(console, 'error', (...values) => logs.push(values));
    t.mock.method(globalThis, 'fetch', async () => Response.json({ error: 'invalid_client', error_description: 'private-provider-description', access_token: 'private-provider-token' }, { status: 400 }));
    assert.equal((await f.callback(url)).headers.get('Location'), '/conta?microsoft_error=unavailable');
    assert.deepEqual(logs[0][1], { action: 'callback', service: 'login.microsoftonline.com/consumers/oauth2/v2.0/token', status: 400, providerCode: 'invalid_client' });
    assert.doesNotMatch(JSON.stringify(logs), /private-provider|server-only-test-secret|test-authorization-code|code_verifier/);
});

test('callback rejects stolen state, expiry and replay before exchanging a token', async t => {
    const f = fixture(t), url = await f.start(), browser = f.jar.get('__Host-luxmc-microsoft');
    f.jar.set('__Host-luxmc-microsoft', 'e'.repeat(64));
    assert.equal((await f.callback(url)).headers.get('Location'), '/conta?microsoft_error=expired');
    assert.equal(f.calls.length, 0);
    f.jar.set('__Host-luxmc-microsoft', browser);
    f.sqlite.prepare('UPDATE lux_microsoft_pending SET expires_at=0').run();
    assert.equal((await f.callback(url)).headers.get('Location'), '/conta?microsoft_error=expired');
    const valid = await f.start();
    assert.equal((await f.callback(valid)).headers.get('Location'), '/conta');
    const count = f.calls.length;
    f.jar.set('__Host-luxmc-microsoft', browser);
    assert.equal((await f.callback(valid)).headers.get('Location'), '/conta?microsoft_error=expired');
    assert.equal(f.calls.length, count);
});

test('same Microsoft account keeps one social identity across fresh browser sessions', async t => {
    const f = fixture(t);
    const response = await f.callback(await f.start());
    assert.equal(response.status, 303);
    assert.equal(response.headers.get('Location'), '/conta');
    assert.ok(response.headers.getSetCookie().some(value => value.startsWith('__Host-luxmc=') && value.includes('HttpOnly; Secure; SameSite=Strict')));
    const identity = await authenticate(new Request(origin, { headers: { Cookie: `__Host-luxmc=${f.jar.get('__Host-luxmc')}` } }), f.db);
    assert.equal(identity.username, 'Spect3rBW');
    assert.equal(identity.auth_provider, 'microsoft');
    f.jar.clear();
    assert.equal((await f.callback(await f.start())).headers.get('Location'), '/conta');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM social_users').get().n, 1);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_accounts').get().n, 1);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n, 2);
    assert.doesNotMatch(JSON.stringify(f.sqlite.prepare('SELECT * FROM lux_accounts').get()), /test-microsoft-token|test-minecraft-token|test-xsts-token/);
});

test('authenticated legacy profile links to Microsoft without losing its existing Pro or friendships', async t => {
    const f = fixture(t), id = f.legacy();
    f.sqlite.prepare('INSERT INTO social_users(id,username,token_hash) VALUES(?,?,?)').run('friend', 'Friend', sha('friend-token'));
    f.sqlite.prepare('INSERT INTO social_relationships(sender,recipient,pair_key,accepted,created_at) VALUES(?,?,?,?,?)').run(id, 'friend', 'pair', 1, 1);
    f.scoped(id);
    assert.equal((await f.callback(await f.start())).headers.get('Location'), '/conta');
    assert.equal(f.sqlite.prepare('SELECT social_id FROM lux_accounts').get().social_id, id);
    assert.equal(f.sqlite.prepare('SELECT plan FROM lux_memberships WHERE social_id=?').get(id).plan, 'pro');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM social_relationships').get().n, 1);
    assert.equal(f.jar.has('__Host-luxmc-commerce'), false);
});

test('matching nickname alone cannot claim another profile or its paid plan', async t => {
    const f = fixture(t), old = f.legacy();
    assert.equal((await f.callback(await f.start())).headers.get('Location'), '/conta');
    assert.notEqual(f.sqlite.prepare('SELECT social_id FROM lux_accounts').get().social_id, old);
    assert.equal(f.sqlite.prepare('SELECT plan FROM lux_memberships WHERE social_id=?').get(old).plan, 'pro');
    const other = 'another-profile';
    f.sqlite.prepare('INSERT INTO social_users(id,username,token_hash) VALUES(?,?,?)').run(other, 'Spect3rBW', sha('other-token'));
    f.jar.clear(); f.scoped(other);
    assert.equal((await f.callback(await f.start())).headers.get('Location'), '/conta?microsoft_error=conflict');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_microsoft_accounts').get().n, 1);
});

test('unlicensed Minecraft profiles and upstream failures cannot create a session', async t => {
    const f = fixture(t);
    f.missingGame();
    assert.equal((await f.callback(await f.start())).headers.get('Location'), '/conta?microsoft_error=minecraft');
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_sessions').get().n, 0);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS n FROM lux_accounts').get().n, 0);
});

test('Microsoft users cannot reset a local password or modify the Microsoft password through Luxmc', async t => {
    const f = fixture(t);
    await f.callback(await f.start());
    const change = await f.call('password', { method: 'POST', handler: account, body: { currentPassword: 'wrong-password', password: 'New-test-password' } });
    assert.equal(change.status, 403);
    const recover = await f.call('recover', { method: 'POST', handler: account, body: { username: 'Spect3rBW', password: 'New-test-password', recoveryCode: '0'.repeat(64) } });
    assert.equal(recover.status, 401);
    assert.match((await recover.json()).error, /Microsoft/);
});
