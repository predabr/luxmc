import { createRemoteJWKSet, jwtVerify } from 'jose';
import { digest, randomToken } from './accounts.js';
import { microsoftJson, type microsoftConfiguration } from './microsoft.js';
import type { Database } from './types.js';

const consumerTenant = '9188040d-6c67-4c5b-b112-36a304b66dad';
const issuer = `https://login.microsoftonline.com/${consumerTenant}/v2.0`;
const keys = createRemoteJWKSet(new URL('https://login.microsoftonline.com/consumers/discovery/v2.0/keys'), { timeoutDuration: 8000 });
export const MICROSOFT_LINK_COOKIE = '__Host-luxmc-microsoft-link';
export const microsoftNonce = (verifier: string) => digest(`luxmc-microsoft-identity:${verifier}`);
export const microsoftLinkCookie = (value: string, age = 600) => `${MICROSOFT_LINK_COOKIE}=${value}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=${age}`;
export const microsoftLinkToken = (request: Request) => request.headers.get('Cookie')?.split(';').map(value => value.trim()).find(value => value.startsWith(`${MICROSOFT_LINK_COOKIE}=`))?.slice(MICROSOFT_LINK_COOKIE.length + 1) || '';

export async function verifiedMicrosoftIdentity(code: string, verifier: string, configuration: NonNullable<ReturnType<typeof microsoftConfiguration>>) {
    const result = await microsoftJson('https://login.microsoftonline.com/consumers/oauth2/v2.0/token', {
        method: 'POST',
        body: new URLSearchParams({ client_id: configuration.clientId, client_secret: configuration.secret, code, code_verifier: verifier, grant_type: 'authorization_code', redirect_uri: configuration.redirect })
    });
    if (typeof result.id_token !== 'string' || result.id_token.length > 32000) throw new Error('Missing Microsoft identity token');
    const { payload } = await jwtVerify(result.id_token, keys, {
        issuer, audience: configuration.clientId, algorithms: ['RS256'],
        requiredClaims: ['sub', 'exp', 'iat', 'nonce', 'tid'], clockTolerance: 60, maxTokenAge: '15m'
    });
    if (payload.tid !== consumerTenant || payload.nonce !== await microsoftNonce(verifier) || typeof payload.sub !== 'string' || !payload.sub || payload.sub.length > 255) throw new Error('Invalid Microsoft identity');
    const label = (typeof payload.preferred_username === 'string' ? payload.preferred_username : typeof payload.name === 'string' ? payload.name : 'Microsoft').replace(/[\u0000-\u001f\u007f]/g, '').slice(0, 200);
    return { subject: await digest(`${issuer}\n${configuration.clientId}\n${payload.sub}`), label };
}

export async function microsoftSubjectAccount(db: Database, subject: string) {
    return db.prepare('SELECT a.* FROM lux_microsoft_subjects m JOIN lux_accounts a ON a.id = m.account_id JOIN social_users u ON u.id = a.social_id WHERE m.subject_hash = ? AND u.removed_at = 0').bind(subject).first();
}

export async function microsoftLinkPending(request: Request, db: Database, now: number) {
    const token = microsoftLinkToken(request);
    if (!/^[a-f0-9]{64}$/.test(token)) return null;
    return db.prepare('SELECT token_hash, subject_hash, display_name FROM lux_microsoft_link_pending WHERE token_hash = ? AND expires_at > ?').bind(await digest(token), now).first();
}

export async function linkMicrosoftIdentity(db: Database, pending: { token_hash: string; subject_hash: string }, socialId: string, now: number) {
    const user = await db.prepare('SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0').bind(socialId).first();
    if (!user) throw Object.assign(new Error('Profile unavailable'), { reason: 'conflict' });
    const mapped = await microsoftSubjectAccount(db, pending.subject_hash);
    if (mapped && mapped.social_id !== socialId) throw Object.assign(new Error('Identity already linked'), { reason: 'conflict' });
    const existing = await db.prepare('SELECT * FROM lux_accounts WHERE social_id = ?').bind(socialId).first();
    const accountId = existing?.id || crypto.randomUUID();
    const linked = await db.prepare('SELECT subject_hash FROM lux_microsoft_subjects WHERE account_id = ?').bind(accountId).first();
    if (linked && linked.subject_hash !== pending.subject_hash) throw Object.assign(new Error('Profile already linked'), { reason: 'conflict' });
    const statements = [];
    if (!existing) statements.push(db.prepare("INSERT INTO lux_accounts(id, username, password_hash, password_salt, recovery_hash, social_id, created_at, auth_provider) VALUES (?, ?, ?, ?, ?, ?, ?, 'microsoft')").bind(accountId, user.username, await digest(randomToken()), randomToken(), await digest(randomToken()), socialId, now));
    if (!mapped) statements.push(db.prepare('INSERT INTO lux_microsoft_subjects(subject_hash, account_id, created_at) VALUES (?, ?, ?)').bind(pending.subject_hash, accountId, now));
    statements.push(db.prepare('DELETE FROM lux_microsoft_link_pending WHERE token_hash = ?').bind(pending.token_hash));
    try { await db.batch(statements); }
    catch { throw Object.assign(new Error('Account link conflict'), { reason: 'conflict' }); }
    return db.prepare('SELECT a.* FROM lux_accounts a JOIN social_users u ON u.id = a.social_id WHERE a.id = ? AND u.removed_at = 0').bind(accountId).first();
}
