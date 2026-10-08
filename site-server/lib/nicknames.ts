import type { Database } from "./types.js";
import { validNickname } from "./accounts.js";
export async function officialNicknameExists(name: any, lookup: any = fetch) {
    if (!validNickname(name))
        throw Object.assign(new Error("Nickname inválido."), { status: 400 });
    for (const base of ["https://api.minecraftservices.com/minecraft/profile/lookup/name/", "https://api.mojang.com/minecraft/profile/lookup/name/", "https://api.mojang.com/users/profiles/minecraft/"]) {
        try {
            const response = await lookup(`${base}${encodeURIComponent(name)}`, { signal: AbortSignal.timeout(3500), headers: { Accept: "application/json", "User-Agent": "Luxmc-Account-Nickname-Check" }, cf: { cacheTtl: 60, cacheEverything: true } });
            if (response.status === 404 || response.status === 204)
                return false;
            if (!response.ok)
                continue;
            const profile = await response.json();
            if (/^[a-f0-9]{32}$/i.test(profile.id || "") && String(profile.name).toLowerCase() === name.toLowerCase())
                return true;
        }
        catch { }
    }
    try {
        const response = await lookup("https://api.minecraftservices.com/minecraft/profile/lookup/bulk/byname", {
            method: "POST", headers: { Accept: "application/json", "Content-Type": "application/json" }, body: JSON.stringify([name]), signal: AbortSignal.timeout(3500)
        });
        if (response.ok) {
            const profiles = await response.json();
            if (Array.isArray(profiles)) {
                if (profiles.length === 0)
                    return false;
                if (profiles.some((profile: any) => /^[a-f0-9]{32}$/i.test(profile.id || "") && String(profile.name).toLowerCase() === name.toLowerCase()))
                    return true;
            }
        }
    }
    catch { }
    try {
        const response = await lookup(`https://playerdb.co/api/player/minecraft/${encodeURIComponent(name)}`, { signal: AbortSignal.timeout(3500), headers: { Accept: "application/json", "User-Agent": "Luxmc-Account-Nickname-Check" } });
        const result = await response.json();
        if (response.ok && result.code === "player.found" && result.success === true && /^[a-f0-9]{32}$/i.test(result.data?.player?.raw_id || "") && String(result.data.player.username).toLowerCase() === name.toLowerCase())
            return true;
        if (response.status === 400 && result.code === "minecraft.invalid_username" && result.success === false)
            return false;
    }
    catch { }
    throw Object.assign(new Error("A consulta de nomes oficiais está indisponível. Tente novamente em instantes."), { status: 503 });
}
export async function nicknameAvailability(db: Database, name: any, lookup: any = fetch) {
    const local = await db.prepare("SELECT id FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(name).first();
    const official = await officialNicknameExists(name, lookup);
    if (!local && !official)
        return { available: true, suggestions: [] };
    const suffixes = ["_LX", "_MC", String(crypto.getRandomValues(new Uint16Array(1))[0] % 9000 + 1000)];
    const candidates = suffixes.map((suffix: any) => `${name.slice(0, 16 - suffix.length)}${suffix}`);
    const suggestions = [];
    for (const candidate of candidates) {
        const registered = await db.prepare("SELECT id FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(candidate).first();
        if (!registered && !await officialNicknameExists(candidate, lookup))
            suggestions.push(candidate);
    }
    return { available: false, reason: official ? "official" : "registered", suggestions };
}
