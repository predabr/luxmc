import test from 'node:test';
import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { onRequest as social } from '../website/functions/api/social/[action].js';
import { onRequest as commerce } from '../website/functions/api/commerce/[action].js';
import { defaultProfileStyle, parseProfileStyle, profileBenefits, isProfileGif } from '../website/lib/profile-style.js';

const alice = '22222222-2222-4222-8222-222222222222';
const bob = '33333333-3333-4333-8333-333333333333';
const tokens = { [alice]: 'b'.repeat(64), [bob]: 'c'.repeat(64) };
const proStyle = { accent:'lilac', frame:'orbit', effect:'shimmer', layout:'centered' };
const empty = { description:'', banner:'', portrait:'', packs:[], collections:[] };

test('optimized animated WebP follows the same plan and expiry rules as GIF', async t => {
    const f = fixture(t);
    const header = Buffer.alloc(30);
    header.write('RIFF'); header.writeUInt32LE(22,4); header.write('WEBPVP8X',8); header.writeUInt32LE(10,16);
    const staticImage = 'data:image/webp;base64,'+header.toString('base64');
    header[20] = 2;
    const animated = 'data:image/webp;base64,'+header.toString('base64');
    assert.equal(isProfileGif(staticImage),false); assert.equal(isProfileGif(animated),true);
    assert.equal((await f.call('social','profile_save',alice,{...empty,portrait:animated,banner:animated})).status,403);
    f.plan(alice,'plus');
    assert.equal((await f.call('social','profile_save',alice,{...empty,portrait:animated,banner:animated})).status,200);
    f.plan(alice,'plus',1);
    let profile = (await f.call('social','profile_get',alice,{})).body.profile;
    assert.equal(profile.portrait,''); assert.equal(profile.banner,'');
    assert.equal((await f.call('social','profile_save',alice,{...empty,description:'Preserve my images'})).status,200);
    f.plan(alice,'plus');
    profile = (await f.call('social','profile_get',alice,{})).body.profile;
    assert.equal(profile.portrait,animated); assert.equal(profile.banner,animated);
});

test('friend card covers and Prism require Pro and survive downgrade without remaining active', async t => {
    const f = fixture(t);
    const style = {...defaultProfileStyle, frameDetail:'prism', cardBackground:'cover'};
    const gif = 'data:image/gif;base64,' + Buffer.from('GIF89a1234567890').toString('base64');
    f.plan(alice,'plus');
    assert.equal((await f.call('social','profile_save',alice,{...empty,banner:gif,profileStyle:style})).status,403);
    f.plan(alice,'pro');
    assert.equal((await f.call('social','profile_save',alice,{...empty,banner:gif,profileStyle:style})).status,200);
    assert.deepEqual((await f.call('social','profile_get',alice,{})).body.profile.profileStyle,style);
    assert.equal((await f.call('social','profile_save',alice,{...empty,banner:gif,description:'Edited by a previous launcher',profileStyle:defaultProfileStyle})).status,200);
    assert.deepEqual((await f.call('social','profile_get',alice,{})).body.profile.profileStyle,style);
    f.plan(alice,'plus');
    let profile = (await f.call('social','profile_get',alice,{})).body.profile;
    assert.equal(profile.profileStyle.cardBackground,'none');
    assert.equal(profile.profileStyle.frameDetail,'none');
    assert.equal(profile.banner,gif);
    assert.equal((await f.call('social','profile_save',alice,{...empty,banner:gif,profileStyle:profile.profileStyle})).status,200);
    f.plan(alice,'pro');
    profile = (await f.call('social','profile_get',alice,{})).body.profile;
    assert.deepEqual(profile.profileStyle,style);
});

test('GIFs require Plus or Pro, survive expiry and reach own identity and accepted friends', async t => {
    const f = fixture(t);
    const gif = 'data:image/gif;base64,' + Buffer.from('GIF89a1234567890').toString('base64');
    const draft = {...empty, displayName:'Nome personalizado',portrait:gif,banner:gif};
    assert.equal((await f.call('social','profile_save',alice,{...draft,plan:'pro'})).status,403);
    f.plan(alice,'plus');
    assert.equal((await f.call('social','profile_save',alice,draft)).status,200);
    await f.call('social','invite',alice,{targetId:bob}); await f.call('social','accept',bob,{targetId:alice});
    let own = (await f.call('social','sync',alice,{readOnly:true})).body.me;
    assert.equal(own.displayName,draft.displayName); assert.equal((await f.call('social','profile_get',alice,{})).body.profile.portrait,gif);
    let friend = (await f.call('social','sync',bob,{readOnly:true})).body.friends[0];
    assert.equal(friend.displayName,draft.displayName); assert.equal((await f.call('social','profile_get',bob,{targetId:alice})).body.profile.portrait,gif); assert.ok(JSON.stringify(friend).length < 4096);
    f.plan(alice,'plus',1);
    assert.equal((await f.call('social','profile_get',alice,{})).body.profile.portrait,'');
    friend = (await f.call('social','sync',bob,{readOnly:true})).body.friends[0];
    assert.notEqual(friend.avatarUrl,gif); assert.equal(friend.displayName,draft.displayName);
    assert.equal((await f.call('social','profile_save',alice,{...empty,displayName:draft.displayName,description:'Outra descrição'})).status,200);
    assert.equal(f.sqlite.prepare('SELECT portrait FROM social_public_profiles WHERE user_id = ?').get(alice).portrait,gif);
    f.plan(alice,'pro');
    own = (await f.call('social','profile_get',alice,{})).body.profile;
    assert.equal(own.portrait,gif); assert.equal(own.banner,gif); assert.equal(own.displayName,draft.displayName);
});
function fixture(t) {
    const sqlite = new DatabaseSync(':memory:');
    for (const file of readdirSync(new URL('../website/migrations/', import.meta.url)).filter(name => name.endsWith('.sql')).sort()) sqlite.exec(readFileSync(new URL(`../website/migrations/${file}`, import.meta.url), 'utf8'));
    for (const [id, token] of Object.entries(tokens)) sqlite.prepare('INSERT INTO social_users(id, username, token_hash) VALUES (?, ?, ?)').run(id, id === alice ? 'Alice' : 'Bob', createHash('sha256').update(token).digest('hex'));
    t.after(() => sqlite.close());
    const db = { prepare(sql) { const statement = sqlite.prepare(sql); const bound = (values = []) => ({ bind:(...next)=>bound(next), async first() {return statement.get(...values) || null;}, async all() {return {results:statement.all(...values)};}, async run() {return {meta:statement.run(...values)};}, execute() {return {meta:statement.run(...values)};} }); return bound(); }, async batch(statements) {sqlite.exec('BEGIN'); try {const results=statements.map(statement=>statement.execute());sqlite.exec('COMMIT');return results;} catch(error) {sqlite.exec('ROLLBACK');throw error;} } };
    const plan = (id, value, expires = Math.floor(Date.now()/1000) + 3600) => sqlite.prepare('INSERT INTO lux_memberships(social_id, plan, expires_at, revision) VALUES (?, ?, ?, 1) ON CONFLICT(social_id) DO UPDATE SET plan = excluded.plan, expires_at = excluded.expires_at').run(id, value, expires);
    const call = async (area, action, id = alice, body = {}, headers = {}) => {
        const response = await (area === 'social' ? social : commerce)({request:new Request(`https://luxmc.test/api/${area}/${action}`, {method:'POST',headers:{'Content-Type':'application/json',Authorization:`Bearer ${tokens[id]}`, ...headers},body:JSON.stringify(body)}), env:{SOCIAL_DB:db, OWNER_SOCIAL_ID:bob},params:{action},waitUntil(promise) {void promise.catch(()=>{});} });
        return {status:response.status,body:await response.json()};
    };
    return {sqlite, db, plan, call};
}
test('server enforces each profile tier and ignores forged plans and owner tags', async t => {
    const f = fixture(t);
    assert.equal((await f.call('social','profile_save',alice,{...empty,profileStyle:proStyle,plan:'pro',role:'owner',tags:['Dono']})).status,403);
    assert.equal((await f.call('social','profile_save',alice,{...empty,profileStyle:defaultProfileStyle})).status,200);
    f.plan(alice,'plus');
    const plusStyle = {accent:'mint',frame:'double',effect:'none',layout:'classic'};
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs:Array.from({length:16},(_,i)=>`Pack ${i}`),profileStyle:plusStyle})).status,200);
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:proStyle,plan:'pro'})).status,403);
    f.plan(alice,'pro');
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs:Array.from({length:24},(_,i)=>`Pack ${i}`),profileStyle:proStyle,role:'owner',tags:['Dono']})).status,200);
    const profile = (await f.call('social','profile_get',alice,{favouritesLimit:24})).body.profile;
    assert.equal(profile.role,'member'); assert.deepEqual(profile.tags,[]); assert.deepEqual(profile.profileStyle,proStyle); assert.equal(profile.packs.length,24);
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs:Array.from({length:25},(_,i)=>`Pack ${i}`),profileStyle:proStyle})).status,403);
});
test('site and launcher share the stored style without overwriting profile content or another account', async t => {
    const f = fixture(t); f.plan(alice,'pro');
    await f.call('social','profile_save',alice,{...empty,displayName:'Explorer',status:'Jogando juntos',description:'Minha bio',packs:['Cobblemon']});
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:proStyle,targetId:bob})).status,200);
    const response = await f.call('commerce','status'); assert.equal(response.status,200);
    const site = response.body;
    const launcher = (await f.call('social','profile_get',alice,{favouritesLimit:24})).body.profile;
    assert.deepEqual(site.profile.profileStyle,launcher.profileStyle); assert.equal(launcher.description,'Minha bio'); assert.deepEqual(launcher.packs,['Cobblemon']);
    assert.equal(f.sqlite.prepare('SELECT profile_style FROM social_public_profiles WHERE user_id = ?').get(bob),undefined);
    assert.equal(site.checkout.enabled,false);
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:proStyle},{Authorization:''})).status,401);
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:proStyle},{Authorization:'',Cookie:`__Host-luxmc-commerce=${tokens[alice]}`})).status,403);
});
test('expiry hides premium effects, preserves content and restores the signature on renewal', async t => {
    const f = fixture(t); f.plan(alice,'pro');
    const packs = Array.from({length:24},(_,i)=>`Pack ${i}`);
    await f.call('social','profile_save',alice,{...empty,packs,profileStyle:proStyle});
    await f.call('social','invite',alice,{targetId:bob}); await f.call('social','accept',bob,{targetId:alice});
    f.plan(alice,'pro',1);
    let own = (await f.call('social','profile_get',alice,{favouritesLimit:24})).body.profile;
    assert.deepEqual(own.profileStyle,defaultProfileStyle); assert.deepEqual(own.savedProfileStyle,proStyle); assert.deepEqual(own.packs,packs);
    const friend = (await f.call('social','profile_get',bob,{targetId:alice,favouritesLimit:24})).body.profile;
    assert.equal(friend.savedProfileStyle,undefined); assert.deepEqual(friend.profileStyle,defaultProfileStyle);
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs,profileStyle:defaultProfileStyle,description:'Atualizado no Free'})).status,200);
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:defaultProfileStyle})).status,200);
    assert.deepEqual(JSON.parse(f.sqlite.prepare('SELECT profile_style FROM social_public_profiles WHERE user_id = ?').get(alice).profile_style),proStyle);
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs:[...packs.slice(1),'Novo'],profileStyle:defaultProfileStyle})).status,403);
    f.plan(alice,'pro'); own = (await f.call('social','profile_get',alice,{favouritesLimit:24})).body.profile;
    assert.deepEqual(own.profileStyle,proStyle); assert.equal(own.description,'Atualizado no Free');
    const snapshot = (await f.call('social','sync',bob,{readOnly:true})).body.friends[0];
    assert.deepEqual(snapshot.profileStyle,proStyle);
});
test('legacy clients keep extra favourites and style; malicious style values are rejected', async t => {
    const f = fixture(t); f.plan(alice,'pro');
    const packs = Array.from({length:24},(_,i)=>`Pack ${i}`);
    await f.call('social','profile_save',alice,{...empty,packs,profileStyle:proStyle});
    assert.equal((await f.call('social','profile_save',alice,{...empty,packs:packs.slice(0,8),description:'Cliente anterior'})).status,200);
    const profile = (await f.call('social','profile_get',alice,{favouritesLimit:24})).body.profile; assert.deepEqual(profile.profileStyle,proStyle); assert.deepEqual(profile.packs,packs);
    const legacy = (await f.call('social','profile_get',alice,{})).body.profile;assert.deepEqual(legacy.packs,packs.slice(0,8));
    for (const value of [{...proStyle,accent:'url(javascript:alert(1))'},{...proStyle,frame:'owner'},{...proStyle,css:'position:fixed'},{...proStyle,layout:null},null,[]]) {
        assert.equal(parseProfileStyle(value),null);
        assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:value})).status,400);
    }
    assert.deepEqual(Object.values(profileBenefits).map(item=>item.favourites),[8,16,24]);
});
test('a concurrent removal cannot recreate community data through the plan page', async t => {
    const f = fixture(t); f.plan(alice,'pro');
    await f.call('social','profile_save',alice,{...empty,profileStyle:proStyle});
    const prepare = f.db.prepare.bind(f.db);
    f.db.prepare = sql => {
        const statement = prepare(sql);
        if (sql === 'SELECT profile_style FROM social_public_profiles WHERE user_id = ?') {
            const bind = statement.bind;
            statement.bind = (...values) => {
                const bound = bind(...values); const first = bound.first;
                bound.first = async () => {
                    const row = await first();
                    f.sqlite.prepare('UPDATE social_users SET removed_at = 1 WHERE id = ?').run(alice);
                    f.sqlite.prepare('DELETE FROM social_public_profiles WHERE user_id = ?').run(alice);
                    return row;
                };
                return bound;
            };
        }
        return statement;
    };
    assert.equal((await f.call('commerce','profile_style',alice,{profileStyle:proStyle})).status,404);
    assert.equal(f.sqlite.prepare('SELECT COUNT(*) AS count FROM social_public_profiles WHERE user_id = ?').get(alice).count,0);
});
