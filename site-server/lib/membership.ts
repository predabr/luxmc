import { authenticate, digest, sessionToken } from './accounts.js';
import type { Database } from './types.js';
import { profileBenefits } from './profile-style.js';

export const COMMERCE_COOKIE = '__Host-luxmc-commerce';
export const catalog = {
    plus: { name: 'Luxmc Plus', amount: 990, days: 30, collections: 12, presets: 6, profile: profileBenefits.plus },
    pro: { name: 'Luxmc Pro', amount: 1990, days: 30, collections: 24, presets: 20, profile: profileBenefits.pro }
};
export type PaidPlan = keyof typeof catalog;
export const nowSeconds = () => Math.floor(Date.now() / 1000);
export const fail = (message: string, status = 400): never => { throw Object.assign(new Error(message), { status }); };
export async function commerceIdentity(request: Request, db: Database) {
    if (!request.headers.get('Authorization')) {
        const scoped = request.headers.get('Cookie')?.split(';').map(value => value.trim()).find(value => value.startsWith(`${COMMERCE_COOKIE}=`))?.slice(COMMERCE_COOKIE.length + 1);
        if (scoped && /^[a-f0-9]{64}$/.test(scoped)) {
            const session = await db.prepare('SELECT u.id, u.username FROM lux_commerce_sessions s JOIN social_users u ON u.id = s.social_id WHERE s.token_hash = ? AND s.expires_at > ? AND u.removed_at = 0').bind(await digest(scoped), nowSeconds()).first();
            if (session) return session;
        }
    }
    const registered = await authenticate(request, db);
    if (registered) return db.prepare('SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0').bind(registered.social_id).first();
    const token = sessionToken(request);
    if (token && request.headers.has('Authorization')) return db.prepare('SELECT id, username FROM social_users WHERE token_hash = ? AND removed_at = 0').bind(await digest(token)).first();
    return null;
}
export async function membership(db: Database, id: string) {
    const row = await db.prepare('SELECT plan, expires_at, revision FROM lux_memberships WHERE social_id = ?').bind(id).first();
    const plan: 'free' | PaidPlan = row && row.expires_at > nowSeconds() ? row.plan : 'free';
    return { plan, expiresAt: row ? row.expires_at : null, revision: row?.revision || 0, collections: plan === 'free' ? 6 : catalog[plan].collections, presets: plan === 'free' ? 0 : catalog[plan].presets };
}
export function appearancePreferences(value: unknown): Record<string, string | number | boolean> {
    if (!value || typeof value !== 'object' || Array.isArray(value)) return fail('Preferências inválidas.');
    const record = value as Record<string, unknown>;
    const allowed: Record<string, unknown[]> = { density: ['compact', 'comfortable', 'spacious'], motionStyle: ['subtle', 'expressive'], motionSpeed: ['fast', 'balanced'], wallpaperFps: [15, 24, 30, 60], pauseSkinWhileGaming: [true, false], preloadContentIcons: [true, false], respectReducedMotion: [true, false], gifQuality: ['balanced', 'high'] };
    const clean: Record<string, string | number | boolean> = {};
    for (const [key, values] of Object.entries(allowed)) {
        if (Object.hasOwn(record, key)) {
            if (!values.includes(record[key])) return fail('Preferência inválida.');
            clean[key] = record[key] as string | number | boolean;
        }
    }
    if (Object.hasOwn(record, 'interfaceOpacity')) {
        if (typeof record.interfaceOpacity !== 'number' || !Number.isFinite(record.interfaceOpacity) || record.interfaceOpacity < 0.05 || record.interfaceOpacity > 1) return fail('Opacidade inválida.');
        clean.interfaceOpacity = record.interfaceOpacity;
    }
    return clean;
}
