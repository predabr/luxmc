import assert from "node:assert/strict";
import { randomBytes, createHash } from "node:crypto";
import { writeFileSync } from "node:fs";

const base = "https://luxmc-r92.pages.dev";
const users = [];
let socket;
let snapshots = [];
async function call(action, token, body = {}) {
    const response = await fetch(`${base}/api/social/${action}`, {
        method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(body), signal: AbortSignal.timeout(15000)
    });
    const result = await response.json();
    assert.equal(response.status, 200, `${action}: ${result.error || response.status}`);
    return result;
}
async function waitSnapshot(predicate) {
    const deadline = Date.now() + 15000;
    while (Date.now() < deadline) {
        const value = snapshots.find(predicate);
        if (value) return value;
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw Error("A atualização da solicitação não chegou pelo serviço ao vivo");
}
try {
    for (let index = 0; index < 2; index++) {
        const token = randomBytes(32).toString("hex");
        const name = `LuxQA${randomBytes(4).toString("hex")}`;
        const { me } = await call("register", token, { username: name });
        users.push({ token, name, id: me.id });
        await call("sync", token, { status: "online" });
    }
    const [sender, recipient] = users;
    const result = await call("search", sender.token, { query: `${recipient.name}#${recipient.id.slice(0, 8)}` });
    assert.equal(result.users[0].id, recipient.id);
    const ticket = await call("stream_ticket", recipient.token);
    assert.ok(ticket.url, "O portal precisa fornecer o endereço das atualizações ao vivo");
    socket = new WebSocket(ticket.url);
    socket.addEventListener("message", event => snapshots.push(JSON.parse(String(event.data))));
    await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(Error("Conexão ao serviço ao vivo não abriu")), 15000);
        socket.addEventListener("open", () => { clearTimeout(timeout); resolve(); }, { once: true });
        socket.addEventListener("error", () => { clearTimeout(timeout); reject(Error("Erro de conexão ao serviço ao vivo")); }, { once: true });
    });
    await waitSnapshot(value => value.friends.length === 0);
    snapshots = [];
    await call("invite", sender.token, { targetId: recipient.id });
    const delivered = await waitSnapshot(value => value.friends.some(friend => friend.id === sender.id && friend.incoming && friend.status === "pending"));
    assert.equal(delivered.friends[0].incoming, true);
    await call("accept", recipient.token, { targetId: sender.id });
    assert.equal((await call("sync", sender.token, { readOnly: true })).friends[0].status, "online");
    assert.equal((await call("sync", recipient.token, { readOnly: true })).friends[0].status, "online");
    await call("remove", sender.token, { targetId: recipient.id });
    console.log(JSON.stringify({ registration: true, friendCode: true, liveIncomingRequest: true, recipientAcceptance: true, bothListsUpdated: true, testFriendshipRemoved: true }));
} finally {
    socket?.close();
    const cleanup = users.map(user => {
        assert.match(user.id, /^[a-f0-9-]{36}$/);
        const hash = createHash("sha256").update(user.token).digest("hex");
        return `DELETE FROM social_relationships WHERE sender='${user.id}' OR recipient='${user.id}';\nDELETE FROM social_stream_tickets WHERE user_id='${user.id}';\nDELETE FROM social_blocks WHERE owner='${user.id}' OR target='${user.id}';\nDELETE FROM social_users WHERE id='${user.id}' AND token_hash='${hash}' AND username='${user.name}';`;
    }).join("\n");
    writeFileSync("docs/validation/multiplayer-live-fixture-cleanup.sql", cleanup, "utf8");
}
