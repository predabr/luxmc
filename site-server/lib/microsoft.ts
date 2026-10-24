import { digest, randomToken, validNickname } from './accounts.js';
import type { Database, SiteEnvironment } from './types.js';

export const MICROSOFT_COOKIE = '__Host-luxmc-microsoft';
const microsoftCookieName = (state = '') => /^[a-f0-9]{64}$/.test(state) ? `${MICROSOFT_COOKIE}-${state}` : MICROSOFT_COOKIE;
export const microsoftCookie = (token: string, age = 600, state = '') => `${microsoftCookieName(state)}=${token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=${age}`;
export const microsoftBrowser = (request: Request, state = '') => request.headers.get('Cookie')?.split(';').map(value => value.trim()).find(value => value.startsWith(`${microsoftCookieName(state)}=`))?.slice(microsoftCookieName(state).length + 1) || '';
export function microsoftConfiguration(env: SiteEnvironment) {
    if (!env.MICROSOFT_CLIENT_ID || !/^[a-f0-9-]{36}$/i.test(env.MICROSOFT_CLIENT_ID) || !env.MICROSOFT_CLIENT_SECRET) return null;
    const redirect = env.MICROSOFT_REDIRECT_URI || 'https://luxmc-r92.pages.dev/api/microsoft/callback';
    let url: URL;
    try { url = new URL(redirect); } catch { return null; }
    if (url.protocol !== 'https:' || !['luxmc-r92.pages.dev', 'luxmc.top', 'www.luxmc.top'].includes(url.hostname) || url.pathname !== '/api/microsoft/callback' || url.search || url.hash || url.port || url.username || url.password) return null;
    return { clientId: env.MICROSOFT_CLIENT_ID, secret: env.MICROSOFT_CLIENT_SECRET, redirect };
}
export async function microsoftJson(url: string, options: RequestInit): Promise<Record<string, any>> {
    const service = new URL(url).hostname + new URL(url).pathname;
    let response: Response;
    const headers = new Headers(options.headers);
    headers.set('Accept', 'application/json');
    headers.set('User-Agent', 'Luxmc-Website/4.0.0 (+https://luxmc-r92.pages.dev)');
    try { response = await fetch(url, { ...options, headers, redirect: 'manual', signal: AbortSignal.timeout(10000) }); }
    catch { throw Object.assign(new Error('Microsoft authentication unavailable'), { reason: 'unavailable', service, status: 0 }); }
    if (!response.ok) {
        let providerCode: string | undefined;
        let providerReason: string | undefined;
        try {
            const bytes = await response.arrayBuffer();
            if (bytes.byteLength <= 131072) {
                const text = new TextDecoder().decode(bytes);
                if (/invalid app registration|application.*(?:not approved|not allowed)|app.*registration/i.test(text)) providerReason = 'application_registration';
                else if (/request is blocked|access denied|cloudflare|forbidden/i.test(text)) providerReason = 'provider_policy';
                const value = JSON.parse(text);
                if (typeof value?.error === 'string' && /^[a-zA-Z_]{1,64}$/.test(value.error)) providerCode = value.error;
                else if (Number.isSafeInteger(value?.XErr)) providerCode = String(value.XErr);
            }
        } catch {}
        throw Object.assign(new Error('Microsoft authentication unavailable'), { reason: url.endsWith('/minecraft/profile') && response.status === 404 ? 'minecraft' : 'unavailable', service, status: response.status, providerCode, providerReason });
    }
    const bytes = await response.arrayBuffer();
    if (bytes.byteLength > 131072) throw new Error('Authentication response too large');
    const value = JSON.parse(new TextDecoder().decode(bytes));
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid authentication response');
    return value;
}
export async function verifiedMinecraftProfile(accessToken: string) {
    const profile = await microsoftJson('https://api.minecraftservices.com/minecraft/profile', { headers: { Authorization: `Bearer ${accessToken}` } });
    if (!/^[a-f0-9]{32}$/i.test(profile.id) || !validNickname(profile.name)) throw new Error('Invalid Minecraft profile');
    return { uuid: String(profile.id).toLowerCase(), username: String(profile.name) };
}
export async function microsoftProfile(code: string, verifier: string, configuration: NonNullable<ReturnType<typeof microsoftConfiguration>>) {
    const token = await microsoftJson('https://login.microsoftonline.com/consumers/oauth2/v2.0/token', { method: 'POST', body: new URLSearchParams({ client_id: configuration.clientId, client_secret: configuration.secret, code, code_verifier: verifier, grant_type: 'authorization_code', redirect_uri: configuration.redirect }) });
    if (typeof token.access_token !== 'string' || token.access_token.length > 32000) throw new Error('Missing Microsoft token');
    const xbox = await microsoftJson('https://user.auth.xboxlive.com/user/authenticate', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-xbl-contract-version': '1' }, body: JSON.stringify({ RelyingParty: 'http://auth.xboxlive.com', TokenType: 'JWT', Properties: { AuthMethod: 'RPS', SiteName: 'user.auth.xboxlive.com', RpsTicket: `d=${token.access_token}` } }) });
    if (typeof xbox.Token !== 'string') throw new Error('Missing Xbox token');
    const xsts = await microsoftJson('https://xsts.auth.xboxlive.com/xsts/authorize', { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-xbl-contract-version': '1' }, body: JSON.stringify({ RelyingParty: 'rp://api.minecraftservices.com/', TokenType: 'JWT', Properties: { SandboxId: 'RETAIL', UserTokens: [xbox.Token] } }) });
    const hash = xsts.DisplayClaims?.xui?.[0]?.uhs;
    if (typeof xsts.Token !== 'string' || typeof hash !== 'string' || !/^[0-9]+$/.test(hash)) throw new Error('Missing Xbox identity');
    const minecraft = await microsoftJson('https://api.minecraftservices.com/authentication/login_with_xbox', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ identityToken: `XBL3.0 x=${hash};${xsts.Token}` }) });
    if (typeof minecraft.access_token !== 'string') throw new Error('Missing Minecraft token');
    return verifiedMinecraftProfile(minecraft.access_token);
}
export async function microsoftAccount(db: Database, profile: { uuid: string; username: string }, linkedSocialId: string | null, now: number) {
    const existing = await db.prepare('SELECT a.* FROM lux_microsoft_accounts m JOIN lux_accounts a ON a.id = m.account_id WHERE m.minecraft_uuid = ?').bind(profile.uuid).first();
    if (existing) {
        if (linkedSocialId && existing.social_id !== linkedSocialId) throw Object.assign(new Error('Identity already linked'), { reason: 'conflict' });
        const active = await db.prepare('SELECT id FROM social_users WHERE id = ? AND removed_at = 0').bind(existing.social_id).first();
        if (!active) throw Object.assign(new Error('Profile unavailable'), { reason: 'conflict' });
        return existing;
    }
    const linked = linkedSocialId ? await db.prepare('SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0').bind(linkedSocialId).first() : null;
    if (linkedSocialId && !linked) throw Object.assign(new Error('Profile unavailable'), { reason: 'conflict' });
    if (linked && linked.username.toLowerCase() !== profile.username.toLowerCase()) throw Object.assign(new Error('Different Minecraft identity'), { reason: 'conflict' });
    const account = linked ? await db.prepare('SELECT * FROM lux_accounts WHERE social_id = ?').bind(linked.id).first() : null;
    const accountId = account?.id || crypto.randomUUID();
    const socialId = linked?.id || crypto.randomUUID();
    const statements = [];
    if (!linked) statements.push(db.prepare('INSERT INTO social_users(id, token_hash, username) VALUES (?, ?, ?)').bind(socialId, await digest(randomToken()), profile.username));
    if (!account) statements.push(db.prepare("INSERT INTO lux_accounts(id, username, password_hash, password_salt, recovery_hash, social_id, created_at, auth_provider) VALUES (?, ?, ?, ?, ?, ?, ?, 'microsoft')").bind(accountId, profile.username, await digest(randomToken()), randomToken(), await digest(randomToken()), socialId, now));
    statements.push(db.prepare('INSERT INTO lux_microsoft_accounts(minecraft_uuid, account_id, created_at) VALUES (?, ?, ?)').bind(profile.uuid, accountId, now));
    try { await db.batch(statements); }
    catch {
        const raced = await db.prepare('SELECT a.* FROM lux_microsoft_accounts m JOIN lux_accounts a ON a.id = m.account_id WHERE m.minecraft_uuid = ?').bind(profile.uuid).first();
        if (raced && (!linkedSocialId || raced.social_id === linkedSocialId)) return raced;
        throw Object.assign(new Error('Account link conflict'), { reason: 'conflict' });
    }
    return db.prepare('SELECT * FROM lux_accounts WHERE id = ?').bind(accountId).first();
}
