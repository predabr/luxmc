import { membership } from "../../../lib/membership.js";
import { activeProfileStyle, parseProfileStyle, profileBenefits, profileStyleAllowed, retainedProfileStyle } from '../../../lib/profile-style.js';
import type { SiteContext } from "../../../lib/types.js";
import { validatePublicProfile } from "../../../lib/public-profile.js";
import { validAvatar } from "../../../lib/avatar.js";
import { friendSnapshot } from "../../../lib/social.js";
import { authenticate, sameOrigin, sessionToken } from "../../../lib/accounts.js";
const headers = { "Content-Type": "application/json", "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff" };
const respond = (data: any, status: any = 200) => new Response(JSON.stringify(data), { status, headers });
const digest = async (value: any) => Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value))), (b: any) => b.toString(16).padStart(2, "0")).join("");
const nickname = (value: any) => typeof value === "string" && /^[A-Za-z0-9_]{3,16}$/.test(value);
const shortText = (value: any, max: any) => typeof value === "string" ? value.slice(0, max) : "";
export function validServer(host: any, port: any) {
    if (typeof host !== "string" || host.length > 253 || !Number.isInteger(port) || port < 1 || port > 65535)
        return false;
    if (host.startsWith("[")) {
        try {
            return new URL(`http://${host}:${port}`).hostname === host.toLowerCase();
        }
        catch {
            return false;
        }
    }
    return host.split(".").every((label: any) => /^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$/.test(label));
}
async function limited(db: any, key: any, now: any, window: any, max: any) {
    const bucket = `${key}:${Math.floor(now / window)}`;
    const row = await db.prepare("INSERT INTO social_limits(bucket, count, expires_at) VALUES (?, 1, ?) ON CONFLICT(bucket) DO UPDATE SET count = count + 1 RETURNING count")
        .bind(bucket, now + window).first();
    return row.count > max;
}
export async function onRequest({ request, env, params, waitUntil }: SiteContext) {
    if (!sameOrigin(request))
        return respond({ error: "Origem não permitida" }, 403);
    if (request.method !== "POST")
        return respond({ error: "Method not allowed" }, 405);
    if (!env.SOCIAL_DB)
        return respond({ error: "O serviço social ainda não foi configurado no portal." }, 503);
    if (!request.headers.get("Content-Type")?.startsWith("application/json"))
        return respond({ error: "JSON required" }, 415);
    if (Number(request.headers.get("Content-Length")) > (params.action === "profile_save" ? 1950000 : 4096))
        return respond({ error: "Request too large" }, 413);
    const action = params.action;
    if (!["register", "sync", "search", "invite", "accept", "remove", "block", "unblock", "room_create", "room_join", "room_close", "stream_ticket", "profile_get", "profile_save", "owner_users", "owner_tags", "owner_delete"].includes(action))
        return respond({ error: "Unknown action" }, 404);
    const bearer = sessionToken(request);
    if (!bearer)
        return respond({ error: "Authentication required" }, 401);
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
                if (done)
                    break;
                byteCount += value.byteLength;
                if (byteCount > (params.action === "profile_save" ? 1950000 : 4096)) {
                    await reader.cancel();
                    return respond({ error: "Request too large" }, 413);
                }
                raw += decoder.decode(value, { stream: true });
            }
            raw += decoder.decode();
        }
        let body;
        try {
            body = JSON.parse(raw);
        }
        catch {
            return respond({ error: "Invalid JSON" }, 400);
        }
        if (!body || typeof body !== "object" || Array.isArray(body))
            return respond({ error: "Invalid body" }, 400);
        const tokenHash = await digest(bearer);
        const ipHash = await digest(request.headers.get("CF-Connecting-IP") || "local");
        if (await limited(db, `ip:${ipHash}`, now, 60, 120))
            return respond({ error: "Tente novamente em um minuto." }, 429);
        if (Math.random() < 0.01)
            waitUntil(db.prepare("DELETE FROM social_limits WHERE expires_at < ?").bind(now).run());
        const registered = await authenticate(request, db);
        if (!registered && (body.registeredAccount === true || request.headers.has("Cookie")))
            return respond({ error: "Sua sessão expirou. Entre novamente." }, 401);
        let me = registered ? { id: registered.social_id, username: registered.username } : await db.prepare("SELECT id, username, avatar_url AS avatarUrl FROM social_users WHERE token_hash = ?").bind(tokenHash).first();
        const removal = me ? await db.prepare("SELECT removed_at FROM social_users WHERE id = ?").bind(me.id).first() : null;
        if (removal?.removed_at) return respond({ error: "Este perfil foi removido da comunidade pelo dono do Luxmc." }, 410);
        if (action === "register") {
            if (registered)
                return respond({ me });
            if (!nickname(body.username))
                return respond({ error: "Nickname inválido (3–16 letras, números ou _)." }, 400);
            if (!me) {
                if (await limited(db, `register:${ipHash}`, now, 3600, 5))
                    return respond({ error: "Limite de novos perfis atingido." }, 429);
                const id = crypto.randomUUID();
                await db.prepare("INSERT INTO social_users(id, token_hash, username) VALUES (?, ?, ?) ON CONFLICT(token_hash) DO NOTHING").bind(id, tokenHash, body.username).run();
                me = await db.prepare("SELECT id, username, avatar_url AS avatarUrl FROM social_users WHERE token_hash = ?").bind(tokenHash).first();
            }
            return respond({ me });
        }
        if (!me)
            return respond({ error: "Perfil social não registrado." }, 401);
        if (action === "owner_users" || action === "owner_tags" || action === "owner_delete") {
            if (!env.OWNER_SOCIAL_ID || me.id !== env.OWNER_SOCIAL_ID)
                return respond({ error: "Acesso restrito ao dono do Luxmc." }, 403);
            if (action === "owner_delete") {
                if (typeof body.targetId !== "string" || !/^[a-f0-9-]{36}$/.test(body.targetId) || typeof body.confirmation !== "string") return respond({ error: "Perfil inválido." }, 400);
                if (body.targetId === me.id) return respond({ error: "O perfil do dono não pode ser excluído por este painel." }, 400);
                const target = await db.prepare("SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0").bind(body.targetId).first();
                if (!target) return respond({ error: "Perfil não encontrado." }, 404);
                if (body.confirmation !== target.username) return respond({ error: "Digite o nome de usuário para confirmar a exclusão." }, 400);
                await db.batch([
                    db.prepare("UPDATE social_users SET removed_at = ?, last_seen = 0, activity = 'offline', avatar_url = NULL, instance_name = '', mc_version = '', loader = '', server_host = NULL, server_port = NULL WHERE id = ?").bind(now, target.id),
                    db.prepare("DELETE FROM social_relationships WHERE sender = ? OR recipient = ?").bind(target.id, target.id),
                    db.prepare("DELETE FROM social_blocks WHERE owner = ? OR target = ?").bind(target.id, target.id),
                    db.prepare("DELETE FROM social_public_profiles WHERE user_id = ?").bind(target.id),
                    db.prepare("DELETE FROM social_user_tags WHERE user_id = ?").bind(target.id),
                    db.prepare("DELETE FROM social_rooms WHERE owner = ?").bind(target.id),
                    db.prepare("DELETE FROM social_stream_tickets WHERE user_id = ?").bind(target.id)
                ]);
                return respond({ ok: true });
            }
            if (action === "owner_tags") {
                if (typeof body.targetId !== "string" || !/^[a-f0-9-]{36}$/.test(body.targetId) || !Array.isArray(body.tags) || body.tags.length > 5 || body.tags.some((tag: unknown) => typeof tag !== "string" || !/^[\p{L}\p{N} _-]{1,24}$/u.test(tag)))
                    return respond({ error: "Use até 5 tags com no máximo 24 caracteres." }, 400);
                const target = await db.prepare("SELECT id FROM social_users WHERE id = ? AND removed_at = 0").bind(body.targetId).first();
                if (!target) return respond({ error: "Usuário não encontrado." }, 404);
                const tags = [...new Set<string>(body.tags.map((tag: string) => tag.trim()).filter(Boolean))];
                await db.prepare("INSERT INTO social_user_tags(user_id, tags, updated_by, updated_at) SELECT ?, ?, ?, ? WHERE EXISTS (SELECT 1 FROM social_users WHERE id = ? AND removed_at = 0) ON CONFLICT(user_id) DO UPDATE SET tags = excluded.tags, updated_by = excluded.updated_by, updated_at = excluded.updated_at").bind(target.id, JSON.stringify(tags), me.id, now, target.id).run();
                return respond({ ok: true, tags });
            }
            const offset = body.offset ?? 0;
            if (!Number.isInteger(offset) || offset < 0 || offset > 100000 || (body.query !== undefined && (typeof body.query !== "string" || body.query.length > 80)))
                return respond({ error: "Busca inválida." }, 400);
            const query = (body.query || "").trim().replace(/[!%_]/g, (value: string) => `!${value}`);
            const search = `%${query}%`;
            const stats = await db.prepare("SELECT COUNT(*) AS total, COALESCE(SUM(last_seen > ? AND activity != 'offline'), 0) AS online, COALESCE(SUM(last_seen > ? AND activity = 'in_game'), 0) AS playing FROM social_users WHERE removed_at = 0").bind(now - 75, now - 75).first();
            const filter = "WHERE u.removed_at = 0 AND (u.username LIKE ? ESCAPE '!' COLLATE NOCASE OR COALESCE(p.display_name, '') LIKE ? ESCAPE '!' COLLATE NOCASE)";
            const count = await db.prepare(`SELECT COUNT(*) AS total FROM social_users u LEFT JOIN social_public_profiles p ON p.user_id = u.id ${filter}`).bind(search, search).first();
            const { results } = await db.prepare(`SELECT u.id, u.username, u.avatar_url AS avatarUrl, u.last_seen, u.activity, COALESCE(p.display_name, '') AS displayName, COALESCE(t.tags, '[]') AS tags FROM social_users u LEFT JOIN social_public_profiles p ON p.user_id = u.id LEFT JOIN social_user_tags t ON t.user_id = u.id ${filter} ORDER BY u.last_seen DESC, u.id LIMIT 24 OFFSET ?`).bind(search, search, offset).all();
            return respond({ stats, users: results.map((user: any) => ({ id: user.id, username: user.username, displayName: user.displayName, avatarUrl: user.avatarUrl || null, status: user.last_seen > now - 75 ? user.activity : "offline", lastSeen: user.last_seen ? new Date(user.last_seen * 1000).toISOString() : null, tags: JSON.parse(user.tags) })), total: count?.total || 0, nextOffset: offset + results.length < (count?.total || 0) ? offset + results.length : null });
        }
        if (action === "profile_save") {
            if (await limited(db, `profile:${me.id}`, now, 60, 8))
                return respond({ error: "Aguarde antes de salvar novamente." }, 429);
            const entitlement = await membership(db, me.id);
            const savedCollections = await db.prepare("SELECT collections, packs, profile_style FROM social_public_profiles WHERE user_id = ?").bind(me.id).first();
            const existingCollections = JSON.parse(savedCollections?.collections || "[]");
            const existingPacks: string[] = JSON.parse(savedCollections?.packs || '[]');
            if (body.profileStyle === undefined && Array.isArray(body.packs) && body.packs.length <= 8 && existingPacks.length > 8) body.packs = [...body.packs, ...existingPacks.slice(8).filter(pack => !body.packs.includes(pack))];
            const favourites = profileBenefits[entitlement.plan].favourites;
            const style = parseProfileStyle(body.profileStyle);
            if (!style) return respond({ error: 'Estilo de perfil inválido.' }, 400);
            if (body.profileStyle !== undefined && !profileStyleAllowed(style, entitlement.plan)) return respond({ error: 'Essa personalização requer um plano ativo compatível.' }, 403);
            if (Array.isArray(body.packs) && body.packs.length > favourites && body.packs.some((pack: unknown) => typeof pack !== 'string' || !existingPacks.includes(pack.trim()))) return respond({ error: `Seu plano permite ${favourites} favoritos.` }, 403);
            if (body.collectionsLimit !== 24 && Array.isArray(body.collections) && body.collections.length <= 6 && existingCollections.length > 6) body.collections = [...body.collections, ...existingCollections.slice(6).filter((saved: any) => !body.collections.some((item: any) => item?.id === saved.id))];
            if (Array.isArray(body.collections) && body.collections.length > entitlement.collections && body.collections.some((item: any) => !existingCollections.some((saved: any) => saved.id === item?.id))) return respond({ error: `Seu plano permite ${entitlement.collections} coleções.` }, 403);
            const profile = validatePublicProfile(body, Math.max(entitlement.collections, existingCollections.length), Math.max(favourites, existingPacks.length));
            if (!profile)
                return respond({ error: "Perfil inválido. Use imagens PNG, JPEG, WebP ou GIF de até 700 KB e uma descrição de até 400 caracteres." }, 400);
            const persistedStyle = body.profileStyle === undefined ? savedCollections?.profile_style || JSON.stringify(style) : JSON.stringify(retainedProfileStyle(style, JSON.parse(savedCollections?.profile_style || '{}'), entitlement.plan));
            await db.prepare("INSERT INTO social_public_profiles(user_id, description, banner, portrait, packs, updated_at, display_name, status, collections, profile_style) SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ? WHERE EXISTS (SELECT 1 FROM social_users WHERE id = ? AND removed_at = 0) ON CONFLICT(user_id) DO UPDATE SET description = excluded.description, banner = excluded.banner, portrait = excluded.portrait, packs = excluded.packs, updated_at = excluded.updated_at, display_name = CASE WHEN ? THEN excluded.display_name ELSE social_public_profiles.display_name END, status = CASE WHEN ? THEN excluded.status ELSE social_public_profiles.status END, collections = CASE WHEN ? THEN excluded.collections ELSE social_public_profiles.collections END, profile_style = excluded.profile_style")
                .bind(me.id, profile.description, profile.banner, profile.portrait, JSON.stringify(profile.packs), now, profile.displayName, profile.status, JSON.stringify(profile.collections), persistedStyle, me.id, Number(typeof body.displayName === "string"), Number(typeof body.status === "string"), Number(Array.isArray(body.collections))).run();
            return respond({ ok: true });
        }
        if (action === "profile_get") {
            const target = body.targetId || me.id;
            if (typeof target !== 'string' || !/^[a-f0-9-]{36}$/.test(target))
                return respond({ error: "Invalid profile" }, 400);
            if (target !== me.id) {
                const relation = await db.prepare("SELECT 1 FROM social_relationships WHERE pair_key = ? AND accepted = 1 AND NOT EXISTS (SELECT 1 FROM social_blocks WHERE (owner = ? AND target = ?) OR (owner = ? AND target = ?))").bind([me.id, target].sort().join(':'), me.id, target, target, me.id).first();
                if (!relation)
                    return respond({ error: "Perfil indisponível." }, 403);
            }
            const user = await db.prepare("SELECT id, username, avatar_url AS avatarUrl FROM social_users WHERE id = ? AND removed_at = 0").bind(target).first();
            if (!user)
                return respond({ error: "Perfil não encontrado." }, 404);
            const saved = await db.prepare("SELECT description, banner, portrait, packs, display_name, status, collections, profile_style FROM social_public_profiles WHERE user_id = ?").bind(target).first();
            const assigned = await db.prepare("SELECT tags FROM social_user_tags WHERE user_id = ?").bind(target).first();
            const benefits = await membership(db, target);
            const storedStyle = parseProfileStyle(JSON.parse(saved?.profile_style || '{}'));
            return respond({ profile: { ...user, membership: benefits, profileStyle: activeProfileStyle(storedStyle, benefits.plan), ...(target === me.id ? { savedProfileStyle: storedStyle } : {}), role: env.OWNER_SOCIAL_ID && target === env.OWNER_SOCIAL_ID ? "owner" : "member", tags: JSON.parse(assigned?.tags || '[]'), displayName: saved?.display_name || '', status: saved?.status || '', description: saved?.description || '', banner: saved?.banner || '', portrait: saved?.portrait || '', packs: JSON.parse(saved?.packs || '[]').slice(0, body.favouritesLimit === 24 ? 24 : 8), collections: JSON.parse(saved?.collections || '[]').slice(0, body.collectionsLimit === 24 ? 24 : 6) } });
        }
        if (action === "stream_ticket") {
            if (!env.SOCIAL_STREAM_URL)
                return respond({ url: null });
            const url = new URL(env.SOCIAL_STREAM_URL);
            if (url.protocol !== "wss:")
                return respond({ error: "Invalid stream configuration" }, 503);
            if (await limited(db, `stream:${me.id}`, now, 60, 12))
                return respond({ error: "Aguarde antes de reconectar." }, 429);
            const ticket = `${crypto.randomUUID()}${crypto.randomUUID()}`;
            await db.prepare("DELETE FROM social_stream_tickets WHERE expires_at < ?").bind(now).run();
            await db.prepare("INSERT INTO social_stream_tickets(token_hash, user_id, expires_at) VALUES (?, ?, ?)").bind(await digest(ticket), me.id, now + 30).run();
            url.searchParams.set("ticket", ticket);
            return respond({ url: url.toString() });
        }
        if (action === "room_close") {
            await db.prepare("DELETE FROM social_rooms WHERE owner = ?").bind(me.id).run();
            return respond({ ok: true });
        }
        if (action === "room_create") {
            const octets = typeof body.host === "string" ? body.host.split(".").map(Number) : [];
            if (octets.length !== 4 || octets[0] !== 100 || octets[1] < 64 || octets[1] > 127 || !octets.every((n: any) => Number.isInteger(n) && n >= 0 && n <= 255) || !validServer(body.host, body.port))
                return respond({ error: "Use o IP virtual Tailscale e uma porta LAN válida." }, 400);
            if (await limited(db, `room:${me.id}`, now, 3600, 20))
                return respond({ error: "Limite de salas atingido." }, 429);
            await db.prepare("DELETE FROM social_rooms WHERE owner = ? OR expires_at < ?").bind(me.id, now).run();
            for (let attempt = 0; attempt < 8; attempt++) {
                const random = crypto.getRandomValues(new Uint32Array(1))[0];
                const code = String(100000 + random % 900000);
                const result = await db.prepare("INSERT OR IGNORE INTO social_rooms(code, owner, host, port, expires_at) VALUES (?, ?, ?, ?, ?)").bind(code, me.id, body.host, body.port, now + 3600).run();
                if (result.meta.changes)
                    return respond({ code, expiresAt: now + 3600 });
            }
            return respond({ error: "Tente criar a sala novamente." }, 503);
        }
        if (action === "room_join") {
            if (await limited(db, `join:${me.id}`, now, 60, 10))
                return respond({ error: "Muitas tentativas. Aguarde um minuto." }, 429);
            if (typeof body.code !== "string" || !/^\d{6}$/.test(body.code))
                return respond({ error: "Código inválido." }, 400);
            const room = await db.prepare("SELECT host, port, owner FROM social_rooms WHERE code = ? AND expires_at > ?").bind(body.code, now).first();
            if (!room)
                return respond({ error: "Sala expirada ou inexistente." }, 404);
            const relation = await db.prepare("SELECT accepted FROM social_relationships WHERE pair_key = ? AND accepted = 1").bind([me.id, room.owner].sort().join(":")).first();
            const blocked = await db.prepare("SELECT 1 FROM social_blocks WHERE (owner = ? AND target = ?) OR (owner = ? AND target = ?)").bind(me.id, room.owner, room.owner, me.id).first();
            if (blocked || (room.owner !== me.id && !relation))
                return respond({ error: "Esta sala é exclusiva para amigos aceitos pelo anfitrião." }, 403);
            return respond({ host: room.host, port: room.port });
        }
        if (action === "search") {
            const query = shortText(body.query, 64).trim();
            if (query.length < 3 || !/^[A-Za-z0-9_#-]+$/.test(query))
                return respond({ users: [] });
            const [name, code] = query.split("#");
            const escaped = name.replace(/_/g, "!_");
            const { results } = await db.prepare("SELECT u.id, u.username, u.avatar_url AS avatarUrl, COALESCE(t.tags, '[]') AS tags FROM social_users u LEFT JOIN social_user_tags t ON t.user_id = u.id WHERE u.removed_at = 0 AND u.username LIKE ? ESCAPE '!' COLLATE NOCASE AND u.id != ? AND NOT EXISTS (SELECT 1 FROM social_blocks b WHERE (b.owner = ? AND b.target = u.id) OR (b.target = ? AND b.owner = u.id)) AND (? = '' OR u.id LIKE ?) ORDER BY u.last_seen DESC, u.id LIMIT 8")
                .bind(`${escaped}%`, me.id, me.id, me.id, code || "", `${code || ""}%`).all();
            return respond({ users: results.map((user: any) => ({...user, role: env.OWNER_SOCIAL_ID && user.id === env.OWNER_SOCIAL_ID ? 'owner' : 'member', tags: JSON.parse(user.tags)})) });
        }
        if (action === "sync") {
            const activity = ["online", "in_game", "offline"].includes(body.status) ? body.status : "online";
            const server = activity === "in_game" && validServer(body.serverHost, body.serverPort);
            if (body.readOnly !== true)
                await db.prepare("UPDATE social_users SET last_seen = ?, activity = ?, instance_name = ?, mc_version = ?, loader = ?, server_host = ?, server_port = ?, avatar_url = CASE WHEN ? THEN ? ELSE avatar_url END WHERE id = ? AND removed_at = 0")
                    .bind(activity === "offline" ? 0 : now, activity, shortText(body.instanceName, 80), shortText(body.mcVersion, 40), shortText(body.loader, 20), server ? body.serverHost : null, server ? body.serverPort : null, Object.hasOwn(body, "avatarUrl") ? 1 : 0, validAvatar(body.avatarUrl) ? body.avatarUrl : null, me.id).run();
            me = await db.prepare("SELECT id, username, avatar_url AS avatarUrl FROM social_users WHERE id = ? AND removed_at = 0").bind(me.id).first();
            if (!me) return respond({error:"Perfil indisponível."},404);
            const assigned = await db.prepare("SELECT tags FROM social_user_tags WHERE user_id = ?").bind(me.id).first();
            const entitlement = await membership(db, me.id);
            const ownStyle = await db.prepare('SELECT profile_style FROM social_public_profiles WHERE user_id = ?').bind(me.id).first();
            me = {...me, profileStyle:activeProfileStyle(JSON.parse(ownStyle?.profile_style || '{}'), entitlement.plan), membership: entitlement, role: env.OWNER_SOCIAL_ID && me.id === env.OWNER_SOCIAL_ID ? 'owner' : 'member', tags: JSON.parse(assigned?.tags || '[]')};
            const friends = await friendSnapshot(db, me.id, now, env.OWNER_SOCIAL_ID);
            return respond({ me, friends });
        }
        if (typeof body.targetId !== "string" || !/^[a-f0-9-]{36}$/.test(body.targetId) || body.targetId === me.id)
            return respond({ error: "Invalid friend" }, 400);
        const target = body.targetId;
        if (!await db.prepare("SELECT id FROM social_users WHERE id = ? AND removed_at = 0").bind(target).first()) return respond({error:"Perfil não encontrado."},404);
        if (action === "block") {
            await db.prepare("INSERT OR IGNORE INTO social_blocks(owner, target) VALUES (?, ?)").bind(me.id, target).run();
            await db.prepare("DELETE FROM social_relationships WHERE pair_key = ?").bind([me.id, target].sort().join(":")).run();
            return respond({ ok: true });
        }
        if (action === "unblock") {
            await db.prepare("DELETE FROM social_blocks WHERE owner = ? AND target = ?").bind(me.id, target).run();
            return respond({ ok: true });
        }
        if (["invite", "accept"].includes(action)) {
            const blocked = await db.prepare("SELECT 1 FROM social_blocks WHERE (owner = ? AND target = ?) OR (owner = ? AND target = ?)").bind(me.id, target, target, me.id).first();
            if (blocked)
                return respond({ error: "Este jogador não está disponível para convites." }, 403);
        }
        if (action === "invite") {
            if (await limited(db, `invite:${me.id}`, now, 3600, 20))
                return respond({ error: "Limite de convites atingido." }, 429);
            const exists = await db.prepare("SELECT id FROM social_users WHERE id = ? AND removed_at = 0").bind(target).first();
            if (!exists)
                return respond({ error: "Jogador não encontrado." }, 404);
            const pair = [me.id, target].sort().join(":");
            const result = await db.prepare("INSERT OR IGNORE INTO social_relationships(sender, recipient, pair_key, created_at) SELECT ?, ?, ?, ? WHERE (SELECT count(*) FROM social_relationships WHERE sender = ? OR recipient = ?) < 200 AND (SELECT count(*) FROM social_relationships WHERE sender = ? OR recipient = ?) < 200")
                .bind(me.id, target, pair, now, me.id, me.id, target, target).run();
            if (!result.meta.changes)
                return respond({ error: "Convite já existente ou lista de amigos cheia." }, 409);
        }
        else if (action === "accept") {
            const result = await db.prepare("UPDATE social_relationships SET accepted = 1 WHERE sender = ? AND recipient = ? AND accepted = 0").bind(target, me.id).run();
            if (!result.meta.changes)
                return respond({ error: "Convite recebido não encontrado." }, 404);
        }
        else {
            await db.prepare("DELETE FROM social_relationships WHERE (sender = ? AND recipient = ?) OR (sender = ? AND recipient = ?)").bind(me.id, target, target, me.id).run();
        }
        return respond({ ok: true });
    }
    catch {
        return respond({ error: "Serviço social temporariamente indisponível." }, 503);
    }
}
