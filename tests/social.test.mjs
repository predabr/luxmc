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
  const db = { prepare(sql) {
    const statement = sqlite.prepare(sql);
    return { bind(...values) {
      return { async first() { return statement.get(...values) || null; }, async all() { return { results: statement.all(...values) }; }, async run() { return { meta: statement.run(...values) }; } };
    } };
  } };
  async function request(action, token, body) {
    const response = await onRequest({ request: new Request(`https://luxmc.test/api/social/${action}`, { method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` }, body: JSON.stringify(body) }), env: { SOCIAL_DB: db }, params: { action }, waitUntil() {} });
    return { status: response.status, body: await response.json() };
  }
  return { sqlite, request, db };
}

const aliceToken = "a".repeat(64);
const bobToken = "b".repeat(64);
const eveToken = "c".repeat(64);

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
