import type { SiteContext } from '../../lib/types.js';
import { authenticate, digest, json, sameOrigin, sessionToken } from '../../lib/accounts.js';
import { commerceIdentity, membership } from '../../lib/membership.js';
import { commerceCookie, csrfFor, verifyCsrf } from '../../lib/owner-security.js';
import { activeProfileImage } from '../../lib/profile-style.js';
import { validAvatar } from '../../lib/avatar.js';

export async function onRequest({request, env}: SiteContext) {
    if (!['GET','POST'].includes(request.method)) return json({error:'Método não permitido.'},405);
    if (!sameOrigin(request)) return json({error:'Origem não permitida.'},403);
    if (!env.SOCIAL_DB) return json({error:'Serviço indisponível.'},503);
    try {
        const db = env.SOCIAL_DB;
        const me = await commerceIdentity(request, db);
        if (request.method === 'POST') {
            await verifyCsrf(request);
            if (!me) return json({error:'Sua sessão expirou.'},401);
            const accountToken = sessionToken(request), scoped = commerceCookie(request);
            const statements = [];
            if (accountToken) statements.push(db.prepare('DELETE FROM lux_sessions WHERE token_hash = ?').bind(await digest(accountToken)));
            if (scoped) statements.push(db.prepare('DELETE FROM lux_commerce_sessions WHERE token_hash = ?').bind(await digest(scoped)));
            if (statements.length) await db.batch(statements);
            const response = json({ok:true});
            for (const name of ['__Host-luxmc','__Host-luxmc-commerce']) response.headers.append('Set-Cookie',`${name}=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0`);
            return response;
        }
        if (!me) return json({user:null});
        const profile = await db.prepare('SELECT p.display_name, p.portrait, p.updated_at, u.avatar_url, a.username AS account_username, s.updated_at AS skin_updated FROM social_users u LEFT JOIN social_public_profiles p ON p.user_id = u.id LEFT JOIN lux_accounts a ON a.social_id = u.id LEFT JOIN lux_appearance s ON s.account_id = a.id WHERE u.id = ? AND u.removed_at = 0').bind(me.id).first();
        const entitlement = await membership(db,me.id);
        const portrait = activeProfileImage(profile?.portrait || '', entitlement.plan);
        const photo = typeof portrait === 'string' && portrait.length <= 960000 ? portrait.match(/^data:image\/(png|jpeg|webp|gif);base64,([A-Za-z0-9+/=]+)$/) : null;
        const url = new URL(request.url);
        if (url.searchParams.get('asset') === 'avatar') {
            if (url.searchParams.get('profile') !== me.id) return json({error:'Perfil incorreto.'},403);
            if (!photo) return json({error:'Foto indisponível.'},404);
            const bytes = Uint8Array.from(atob(photo[2]),character=>character.charCodeAt(0));
            return new Response(bytes,{headers:{'Content-Type':`image/${photo[1]}`,'Content-Length':String(bytes.length),'Cache-Control':'private, max-age=60','Vary':'Cookie','X-Content-Type-Options':'nosniff','Cross-Origin-Resource-Policy':'same-origin'}});
        }
        const token = commerceCookie(request) || sessionToken(request);
        const registered = await authenticate(request,db);
        const avatarUrl = photo ? `/api/session?asset=avatar&profile=${me.id}&v=${profile?.updated_at || 0}` : validAvatar(profile?.avatar_url) ? profile?.avatar_url : null;
        return json({user:{id:me.id,username:me.username,displayName:profile?.display_name || me.username,avatarUrl,skinUrl:profile?.skin_updated ? `/api/appearance/${encodeURIComponent(profile.account_username)}?asset=skin&v=${profile.skin_updated}` : null,canManageAccount:registered?.social_id === me.id,membership:entitlement},csrfToken:token ? await csrfFor(token) : null});
    } catch (error) {
        const status = Number((error as {status?:number}).status) || 503;
        return json({error:status < 500 ? (error as Error).message : 'Não foi possível conferir sua conta agora.'},status);
    }
}
