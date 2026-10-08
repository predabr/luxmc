import test from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { readFileSync } from "node:fs";
import { onRequest, validServer } from "../website/functions/api/social/[action].js";
import { assetFor, safeAssetUrl } from "../website/lib/releases.js";

function fixture() {
  const sqlite = new DatabaseSync(":memory:");
  sqlite.exec(readFileSync(new URL("../website/migrations/0001_social.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0002_accounts.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0003_mesh_social.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0004_social_stream.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0005_social_avatars.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0008_public_profiles.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0009_profile_identity.sql", import.meta.url), "utf8"));
  sqlite.exec(readFileSync(new URL("../website/migrations/0010_pack_collections.sql", import.meta.url), "utf8"));
  const db = { prepare(sql) {
    const statement = sqlite.prepare(sql);
    return { bind(...values) {
      return { async first() { return statement.get(...values) || null; }, async all() { return { results: statement.all(...values) }; }, async run() { return { meta: statement.run(...values) }; } };
    } };
  } };
  async function request(action, token, body, streamUrl, ownerId) {
    const response = await onRequest({ request: new Request(`https://luxmc.test/api/social/${action}`, { method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` }, body: JSON.stringify(body) }), env: { SOCIAL_DB: db, SOCIAL_STREAM_URL: streamUrl, OWNER_SOCIAL_ID: ownerId }, params: { action }, waitUntil() {} });
    return { status: response.status, body: await response.json() };
  }
  return { sqlite, request, db };
}

const aliceToken = "a".repeat(64);
const bobToken = "b".repeat(64);
const eveToken = "c".repeat(64);

test("owner role follows the authenticated configured identity and cannot be claimed by nickname or saved fields", async () => {
  const { sqlite, request } = fixture();
  try {
    const owner = (await request('register', aliceToken, {username:'Spect3rBW'})).body.me;
    await request('register', bobToken, {username:'Spect3rBW'});
    const save = await request('profile_save', bobToken, {role:'owner',displayName:'OWNER',description:'',banner:'',portrait:'',packs:[]},undefined,owner.id);
    assert.equal(save.status,200);
    assert.equal((await request('profile_get',bobToken,{},undefined,owner.id)).body.profile.role,'member');
    assert.equal((await request('profile_get',aliceToken,{},undefined,owner.id)).body.profile.role,'owner');
    assert.equal((await request('profile_get',aliceToken,{})).body.profile.role,'member');
    assert.equal((await request('profile_get',bobToken,{targetId:owner.id},undefined,owner.id)).status,403);
  } finally { sqlite.close(); }
});

test("invites require recipient consent and hide presence until accepted", async () => {
  const { sqlite, request } = fixture();
  try {
    const alice = (await request("register", aliceToken, { username: "Alice" })).body.me;
    const bob = (await request("register", bobToken, { username: "Bob" })).body.me;
    await request("register", eveToken, { username: "Eve" });
    assert.equal((await request("sync", "d".repeat(64), {})).status, 401);
    assert.equal((await request("invite", aliceToken, { targetId: bob.id })).status, 200);
    await request("sync", bobToken, { status: "in_game", instanceName: "World", mcVersion: "1.20.1", serverHost: "203.0.113.1", serverPort: 25565 });
    const pending = (await request("sync", aliceToken, {})).body.friends[0];
    assert.equal(pending.serverIp, null);
    assert.equal(pending.incoming, false);
    const received = (await request("sync", bobToken, { readOnly: true })).body.friends[0];
    assert.equal(received.id, alice.id);
    assert.equal(received.status, "pending");
    assert.equal(received.incoming, true);
    assert.equal((await request("accept", aliceToken, { targetId: bob.id })).status, 404);
    assert.equal((await request("accept", eveToken, { targetId: alice.id })).status, 404);
    assert.equal((await request("accept", bobToken, { targetId: alice.id })).status, 200);
    assert.equal((await request("sync", aliceToken, {})).body.friends[0].serverIp, "203.0.113.1");
    assert.deepEqual((await request("sync", eveToken, {})).body.friends, []);
    sqlite.prepare("UPDATE social_users SET last_seen = 1 WHERE id = ?").run(bob.id);
    const stale = (await request("sync", aliceToken, {})).body.friends[0];
    assert.equal(stale.status, "offline");
    assert.equal(stale.serverIp, null);
    await request("remove", bobToken, { targetId: alice.id });
    assert.deepEqual((await request("sync", aliceToken, {})).body.friends, []);
  } finally { sqlite.close(); }
});

test("search handles nicknames safely and limits validate content", async () => {
  const { sqlite, request } = fixture();
  try {
    await request("register", aliceToken, { username: "Alice_One" });
    await request("register", bobToken, { username: "Bob" });
    assert.equal((await request("search", bobToken, { query: "Alice_" })).body.users.length, 1);
    assert.deepEqual((await request("search", bobToken, { query: "%' OR 1=1" })).body.users, []);
    assert.equal((await request("register", eveToken, { username: "<script>" })).status, 400);
    assert.equal((await request("sync", aliceToken, { activity: "x".repeat(5000) })).status, 413);
    assert.equal((await request("invite", aliceToken, { targetId: "../evil" })).status, 400);
    assert.equal(validServer("host/--evil", 25565), false);
    assert.equal(validServer("example.org", 65536), false);
  } finally { sqlite.close(); }
});

test("downloads select installers and reject off-repository redirects", () => {
  assert.equal(safeAssetUrl("https://github.com.evil.test/predabr/luxmc/releases/download/v1/a.exe"), false);
  assert.equal(safeAssetUrl("https://github.com/other/project/releases/download/v1/a.exe"), false);
  const asset = name => ({ name, browser_download_url: `https://github.com/predabr/luxmc/releases/download/v1/${name}` });
  assert.equal(assetFor([asset("arm64.AppImage"), asset("x86_64.AppImage"), asset("app.exe.sig")], "linux").name, "x86_64.AppImage");
    assert.equal(assetFor([asset("app.exe.sig")], "windows"), null);
    assert.equal(assetFor([asset("Luxmc-x64.exe"), asset("Lux MC Launcher.exe")], "windows").name, "Lux MC Launcher.exe");
    assert.equal(assetFor([asset("Luxmc_3.0.0_x64-setup.exe"), asset("Lux MC Launcher.exe")], "windows").name, "Lux MC Launcher.exe");
    assert.equal(assetFor([asset("Luxmc_3.0.0_x64-setup.exe"), asset("Lux.MC.Launcher.exe")], "windows").name, "Lux.MC.Launcher.exe");
  assert.equal(assetFor([asset("Luxmc.deb")], "debian").name, "Luxmc.deb");
  assert.equal(assetFor([asset("Luxmc.rpm")], "fedora").name, "Luxmc.rpm");
  assert.equal(assetFor([asset("luxmc.pkg.tar.zst")], "arch").name, "luxmc.pkg.tar.zst");
  assert.equal(assetFor([asset("Luxmc.dmg")], "macos").name, "Luxmc.dmg");
});


test("concurrent registration is idempotent and tokens stay private", async () => {
  const { sqlite, request } = fixture();
  try {
    const results = await Promise.all(Array.from({ length: 4 }, () => request("register", aliceToken, { username: "Alice" })));
    assert.ok(results.every(result => result.status === 200));
    assert.equal(new Set(results.map(result => result.body.me.id)).size, 1);
    assert.equal(sqlite.prepare("SELECT count(*) AS count FROM social_users").get().count, 1);
    assert.ok(results.every(result => !JSON.stringify(result.body).includes(aliceToken)));
  } finally { sqlite.close(); }
});

test("request validation fails closed before exposing data", async () => {
  const { sqlite, db } = fixture();
  try {
    const send = (body, extra = {}) => onRequest({ request: new Request("https://luxmc.test/api/social/sync", { method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${aliceToken}`, ...extra }, body }), env: { SOCIAL_DB: db }, params: { action: "sync" }, waitUntil() {} });
    assert.equal((await send("{" )).status, 400);
    assert.equal((await send("[]")).status, 400);
    assert.equal((await send("{}", { Authorization: "Bearer invalid" })).status, 401);
    assert.equal((await send("{}", { "Content-Type": "text/plain" })).status, 415);
    assert.equal((await send(JSON.stringify({ text: "é".repeat(2100) }))).status, 413);
    assert.equal((await send("{}", { "Content-Length": "5000" })).status, 413);
    for (const host of ["foo..bar", "-host.test", "host-.test", "[::::]", "a".repeat(64) + ".test", "host/evil"]) assert.equal(validServer(host, 25565), false, host);
    for (const host of ["localhost", "192.168.1.10", "game.example.org", "[::1]"]) assert.equal(validServer(host, 25565), true, host);
  } finally { sqlite.close(); }
});

test("rate limiting rejects excess requests without leaking identities", async () => {
  const { sqlite, request } = fixture();
  try {
    await request("register", aliceToken, { username: "Alice" });
    let response;
    for (let i = 0; i < 121; i++) response = await request("sync", aliceToken, {});
    assert.equal(response.status, 429);
    assert.equal(response.body.me, undefined);
  } finally { sqlite.close(); }
});


test("blocked users cannot invite or join a private mesh room", async () => {
  const { sqlite, request } = fixture();
  try {
    const alice = (await request("register", aliceToken, { username: "Alice" })).body.me;
    const bob = (await request("register", bobToken, { username: "Bob" })).body.me;
    await request("invite", aliceToken, { targetId: bob.id });
    await request("accept", bobToken, { targetId: alice.id });
    const room = await request("room_create", aliceToken, { host: "100.64.0.1", port: 25565 });
    assert.equal(room.status, 200);
    assert.match(room.body.code, /^\d{6}$/);
    assert.equal((await request("room_join", bobToken, { code: room.body.code })).body.host, "100.64.0.1");
    await request("block", aliceToken, { targetId: bob.id });
    assert.equal((await request("invite", bobToken, { targetId: alice.id })).status, 403);
    assert.equal((await request("room_join", bobToken, { code: room.body.code })).status, 403);
    assert.deepEqual((await request("sync", bobToken, {})).body.friends, []);
    await request("room_close", aliceToken, {});
    assert.equal((await request("room_join", aliceToken, { code: room.body.code })).status, 404);
  } finally { sqlite.close(); }
});


test("websocket tickets are short-lived and only their hash is stored", async () => {
  const { sqlite, request } = fixture();
  try {
    await request("register", aliceToken, { username: "Alice" });
    assert.equal((await request("stream_ticket", aliceToken, {})).body.url, null);
    const response = await request("stream_ticket", aliceToken, {}, "wss://stream.example/connect");
    assert.equal(response.status, 200);
    const ticket = new URL(response.body.url).searchParams.get("ticket");
    assert.equal(ticket.length, 72);
    const stored = sqlite.prepare("SELECT * FROM social_stream_tickets").get();
    assert.notEqual(stored.token_hash, ticket);
    assert.equal(stored.token_hash.length, 64);
    assert.ok(stored.expires_at <= Date.now()/1000 + 30);
    assert.equal((await request("stream_ticket", aliceToken, {}, "ws://stream.example/connect")).status, 503);
  } finally { sqlite.close(); }
});


test("custom skin avatars round-trip to the portal and friend snapshots", async () => {
  const { sqlite, request } = fixture();
  try {
    const avatar = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aX1cAAAAASUVORK5CYII=";
    const alice = (await request("register", aliceToken, { username: "Alice" })).body.me;
    const bob = (await request("register", bobToken, { username: "Bob" })).body.me;
    await request("invite", aliceToken, { targetId: bob.id });
    await request("accept", bobToken, { targetId: alice.id });
    const updated = await request("sync", bobToken, { status: "online", avatarUrl: avatar });
    assert.equal(updated.body.me.avatarUrl, avatar);
    assert.equal((await request("sync", aliceToken, {})).body.friends[0].avatarUrl, avatar);
    assert.equal((await request("search", aliceToken, { query: "Bob" })).body.users[0].avatarUrl, avatar);
    await request("sync", bobToken, { avatarUrl: "data:image/svg+xml,<svg onload=alert(1)>" });
    assert.equal((await request("sync", aliceToken, {})).body.friends[0].avatarUrl, null);
  } finally { sqlite.close(); }
});


test("duplicate nicknames remain distinct and friend codes select the recipient", async () => {
  const { sqlite, request } = fixture();
  try {
    const first = (await request("register", aliceToken, { username: "SameNick" })).body.me;
    const second = (await request("register", bobToken, { username: "SameNick" })).body.me;
    await request("register", eveToken, { username: "Sender" });
    assert.equal((await request("search", eveToken, { query: "SameNick" })).body.users.length, 2);
    const exact = await request("search", eveToken, { query: `SameNick#${second.id.slice(0,8)}` });
    assert.equal(exact.body.users.length, 1);
    assert.equal(exact.body.users[0].id, second.id);
    await request("invite", eveToken, { targetId: second.id });
    assert.deepEqual((await request("sync", aliceToken, {})).body.friends, []);
    assert.equal((await request("sync", bobToken, {})).body.friends[0].incoming, true);
    assert.notEqual(first.id, second.id);
  } finally { sqlite.close(); }
});

test("public profiles persist offline and require an accepted friendship", async () => {
  const {sqlite,request} = fixture();
  try {
    const a = 'a'.repeat(64), b = 'b'.repeat(64), stranger = 'c'.repeat(64);
    const alice = (await request('register',a,{username:'Alice'})).body.me;
    const bob = (await request('register',b,{username:'Bob'})).body.me;
    await request('register',stranger,{username:'Stranger'});
    const saved = await request('profile_save',a,{displayName:'Explorer',status:'Building today',description:'Building worlds',banner:'',portrait:'',packs:['Homestead'],collections:[{id:'group-one',title:'Co-op',description:'Play together',entries:[{source:'modrinth',projectId:'project123',versionId:'exact123',name:'Homestead'}]}]});
    assert.equal(saved.status,200);
    assert.equal((await request('profile_get',b,{targetId:alice.id})).status,403);
    await request('invite',a,{targetId:bob.id}); await request('accept',b,{targetId:alice.id});
    assert.deepEqual((await request('profile_get',b,{targetId:alice.id})).body.profile.packs,['Homestead']);
    const identity = (await request('profile_get',b,{targetId:alice.id})).body.profile;
    assert.equal(identity.displayName,'Explorer');
    assert.equal(identity.status,'Building today');
    assert.equal((await request('profile_save',a,{description:'Legacy update',banner:'',portrait:'',packs:['Homestead']})).status,200);
    const preserved = (await request('profile_get',b,{targetId:alice.id})).body.profile;
    assert.equal(preserved.collections[0].entries[0].versionId,'exact123');
    assert.equal(preserved.displayName,'Explorer');
    assert.equal(preserved.status,'Building today');
    assert.equal((await request('profile_get',stranger,{targetId:alice.id})).status,403);
    await request('block',b,{targetId:alice.id});
    assert.equal((await request('profile_get',b,{targetId:alice.id})).status,403);
    assert.equal((await request('profile_save',a,{description:'ok',banner:'javascript:alert(1)',portrait:'',packs:[]})).status,400);
  } finally {sqlite.close();}
});
