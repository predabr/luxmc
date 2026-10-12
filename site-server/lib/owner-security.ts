import { digest, equalHash, json, limited, passwordHash, randomToken, sessionToken } from './accounts.js';
import { COMMERCE_COOKIE, fail, nowSeconds } from './membership.js';
import type { Database, SiteEnvironment } from './types.js';

const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
export function base32(bytes: Uint8Array) {
    let output = '', bits = 0, value = 0;
    for (const byte of bytes) {
        value = (value << 8) | byte; bits += 8;
        while (bits >= 5) { output += alphabet[(value >>> (bits - 5)) & 31]; bits -= 5; }
    }
    if (bits) output += alphabet[(value << (5 - bits)) & 31];
    return output;
}
export async function otpAt(secret: string, step: number, digits = 6) {
    const bytes: number[] = []; let value = 0, bits = 0;
    for (const character of secret) { const index = alphabet.indexOf(character); if (index < 0) return fail('Chave inválida.'); value = (value << 5) | index; bits += 5; if (bits >= 8) { bytes.push((value >>> (bits - 8)) & 255); bits -= 8; } }
    const counter = new Uint8Array(8); new DataView(counter.buffer).setBigUint64(0, BigInt(step));
    const key = await crypto.subtle.importKey('raw', new Uint8Array(bytes), { name: 'HMAC', hash: 'SHA-1' }, false, ['sign']);
    const result = new Uint8Array(await crypto.subtle.sign('HMAC', key, counter));
    const offset = result[result.length - 1] & 15;
    const number = new DataView(result.buffer).getUint32(offset) & 0x7fffffff;
    return (number % 10 ** digits).toString().padStart(digits, '0');
}
export async function encryptSecret(secret: string, pepper: string) {
    const key = await crypto.subtle.importKey('raw', await crypto.subtle.digest('SHA-256', new TextEncoder().encode(`luxmc-owner-totp:${pepper}`)), 'AES-GCM', false, ['encrypt']);
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const cipher = new Uint8Array(await crypto.subtle.encrypt({ name: 'AES-GCM', iv, additionalData: new TextEncoder().encode('luxmc-owner-totp-v1') }, key, new TextEncoder().encode(secret)));
    return `${Array.from(iv, byte => byte.toString(16).padStart(2, '0')).join('')}.${Array.from(cipher, byte => byte.toString(16).padStart(2, '0')).join('')}`;
}
export async function decryptSecret(encrypted: string, pepper: string) {
    const [ivHex, cipherHex] = encrypted.split('.');
    if (!/^[a-f0-9]{24}$/.test(ivHex) || !/^[a-f0-9]+$/.test(cipherHex) || cipherHex.length % 2) return fail('Autenticador indisponível.', 503);
    const bytes = (hex: string) => new Uint8Array(hex.match(/../g)!.map(pair => parseInt(pair, 16)));
    const key = await crypto.subtle.importKey('raw', await crypto.subtle.digest('SHA-256', new TextEncoder().encode(`luxmc-owner-totp:${pepper}`)), 'AES-GCM', false, ['decrypt']);
    return new TextDecoder().decode(await crypto.subtle.decrypt({ name: 'AES-GCM', iv: bytes(ivHex), additionalData: new TextEncoder().encode('luxmc-owner-totp-v1') }, key, bytes(cipherHex)));
}
export function commerceCookie(request: Request) {
    const token = request.headers.get('Cookie')?.split(';').map(value => value.trim()).find(value => value.startsWith(`${COMMERCE_COOKIE}=`))?.slice(COMMERCE_COOKIE.length + 1);
    return token && /^[a-f0-9]{64}$/.test(token) ? token : null;
}
export const csrfFor = (token: string) => digest(`luxmc-commerce-csrf:${token}`);
export async function verifyCsrf(request: Request) {
    if (/^Bearer [a-f0-9]{64}$/.test(request.headers.get('Authorization') || '')) return;
    const token = commerceCookie(request) || sessionToken(request);
    if (token && !equalHash(request.headers.get('X-Luxmc-CSRF') || '', await csrfFor(token))) return fail('Atualize a página antes de continuar.', 403);
}
export async function ownerSession(request: Request, db: Database, ownerId: string) {
    const token = commerceCookie(request);
    if (!token) return null;
    return db.prepare('SELECT token_hash, expires_at, verified_at, created_at FROM lux_commerce_sessions WHERE token_hash = ? AND social_id = ? AND expires_at > ? AND last_activity_at > ?').bind(await digest(token), ownerId, nowSeconds(), nowSeconds() - 300).first();
}
export async function requireOwnerSession(request: Request, db: Database, ownerId: string, fresh = false) {
    const session = await ownerSession(request, db, ownerId);
    if (!session?.verified_at || (fresh && session.verified_at <= nowSeconds() - 300)) return fail('Confirme sua senha para continuar.', 428);
    return session;
}
export async function ownerCredentials(request: Request, env: SiteEnvironment, password: unknown, otp: unknown) {
    if (!env.AUTH_PEPPER || env.AUTH_PEPPER.length < 32 || !env.OWNER_SOCIAL_ID) return fail('Acesso temporariamente indisponível.', 503);
    const db = env.SOCIAL_DB;
    if (await limited(db, `owner-auth:${await digest(request.headers.get('CF-Connecting-IP') || 'local')}`, 60, 5) || await limited(db, 'owner-auth-global', 600, 30)) return fail('Aguarde antes de tentar novamente.', 429);
    if (typeof password !== 'string' || password.length < 14 || password.length > 128) return fail('Senha ou código inválido.', 401);
    const config = await db.prepare('SELECT c.* FROM lux_commerce_config c JOIN social_users u ON u.id = ? WHERE c.id = 1 AND u.removed_at = 0').bind(env.OWNER_SOCIAL_ID).first();
    if (!config?.owner_password_hash || !equalHash(await passwordHash(password, config.owner_password_salt, env.AUTH_PEPPER), config.owner_password_hash)) return fail('Senha ou código inválido.', 401);
    if (config.owner_totp_secret) {
        const code = typeof otp === 'string' ? otp.replace(/[\s-]/g, '').toUpperCase() : '';
        let consumed = false;
        if (/^\d{6}$/.test(code)) {
            const secret = await decryptSecret(config.owner_totp_secret, env.AUTH_PEPPER);
            const current = Math.floor(nowSeconds() / 30);
            for (const step of [current, current - 1, current + 1]) {
                if (step > config.owner_totp_last_step && equalHash(code, await otpAt(secret, step))) {
                    consumed = Boolean((await db.prepare('UPDATE lux_commerce_config SET owner_totp_last_step = ? WHERE id = 1 AND owner_totp_secret = ? AND owner_totp_last_step < ?').bind(step, config.owner_totp_secret, step).run()).meta.changes); break;
                }
            }
        } else if (/^[A-F0-9]{32}$/.test(code)) {
            const hashes: string[] = JSON.parse(config.owner_recovery_hashes);
            const hash = await digest(`luxmc-owner-recovery:${code}`);
            if (hashes.includes(hash)) consumed = Boolean((await db.prepare('UPDATE lux_commerce_config SET owner_recovery_hashes = ? WHERE id = 1 AND owner_recovery_hashes = ? AND owner_totp_secret = ?').bind(JSON.stringify(hashes.filter(item => item !== hash)), config.owner_recovery_hashes, config.owner_totp_secret).run()).meta.changes);
            if (consumed) await db.batch([
                db.prepare('DELETE FROM lux_commerce_sessions WHERE social_id = ?').bind(env.OWNER_SOCIAL_ID),
                db.prepare('DELETE FROM lux_commerce_tickets WHERE social_id = ?').bind(env.OWNER_SOCIAL_ID),
                db.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)').bind(crypto.randomUUID(), env.OWNER_SOCIAL_ID, env.OWNER_SOCIAL_ID, 'security', 'Código de recuperação utilizado; outras sessões encerradas', nowSeconds())
            ]);
        }
        if (!consumed) return fail('Senha ou código inválido. Aguarde um código novo do autenticador.', 401);
    }
}
export async function newOwnerSession(request: Request, env: SiteEnvironment, action: string) {
    const now = nowSeconds(), token = randomToken();
    const old = commerceCookie(request);
    const statements = [];
    if (old) statements.push(env.SOCIAL_DB.prepare('DELETE FROM lux_commerce_sessions WHERE token_hash = ?').bind(await digest(old)));
    statements.push(env.SOCIAL_DB.prepare('INSERT INTO lux_commerce_sessions(token_hash, social_id, expires_at, verified_at, created_at, last_activity_at) VALUES (?, ?, ?, ?, ?, ?)').bind(await digest(token), env.OWNER_SOCIAL_ID, now + 900, now, now, now));
    statements.push(env.SOCIAL_DB.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)').bind(crypto.randomUUID(), env.OWNER_SOCIAL_ID, env.OWNER_SOCIAL_ID, 'security', action, now));
    await env.SOCIAL_DB.batch(statements);
    return json({ ok: true, csrfToken: await csrfFor(token) }, 200, { 'Set-Cookie': `${COMMERCE_COOKIE}=${token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=900` });
}
