import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";

const base = process.argv[2] || "http://127.0.0.1:8794";
assert.ok(new URL(base).hostname === "127.0.0.1" || new URL(base).hostname === "localhost", "This smoke test only targets a local test database");
const client = () => {
    const token = randomBytes(32).toString("hex");
    return async (action, body = {}) => {
        const response = await fetch(`${base}/api/social/${action}`, { method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` }, body: JSON.stringify(body) });
        const result = await response.json();
        assert.equal(response.status, 200, `${action}: ${JSON.stringify(result)}`);
        return result;
    };
};
const alice = client();
const bob = client();
const aliceId = (await alice("register", { username: "HttpAlice" })).me.id;
const bobId = (await bob("register", { username: "HttpBob" })).me.id;
assert.ok((await alice("search", { query: "HttpBob" })).users.some(user => user.id === bobId));
await alice("invite", { targetId: bobId });
await bob("sync", { status: "in_game", mcVersion: "1.20.1", instanceName: "HTTP world", loader: "fabric", serverHost: "192.168.1.20", serverPort: 25565 });
const before = (await alice("sync")).friends[0];
assert.equal(before.status, "pending");
assert.equal(before.serverIp, null);
assert.equal((await bob("sync")).friends[0].incoming, true);
await bob("accept", { targetId: aliceId });
await bob("sync", { status: "in_game", mcVersion: "1.20.1", instanceName: "HTTP world", loader: "fabric", serverHost: "192.168.1.20", serverPort: 25565 });
const friend = (await alice("sync")).friends[0];
assert.equal(friend.status, "in_game");
assert.equal(friend.serverIp, "192.168.1.20");
assert.equal(friend.serverPort, 25565);
assert.equal(friend.loader, "fabric");
await bob("sync", { status: "offline" });
assert.equal((await alice("sync")).friends[0].status, "offline");
await alice("remove", { targetId: bobId });
assert.deepEqual((await bob("sync")).friends, []);
console.log("PASS two independent HTTP clients: register, search, invitation consent, activity, join address, offline presence and removal on local Pages + D1");
