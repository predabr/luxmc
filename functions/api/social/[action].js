import { authenticate, sameOrigin, sessionToken } from "../../../lib/accounts.js";
const headers = { "Content-Type": "application/json", "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff" };
const respond = (data, status = 200) => new Response(JSON.stringify(data), { status, headers });
const digest = async value => Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value))), b => b.toString(16).padStart(2, "0")).join("");
const nickname = value => typeof value === "string" && /^[A-Za-z0-9_]{3,16}$/.test(value);
const shortText = (value, max) => typeof value === "string" ? value.slice(0, max) : "";

export function validServer(host, port) {
  if (typeof host !== "string" || host.length > 253 || !Number.isInteger(port) || port < 1 || port > 65535) return false;
  if (host.startsWith("[")) {
    try { return new URL(`http://${host}:${port}`).hostname === host.toLowerCase(); } catch { return false; }
  }
  return host.split(".").every(label => /^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$/.test(label));
}

async function limited(db, key, now, window, max) {
  const bucket = `${key}:${Math.floor(now / window)}`;
  const row = await db.prepare("INSERT INTO social_limits(bucket, count, expires_at) VALUES (?, 1, ?) ON CONFLICT(bucket) DO UPDATE SET count = count + 1 RETURNING count")
    .bind(bucket, now + window).first();
  return row.count > max;
}

export async function onRequest({ request, env, params, waitUntil }) {
  if (!sameOrigin(request)) return respond({ error: "Origem não permitida" }, 403);
  if (request.method !== "POST") return respond({ error: "Method not allowed" }, 405);
  if (!env.SOCIAL_DB) return respond({ error: "O serviço social ainda não foi configurado no portal." }, 503);
  if (!request.headers.get("Content-Type")?.startsWith("application/json")) return respond({ error: "JSON required" }, 415);
  if (Number(request.headers.get("Content-Length")) > 4096) return respond({ error: "Request too large" }, 413);
  const action = params.action;
  if (!["register", "sync", "search", "invite", "accept", "remove"].includes(action)) return respond({ error: "Unknown action" }, 404);
  const bearer = sessionToken(request);
  if (!bearer) return respond({ error: "Authentication required" }, 401);
  const db = env.SOCIAL_DB;
  const now = Math.floor(Date.now() / 1000);
  try {
    const reader = request.body?.getReader();
    let raw = "";
    let byteCount = 0;
    if (reader) {
      const decoder = new TextDecoder();
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        byteCount += value.byteLength;
        if (byteCount > 4096) { await reader.cancel(); return respond({ error: "Request too large" }, 413); }
        raw += decoder.decode(value, { stream: true });
      }
      raw += decoder.decode();
    }
    let body;
    try { body = JSON.parse(raw); } catch { return respond({ error: "Invalid JSON" }, 400); }
    if (!body || typeof body !== "object" || Array.isArray(body)) return respond({ error: "Invalid body" }, 400);
    const tokenHash = await digest(bearer);
    const ipHash = await digest(request.headers.get("CF-Connecting-IP") || "local");
    if (await limited(db, `ip:${ipHash}`, now, 60, 120)) return respond({ error: "Tente novamente em um minuto." }, 429);
    if (Math.random() < 0.01) waitUntil(db.prepare("DELETE FROM social_limits WHERE expires_at < ?").bind(now).run());
    const registered = await authenticate(request, db);
    if (!registered && (body.registeredAccount === true || request.headers.has("Cookie"))) return respond({ error: "Sua sessão expirou. Entre novamente." }, 401);
    let me = registered ? { id: registered.social_id, username: registered.username } : await db.prepare("SELECT id, username FROM social_users WHERE token_hash = ?").bind(tokenHash).first();
    if (action === "register") {
      if (registered) return respond({ me });
      if (!nickname(body.username)) return respond({ error: "Nickname inválido (3–16 letras, números ou _)." }, 400);
      if (!me) {
        if (await limited(db, `register:${ipHash}`, now, 3600, 5)) return respond({ error: "Limite de novos perfis atingido." }, 429);
        const id = crypto.randomUUID();
        await db.prepare("INSERT INTO social_users(id, token_hash, username) VALUES (?, ?, ?) ON CONFLICT(token_hash) DO NOTHING").bind(id, tokenHash, body.username).run();
        me = await db.prepare("SELECT id, username FROM social_users WHERE token_hash = ?").bind(tokenHash).first();
      }
      return respond({ me });
    }
    if (!me) return respond({ error: "Perfil social não registrado." }, 401);
    if (action === "search") {
      const query = shortText(body.query, 64).trim();
      if (query.length < 3 || !/^[A-Za-z0-9_#-]+$/.test(query)) return respond({ users: [] });
      const [name, code] = query.split("#");
      const escaped = name.replace(/_/g, "!_");
      const { results } = await db.prepare("SELECT id, username FROM social_users WHERE username LIKE ? ESCAPE '!' COLLATE NOCASE AND id != ? AND (? = '' OR id LIKE ?) LIMIT 8")
        .bind(`${escaped}%`, me.id, code || "", `${code || ""}%`).all();
      return respond({ users: results });
    }
    if (action === "sync") {
      const activity = ["online", "in_game", "offline"].includes(body.status) ? body.status : "online";
      const server = activity === "in_game" && validServer(body.serverHost, body.serverPort);
      if (body.readOnly !== true) await db.prepare("UPDATE social_users SET last_seen = ?, activity = ?, instance_name = ?, mc_version = ?, loader = ?, server_host = ?, server_port = ? WHERE id = ?")
        .bind(activity === "offline" ? 0 : now, activity, shortText(body.instanceName, 80), shortText(body.mcVersion, 40), shortText(body.loader, 20), server ? body.serverHost : null, server ? body.serverPort : null, me.id).run();
      const { results } = await db.prepare("SELECT u.id, u.username, u.last_seen, u.activity, u.instance_name, u.mc_version, u.loader, u.server_host, u.server_port, r.accepted, r.recipient FROM social_relationships r JOIN social_users u ON u.id = CASE WHEN r.sender = ? THEN r.recipient ELSE r.sender END WHERE r.sender = ? OR r.recipient = ? LIMIT 200")
        .bind(me.id, me.id, me.id).all();
      const friends = results.map(row => {
        const online = row.accepted === 1 && row.last_seen > now - 75;
        return { id: row.id, username: row.username,
          status: row.accepted ? (online ? row.activity : "offline") : "pending",
          incoming: !row.accepted && row.recipient === me.id,
          lastSeen: row.accepted && row.last_seen ? new Date(row.last_seen * 1000).toISOString() : null,
          activity: online ? row.instance_name : null,
          mcVersion: online ? row.mc_version : null, loader: online ? row.loader : null,
          serverIp: online ? row.server_host : null, serverPort: online ? row.server_port : null };
      });
      return respond({ me, friends });
    }
    if (typeof body.targetId !== "string" || !/^[a-f0-9-]{36}$/.test(body.targetId) || body.targetId === me.id) return respond({ error: "Invalid friend" }, 400);
    const target = body.targetId;
    if (action === "invite") {
      if (await limited(db, `invite:${me.id}`, now, 3600, 20)) return respond({ error: "Limite de convites atingido." }, 429);
      const exists = await db.prepare("SELECT id FROM social_users WHERE id = ?").bind(target).first();
      if (!exists) return respond({ error: "Jogador não encontrado." }, 404);
      const pair = [me.id, target].sort().join(":");
      const result = await db.prepare("INSERT OR IGNORE INTO social_relationships(sender, recipient, pair_key, created_at) SELECT ?, ?, ?, ? WHERE (SELECT count(*) FROM social_relationships WHERE sender = ? OR recipient = ?) < 200 AND (SELECT count(*) FROM social_relationships WHERE sender = ? OR recipient = ?) < 200")
        .bind(me.id, target, pair, now, me.id, me.id, target, target).run();
      if (!result.meta.changes) return respond({ error: "Convite já existente ou lista de amigos cheia." }, 409);
    } else if (action === "accept") {
      const result = await db.prepare("UPDATE social_relationships SET accepted = 1 WHERE sender = ? AND recipient = ? AND accepted = 0").bind(target, me.id).run();
      if (!result.meta.changes) return respond({ error: "Convite recebido não encontrado." }, 404);
    } else {
      await db.prepare("DELETE FROM social_relationships WHERE (sender = ? AND recipient = ?) OR (sender = ? AND recipient = ?)").bind(me.id, target, target, me.id).run();
    }
    return respond({ ok: true });
  } catch {
    return respond({ error: "Serviço social temporariamente indisponível." }, 503);
  }
}
