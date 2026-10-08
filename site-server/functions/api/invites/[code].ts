import type { SiteContext } from "../../../lib/types.js";
import { digest, json, limited, readJson, sameOrigin } from "../../../lib/accounts.js";
const validCode = (value: any) => /^LUXMC1-[0-9]{16}$/.test(value);
function createCode() {
    let digits = "";
    while (digits.length < 16)
        for (const value of crypto.getRandomValues(new Uint8Array(24))) {
            if (value < 250 && digits.length < 16)
                digits += value % 10;
        }
    return `LUXMC1-${digits}`;
}
export async function onRequest({ request, env, params }: SiteContext) {
    if (!sameOrigin(request))
        return json({ error: "Origem não permitida" }, 403);
    if (!env.SOCIAL_DB)
        return json({ error: "Portal indisponível" }, 503);
    const db = env.SOCIAL_DB;
    const now = Math.floor(Date.now() / 1000);
    const ip = await digest(request.headers.get("CF-Connecting-IP") || "unknown");
    try {
        if (request.method === "POST" && params.code === "create") {
            if (await limited(db, `invite-create:${ip}`, 3600, 60))
                return json({ error: "Muitos convites. Tente mais tarde." }, 429);
            const body = await readJson(request, 131072);
            if (!["room", "instance"].includes(body.kind) || typeof body.payload !== "string")
                return json({ error: "Convite inválido" }, 400);
            if (body.kind === "room" && (body.payload.length > 4096 || !body.payload.includes("luxmc-world:")))
                return json({ error: "Sala inválida" }, 400);
            if (body.kind === "instance") {
                let manifest;
                try {
                    manifest = JSON.parse(body.payload);
                }
                catch {
                    return json({ error: "Instância inválida" }, 400);
                }
                if (!manifest || typeof manifest !== "object" || typeof manifest.name !== "string" || manifest.name.length > 128 || typeof manifest.mcVersion !== "string" || !["vanilla", "fabric", "forge", "neoforge", "quilt"].includes(manifest.loader) || !Array.isArray(manifest.mods) || manifest.mods.length > 2000)
                    return json({ error: "Instância inválida" }, 400);
            }
            const expires = now + (body.kind === "room" ? 3600 : 7 * 86400);
            for (let attempt = 0; attempt < 4; attempt++) {
                const code = createCode();
                const result = await db.prepare("INSERT OR IGNORE INTO shared_invites(code_hash,kind,payload,expires_at) VALUES(?,?,?,?)").bind(await digest(code), body.kind, body.payload, expires).run();
                if (result.meta.changes === 1) {
                    await db.prepare("DELETE FROM shared_invites WHERE expires_at < ?").bind(now).run();
                    return json({ code, expiresAt: expires });
                }
            }
            return json({ error: "Não foi possível gerar o código" }, 503);
        }
        if (request.method !== "GET")
            return json({ error: "Method not allowed" }, 405);
        const code = String(params.code || "").toUpperCase();
        if (!validCode(code))
            return json({ error: "Código inválido" }, 400);
        if (await limited(db, `invite-read:${ip}`, 60, 180))
            return json({ error: "Muitas tentativas" }, 429);
        const row = await db.prepare("SELECT kind,payload,expires_at FROM shared_invites WHERE code_hash = ? AND expires_at > ?").bind(await digest(code), now).first();
        return row ? json({ kind: row.kind, payload: row.payload, expiresAt: row.expires_at }) : json({ error: "Código não encontrado ou expirado" }, 404);
    }
    catch (error: any) {
        return json({ error: error.status ? error.message : "Serviço indisponível" }, error.status || 503);
    }
}
