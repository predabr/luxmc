import test from "node:test";
import assert from "node:assert/strict";
import { DatabaseSync } from "node:sqlite";
import { readFileSync } from "node:fs";
import { onRequest } from "../website/functions/api/account/[action].js";
import { onRequest as social } from "../website/functions/api/social/[action].js";

function fixture() {
  const sqlite = new DatabaseSync(":memory:");
  sqlite.exec("PRAGMA foreign_keys=ON");
  for (const migration of ["0001_social.sql", "0002_accounts.sql"]) sqlite.exec(readFileSync(new URL(`../website/migrations/${migration}`, import.meta.url), "utf8"));
  const db = {
    prepare(sql) { const statement = sqlite.prepare(sql); return { bind(...values) { return { async first() { return statement.get(...values) || null; }, async all() { return { results: statement.all(...values) }; }, async run() { return { meta: statement.run(...values) }; }, execute() { return { meta: statement.run(...values) }; } }; } }; },
    async batch(statements) { sqlite.exec("BEGIN"); try { const results = statements.map(statement => statement.execute()); sqlite.exec("COMMIT"); return results; } catch (error) { sqlite.exec("ROLLBACK"); throw error; } }
  };
  const env = { SOCIAL_DB: db, AUTH_PEPPER: "test-only-pepper-not-for-production-123456789" };
  async function call(action, body = {}, authorization = {}, handler = onRequest) {
    const response = await handler({ request: new Request(`https://luxmc.test/api/account/${action}`, { method: "POST", headers: { "Content-Type": "application/json", ...authorization }, body: JSON.stringify(body) }), env, params: { action }, waitUntil() {} });
    return { status: response.status, value: await response.json(), cookie: response.headers.get("Set-Cookie") };
  }
  return { sqlite, env, call };
}
const password = "Test-Passphrase-123!";

test("web registration and native login share identity without exposing sessions to browser JS", async () => {
  const { sqlite, call } = fixture();
  try {
    const signup = await call("register", { username: "Pedrin1234", password });
    assert.equal(signup.status, 200);
    assert.equal(signup.value.token, undefined);
    assert.match(signup.cookie, /HttpOnly; Secure; SameSite=Strict/);
    assert.equal(signup.value.recoveryCode.length, 64);
    assert.equal(signup.value.account.password_hash, undefined);
    const stored = sqlite.prepare("SELECT * FROM lux_accounts").get();
    assert.notEqual(stored.password_hash, password);
    assert.notEqual(stored.recovery_hash, signup.value.recoveryCode);
    assert.equal((await call("register", { username: "pedrin1234", password })).status, 409);
    assert.equal((await call("login", { username: "Pedrin1234", password: "Wrong-password!" })).status, 401);
    const login = await call("login", { username: "pedrin1234", password, client: "launcher" });
    assert.equal(login.status, 200);
    assert.equal(login.value.account.id, signup.value.account.id);
    assert.equal(login.value.account.username, "Pedrin1234");
    assert.equal(login.value.token.length, 64);
    const web = { Cookie: signup.cookie.split(";")[0] };
    const native = { Authorization: `Bearer ${login.value.token}` };
    const saved = await call("sync", { preferences: { theme: "default-light", language: "pt-BR" }, revision: 0 }, web);
    assert.equal(saved.status, 200);
    assert.equal((await call("sync", {}, native)).value.account.preferences.theme, "default-light");
    assert.equal((await call("sync", { preferences: { theme: "default-dark" }, revision: 0 }, native)).status, 409);
    assert.equal((await call("register", { username: "Imposter" }, native, social)).value.me.username, "Pedrin1234");
    await call("logout", {}, native);
    assert.equal((await call("me", {}, native)).status, 401);
    assert.equal((await call("me", {}, web)).status, 200);
  } finally { sqlite.close(); }
});

test("password changes and one-time recovery revoke other sessions", async () => {
  const { sqlite, call } = fixture();
  try {
    const signup = await call("register", { username: "RecoveryUser", password });
    const cookie = { Cookie: signup.cookie.split(";")[0] };
    const login = await call("login", { username: "RecoveryUser", password, client: "launcher" });
    const native = { Authorization: `Bearer ${login.value.token}` };
    assert.equal((await call("password", { currentPassword: "wrong", password: "Updated-password" }, cookie)).status, 401);
    assert.equal((await call("password", { currentPassword: password, password: "Updated-password" }, cookie)).status, 200);
    assert.equal((await call("me", {}, native)).status, 401);
    assert.equal((await call("login", { username: "RecoveryUser", password })).status, 401);
    const recovered = await call("recover", { username: "RecoveryUser", password, recoveryCode: signup.value.recoveryCode });
    assert.equal(recovered.status, 200);
    assert.notEqual(recovered.value.recoveryCode, signup.value.recoveryCode);
    assert.equal((await call("me", {}, cookie)).status, 401);
    assert.equal((await call("recover", { username: "RecoveryUser", password, recoveryCode: signup.value.recoveryCode })).status, 401);
  } finally { sqlite.close(); }
});

test("account endpoint rejects CSRF, unsafe payloads and missing server secret", async () => {
  const { sqlite, env, call } = fixture();
  try {
    assert.equal((await call("register", { username: "TestUser", password }, { Origin: "https://evil.test" })).status, 403);
    assert.equal((await call("register", { username: "TestUser", password }, { "Sec-Fetch-Site": "cross-site" })).status, 403);
    assert.equal((await call("register", { username: "TestUser", password: "1234123" })).status, 400);
    assert.equal((await call("register", { username: "TestUser", password, padding: "x".repeat(9000) })).status, 413);
    const signup = await call("register", { username: "TestUser", password });
    const cookie = { Cookie: signup.cookie.split(";")[0] };
    assert.equal((await call("sync", { revision: 0, preferences: { theme: "<script>" } }, cookie)).status, 400);
    env.AUTH_PEPPER = "";
    assert.equal((await call("login", { username: "TestUser", password })).status, 503);
  } finally { sqlite.close(); }
});

test("reading friends from website does not overwrite launcher activity", async () => {
  const { sqlite, call } = fixture();
  try {
    const signup = await call("register", { username: "PlayerOne", password });
    const headers = { Cookie: signup.cookie.split(";")[0] };
    await call("sync", { status: "in_game", instanceName: "Adventure", mcVersion: "1.20.1", serverHost: "192.168.1.2", serverPort: 25565 }, headers, social);
    await call("sync", { readOnly: true }, headers, social);
    const row = sqlite.prepare("SELECT activity, server_host FROM social_users WHERE id = ?").get(signup.value.account.socialId);
    assert.equal(row.activity, "in_game");
    assert.equal(row.server_host, "192.168.1.2");
  } finally { sqlite.close(); }
});
