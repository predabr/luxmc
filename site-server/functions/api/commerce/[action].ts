import type { SiteContext } from '../../../lib/types.js';
import { digest, equalHash, json, limited, passwordHash, randomToken, readJson, sameOrigin, sessionToken } from '../../../lib/accounts.js';
import { appearancePreferences, catalog, COMMERCE_COOKIE, commerceIdentity, fail, membership, nowSeconds, type PaidPlan } from '../../../lib/membership.js';
import { base32, commerceCookie, csrfFor, decryptSecret, encryptSecret, newOwnerSession, otpAt, ownerCredentials, ownerSession, requireOwnerSession, verifyCsrf } from '../../../lib/owner-security.js';
import { activeProfileStyle, parseProfileStyle, profileStyleAllowed, retainedProfileStyle } from '../../../lib/profile-style.js';
import { orderTxid, pixPayload, validPixKey } from '../../../lib/pix.js';

const uuid = (value: unknown): value is string => typeof value === 'string' && /^[a-f0-9-]{36}$/.test(value);
const selectOrders = `SELECT o.*, u.username, COALESCE(p.display_name, '') AS display_name, r.sha256 AS receipt_hash, r.mime AS receipt_mime, r.uploaded_at AS receipt_uploaded_at FROM lux_purchase_orders o JOIN social_users u ON u.id = o.social_id LEFT JOIN social_public_profiles p ON p.user_id = o.social_id LEFT JOIN lux_purchase_receipts r ON r.order_id = o.id`;
const publicOrder = (row: Record<string, any>) => ({ id: row.id, socialId: row.social_id, username: row.username, displayName: row.display_name, plan: row.plan, amount: row.amount, bank: row.bank, pixKey: row.pix_key, pixCode: validPixKey(row.pix_key) ? pixPayload({ key: row.pix_key, amount: row.amount, txid: orderTxid(row.id) }) : null, pixReference: orderTxid(row.id), days: row.days, status: row.status, version: row.version, createdAt: row.created_at, updatedAt: row.updated_at, note: row.note, receiptHash: row.receipt_hash || null, receiptMime: row.receipt_mime || null, receiptUploadedAt: row.receipt_uploaded_at || null });
export async function onRequest({ request, env, params, waitUntil }: SiteContext) {
    if (request.method !== 'POST') return json({ error: 'Use POST.' }, 405);
    if (!sameOrigin(request)) return json({ error: 'Origem não permitida.' }, 403);
    if (!/^Bearer [a-f0-9]{64}$/.test(request.headers.get('Authorization') || '') && request.headers.get('Origin') && request.headers.get('Origin') !== new URL(request.url).origin) return json({ error: 'Origem não permitida.' }, 403);
    const db = env.SOCIAL_DB;
    if (!db) return json({ error: 'Serviço indisponível.' }, 503);
    const now = nowSeconds();
    try {
        const body = await readJson(request, 24576);
        const action = params.action;
        if (!['status', 'exchange', 'owner_login'].includes(action)) await verifyCsrf(request);
        if (action === 'owner_login') {
            await ownerCredentials(request, env, body.password, body.otp);
            return newOwnerSession(request, env, 'Entrada na central do dono');
        }
        if (action === 'exchange') {
            if (typeof body.ticket !== 'string' || !/^[a-f0-9]{64}$/.test(body.ticket)) return fail('Link inválido.', 401);
            if (await limited(db, `commerce-exchange:${await digest(request.headers.get('CF-Connecting-IP') || 'local')}`, 60, 12)) return fail('Aguarde um minuto.', 429);
            const hash = await digest(body.ticket);
            const ticket = await db.prepare('SELECT t.social_id FROM lux_commerce_tickets t JOIN social_users u ON u.id = t.social_id WHERE t.token_hash = ? AND t.expires_at > ? AND u.removed_at = 0').bind(hash, now).first();
            if (!ticket) return fail('Link usado ou expirado. Abra novamente pelo launcher.', 401);
            const token = randomToken();
            const tokenHash = await digest(token);
            const age = ticket.social_id === env.OWNER_SOCIAL_ID ? 900 : 3600;
            const results = await db.batch([
                db.prepare('INSERT INTO lux_commerce_sessions(token_hash, social_id, expires_at, created_at, last_activity_at) SELECT ?, social_id, ?, ?, ? FROM lux_commerce_tickets WHERE token_hash = ? AND expires_at > ?').bind(tokenHash, now + age, now, now, hash, now),
                db.prepare('DELETE FROM lux_commerce_tickets WHERE token_hash = ?').bind(hash)
            ]);
            if (!results[0].meta.changes) return fail('Link já usado.', 401);
            const response = json({ ok: true, csrfToken: await csrfFor(token) }, 200, { 'Set-Cookie': `${COMMERCE_COOKIE}=${token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=${age}` });
            response.headers.append('Set-Cookie', '__Host-luxmc=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0');
            return response;
        }
        const me = await commerceIdentity(request, db);
        if (!me) return fail('Entre com sua conta ou abra esta página pelo launcher.', 401);
        if (await limited(db, `commerce:${me.id}`, 60, 50)) return fail('Aguarde um minuto.', 429);
        const owner = Boolean(env.OWNER_SOCIAL_ID && me.id === env.OWNER_SOCIAL_ID);
        if (action.startsWith('owner_') && !owner) return fail('Acesso restrito ao dono do Luxmc.', 403);
        if (action === 'owner_unlock') {
            await ownerCredentials(request, env, body.password, body.otp);
            return newOwnerSession(request, env, 'Senha confirmada para ações da central');
        }
        if (action.startsWith('owner_') && action !== 'owner_password') await requireOwnerSession(request, db, me.id, !['owner_queue', 'owner_updates', 'owner_find', 'owner_touch', 'owner_totp_begin', 'owner_totp_disable'].includes(action));
        if (action === 'owner_touch') {
            const session = await requireOwnerSession(request, db, me.id);
            await db.prepare('UPDATE lux_commerce_sessions SET last_activity_at = ? WHERE token_hash = ? AND expires_at > ? AND last_activity_at > ?').bind(now, session.token_hash, now, now - 300).run();
            return json({ ok: true });
        }
        if (Math.random() < 0.02) waitUntil(db.batch([
            db.prepare('DELETE FROM lux_commerce_tickets WHERE expires_at < ?').bind(now),
            db.prepare('DELETE FROM lux_commerce_sessions WHERE expires_at < ?').bind(now),
            db.prepare("DELETE FROM lux_purchase_receipts WHERE uploaded_at < ? AND order_id IN (SELECT id FROM lux_purchase_orders WHERE status IN ('approved','rejected','cancelled'))").bind(now - 90 * 86400)
        ]));
        if (action === 'updates' || action === 'owner_updates') {
            const clause = action === 'owner_updates' ? '' : ' WHERE social_id = ?';
            const changes = await db.prepare(`SELECT COUNT(*) AS total, COALESCE(SUM(version), 0) AS versions, COALESCE(MAX(updated_at), 0) AS modified, COALESCE(SUM(status = 'approved'), 0) AS approved FROM lux_purchase_orders${clause}`).bind(...(clause ? [me.id] : [])).first();
            const config = await db.prepare('SELECT enabled, updated_at FROM lux_commerce_config WHERE id = 1').first();
            return json({ revision: await digest(JSON.stringify([changes, config, await membership(db, me.id)])) });
        }
        if (action === 'portal_ticket') {
            const ticket = randomToken();
            await db.prepare('INSERT INTO lux_commerce_tickets(token_hash, social_id, expires_at) VALUES (?, ?, ?)').bind(await digest(ticket), me.id, now + 60).run();
            return json({ path: owner && body.owner === true ? '/painel-luxmc' : '/planos', ticket });
        }
        if (action === 'logout') {
            const cookie = request.headers.get('Cookie')?.match(/(?:^|;\s*)__Host-luxmc-commerce=([a-f0-9]{64})/)?.[1];
            if (cookie) await db.prepare('DELETE FROM lux_commerce_sessions WHERE token_hash = ?').bind(await digest(cookie)).run();
            return json({ ok: true }, 200, { 'Set-Cookie': `${COMMERCE_COOKIE}=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0` });
        }
        const current = await membership(db, me.id);
        if (action === 'status') {
            const profile = await db.prepare('SELECT profile_style, display_name, status FROM social_public_profiles WHERE user_id = ?').bind(me.id).first();
            const config = await db.prepare('SELECT enabled, bank, pix_key FROM lux_commerce_config WHERE id = 1').first();
            const preferences = await db.prepare('SELECT preferences, presets, revision FROM lux_membership_preferences WHERE social_id = ?').bind(me.id).first();
            const orders = await db.prepare(`${selectOrders} WHERE o.social_id = ? ORDER BY o.created_at DESC LIMIT 20`).bind(me.id).all();
            const token = commerceCookie(request) || sessionToken(request);
            const session = owner ? await ownerSession(request, db, me.id) : null;
            const security = owner ? await db.prepare('SELECT owner_totp_secret IS NOT NULL AS two_factor FROM lux_commerce_config WHERE id = 1').first() : null;
            return json({ identity: { id: me.id, username: me.username, owner }, csrfToken: token ? await csrfFor(token) : null, security: owner ? { twoFactor: Boolean(security?.two_factor), verifiedUntil: session?.verified_at ? session.verified_at + 300 : 0, expiresAt: session?.expires_at || 0 } : null, membership: current, catalog, profile: { profileStyle: activeProfileStyle(JSON.parse(profile?.profile_style || '{}'), current.plan), displayName: profile?.display_name || me.username, status: profile?.status || '' }, checkout: { enabled: Boolean(config?.enabled), bank: config?.bank || '', pixKey: config?.pix_key || '' }, settings: { preferences: JSON.parse(preferences?.preferences || '{}'), presets: JSON.parse(preferences?.presets || '[]'), revision: preferences?.revision || 0 }, orders: orders.results.map(publicOrder) });
        }
        if (action === 'profile_style') {
            const style = parseProfileStyle(body.profileStyle);
            if (!style) return fail('Estilo de perfil inválido.');
            if (!profileStyleAllowed(style, current.plan)) return fail('Essa personalização requer um plano ativo compatível.', 403);
            const saved = await db.prepare('SELECT profile_style FROM social_public_profiles WHERE user_id = ?').bind(me.id).first();
            const persisted = retainedProfileStyle(style, JSON.parse(saved?.profile_style || '{}'), current.plan);
            const result = await db.prepare("INSERT INTO social_public_profiles(user_id, description, banner, portrait, packs, display_name, status, collections, updated_at, profile_style) SELECT ?, '', '', '', '[]', '', '', '[]', ?, ? WHERE EXISTS (SELECT 1 FROM social_users WHERE id = ? AND removed_at = 0) ON CONFLICT(user_id) DO UPDATE SET profile_style = excluded.profile_style, updated_at = excluded.updated_at").bind(me.id, now, JSON.stringify(persisted), me.id).run();
            if (!result.meta.changes) return fail('Perfil indisponível.', 404);
            return json({ ok: true, profileStyle: activeProfileStyle(persisted, current.plan) });
        }
        if (action === 'preferences') {
            if (current.plan === 'free') return fail('A sincronização ampliada requer Plus ou Pro.', 403);
            if (!Number.isInteger(body.revision) || body.revision < 0) return fail('Revisão inválida.');
            const preferences = appearancePreferences(body.preferences);
            const saved = await db.prepare('SELECT presets FROM lux_membership_preferences WHERE social_id = ?').bind(me.id).first();
            const existing = JSON.parse(saved?.presets || '[]');
            if (!Array.isArray(body.presets) || body.presets.length > Math.max(current.presets, existing.length) || (body.presets.length > current.presets && body.presets.some((preset: any) => !existing.some((item: any) => item.id === preset?.id)))) return fail(`Use até ${current.presets} perfis de aparência.`);
            const ids = new Set<string>();
            const presets = body.presets.map((preset: any) => {
                if (!uuid(preset?.id) || ids.has(preset.id) || typeof preset.name !== 'string' || !preset.name.trim() || preset.name.length > 40) return fail('Perfil de aparência inválido.');
                ids.add(preset.id);
                return { id: preset.id, name: preset.name.trim(), preferences: appearancePreferences(preset.preferences) };
            });
            const changed = await db.prepare('INSERT INTO lux_membership_preferences(social_id, preferences, presets, revision) SELECT ?, ?, ?, 1 WHERE ? = 0 OR EXISTS (SELECT 1 FROM lux_membership_preferences WHERE social_id = ? AND revision = ?) ON CONFLICT(social_id) DO UPDATE SET preferences = excluded.preferences, presets = excluded.presets, revision = lux_membership_preferences.revision + 1 WHERE lux_membership_preferences.revision = ?').bind(me.id, JSON.stringify(preferences), JSON.stringify(presets), body.revision, me.id, body.revision, body.revision).run();
            if (!changed.meta.changes) return fail('As preferências mudaram em outro dispositivo. Atualize a página.', 409);
            return json({ ok: true, revision: body.revision + 1 });
        }
        if (action === 'create') {
            const config = await db.prepare('SELECT enabled, bank, pix_key FROM lux_commerce_config WHERE id = 1').first();
            if (!config?.enabled) return fail('Os pedidos de planos ainda não estão abertos.', 503);
            if (!Object.hasOwn(catalog, body.plan)) return fail('Plano inválido.');
            const active = await db.prepare(`${selectOrders} WHERE o.social_id = ? AND o.status IN ('awaiting_receipt','pending')`).bind(me.id).first();
            if (active) {
                if (active.plan !== body.plan) return fail('Cancele o pedido atual antes de escolher outro plano.', 409);
                return json({ order: publicOrder(active) });
            }
            if (await limited(db, `commerce-create:${me.id}`, 86400, 5)) return fail('Limite diário de pedidos atingido.', 429);
            const plan = catalog[body.plan as PaidPlan];
            const id = crypto.randomUUID();
            if (!validPixKey(config.pix_key)) return fail('O recebimento Pix precisa ser corrigido pelo dono antes de criar pedidos.');
            await db.prepare('INSERT INTO lux_purchase_orders(id, social_id, plan, amount, bank, pix_key, days, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING').bind(id, me.id, body.plan, plan.amount, config.bank, config.pix_key, plan.days, now, now).run();
            const order = await db.prepare(`${selectOrders} WHERE o.social_id = ? AND o.status IN ('awaiting_receipt','pending')`).bind(me.id).first();
            if (!order || order.plan !== body.plan) return fail('Já existe um pedido em andamento.', 409);
            return json({ order: publicOrder(order) });
        }
        if (action === 'cancel') {
            if (!uuid(body.id)) return fail('Pedido inválido.');
            const result = await db.prepare("UPDATE lux_purchase_orders SET status = 'cancelled', version = version + 1, updated_at = ? WHERE id = ? AND social_id = ? AND status IN ('awaiting_receipt','pending')").bind(now, body.id, me.id).run();
            if (!result.meta.changes) return fail('Este pedido não pode mais ser cancelado.', 409);
            return json({ ok: true });
        }
        if (action === 'owner_queue') {
            const offset = body.offset || 0;
            if (!Number.isInteger(offset) || offset < 0 || offset > 100000) return fail('Página inválida.');
            const status = body.status || 'pending';
            if (!['all', 'pending', 'approved', 'rejected', 'awaiting_receipt', 'cancelled'].includes(status)) return fail('Filtro inválido.');
            const clause = status === 'all' ? '' : ' WHERE o.status = ?';
            const values = status === 'all' ? [] : [status];
            const orders = await db.prepare(`${selectOrders}${clause} ORDER BY o.created_at DESC, o.id LIMIT 50 OFFSET ?`).bind(...values, offset).all();
            const total = await db.prepare(`SELECT COUNT(*) AS total FROM lux_purchase_orders o${clause}`).bind(...values).first();
            const stats = await db.prepare("SELECT SUM(status = 'pending') AS pending, SUM(status = 'approved') AS approved, SUM(CASE WHEN status = 'approved' THEN amount ELSE 0 END) AS approvedAmount FROM lux_purchase_orders").bind().first();
            const config = await db.prepare('SELECT enabled, bank, pix_key, owner_password_hash IS NOT NULL AS password_configured FROM lux_commerce_config WHERE id = 1').bind().first();
            const audit = await db.prepare('SELECT a.action, a.detail, a.created_at, a.order_id, a.social_id, u.username FROM lux_purchase_audit a LEFT JOIN social_users u ON u.id = a.social_id ORDER BY a.created_at DESC LIMIT 30').bind().all();
            return json({ orders: orders.results.map(publicOrder), total: total?.total || 0, stats, config: { enabled: Boolean(config?.enabled), bank: config?.bank, pixKey: config?.pix_key, passwordConfigured: Boolean(config?.password_configured) }, audit: audit.results });
        }
        if (action === 'owner_password') {
            if (!env.AUTH_PEPPER || env.AUTH_PEPPER.length < 32) return fail('Acesso temporariamente indisponível.', 503);
            const configured = await db.prepare('SELECT owner_password_hash FROM lux_commerce_config WHERE id = 1').first();
            if (configured?.owner_password_hash) await requireOwnerSession(request, db, me.id, true);
            if (typeof body.password !== 'string' || body.password.length < 14 || body.password.length > 128) return fail('Use uma senha de pelo menos 14 caracteres.');
            const salt = randomToken();
            const hash = await passwordHash(body.password, salt, env.AUTH_PEPPER);
            const token = randomToken();
            await db.batch([
                db.prepare('UPDATE lux_commerce_config SET owner_password_hash = ?, owner_password_salt = ? WHERE id = 1').bind(hash, salt),
                db.prepare('DELETE FROM lux_commerce_sessions WHERE social_id = ?').bind(me.id),
                db.prepare('INSERT INTO lux_commerce_sessions(token_hash, social_id, expires_at, verified_at, created_at, last_activity_at) VALUES (?, ?, ?, ?, ?, ?)').bind(await digest(token), me.id, now + 900, now, now, now),
                db.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)').bind(crypto.randomUUID(), me.id, me.id, 'password', 'Senha do painel alterada; outras sessões encerradas', now)
            ]);
            return json({ ok: true, csrfToken: await csrfFor(token) }, 200, { 'Set-Cookie': `${COMMERCE_COOKIE}=${token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=900` });
        }
        if (action === 'owner_revoke_sessions') {
            const session = await requireOwnerSession(request, db, me.id, true);
            await db.batch([
                db.prepare('DELETE FROM lux_commerce_sessions WHERE social_id = ? AND token_hash <> ?').bind(me.id, session.token_hash),
                db.prepare('DELETE FROM lux_commerce_tickets WHERE social_id = ?').bind(me.id),
                db.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)').bind(crypto.randomUUID(), me.id, me.id, 'security', 'Outras sessões e links de acesso encerrados', now)
            ]);
            return json({ ok: true });
        }
        if (action === 'owner_totp_begin') {
            if (!env.AUTH_PEPPER || env.AUTH_PEPPER.length < 32) return fail('Acesso temporariamente indisponível.', 503);
            const config = await db.prepare('SELECT owner_totp_secret FROM lux_commerce_config WHERE id = 1').first();
            if (config?.owner_totp_secret) return fail('A autenticação em duas etapas já está ativa.', 409);
            await ownerCredentials(request, env, body.password, body.otp);
            const secret = base32(crypto.getRandomValues(new Uint8Array(20)));
            await db.prepare('UPDATE lux_commerce_config SET owner_totp_pending = ?, owner_totp_pending_until = ? WHERE id = 1 AND owner_totp_secret IS NULL').bind(await encryptSecret(secret, env.AUTH_PEPPER), now + 600).run();
            const response = await newOwnerSession(request, env, 'Configuração do autenticador iniciada');
            return json({ ...(await response.json()), secret, uri: `otpauth://totp/Luxmc:Central%20do%20dono?secret=${secret}&issuer=Luxmc&algorithm=SHA1&digits=6&period=30` }, 200, { 'Set-Cookie': response.headers.get('Set-Cookie')! });
        }
        if (action === 'owner_totp_enable') {
            if (!env.AUTH_PEPPER || env.AUTH_PEPPER.length < 32) return fail('Acesso temporariamente indisponível.', 503);
            if (await limited(db, `owner-enroll:${me.id}`, 600, 5)) return fail('Aguarde antes de tentar novamente.', 429);
            const config = await db.prepare('SELECT owner_totp_pending, owner_totp_pending_until, owner_totp_secret FROM lux_commerce_config WHERE id = 1').first();
            if (!config?.owner_totp_pending || config.owner_totp_pending_until <= now || config.owner_totp_secret) return fail('A configuração expirou. Comece novamente.', 409);
            if (typeof body.otp !== 'string' || !/^\d{6}$/.test(body.otp)) return fail('Digite os seis números do autenticador.');
            const secret = await decryptSecret(config.owner_totp_pending, env.AUTH_PEPPER);
            const current = Math.floor(now / 30); let matched = -1;
            for (const step of [current, current - 1, current + 1]) if (equalHash(body.otp, await otpAt(secret, step))) { matched = step; break; }
            if (matched < 0) return fail('Código inválido. Confira o relógio do celular.');
            const recoveryCodes = Array.from({ length: 8 }, () => randomToken().slice(0, 32).toUpperCase());
            const hashes = await Promise.all(recoveryCodes.map(code => digest(`luxmc-owner-recovery:${code}`)));
            const enabled = await db.prepare('UPDATE lux_commerce_config SET owner_totp_secret = owner_totp_pending, owner_totp_pending = NULL, owner_totp_pending_until = 0, owner_totp_last_step = ?, owner_recovery_hashes = ? WHERE id = 1 AND owner_totp_secret IS NULL AND owner_totp_pending = ? AND owner_totp_pending_until > ?').bind(matched, JSON.stringify(hashes), config.owner_totp_pending, now).run();
            if (!enabled.meta.changes) return fail('A configuração mudou. Atualize antes de continuar.', 409);
            await db.prepare('DELETE FROM lux_commerce_sessions WHERE social_id = ?').bind(me.id).run();
            const response = await newOwnerSession(request, env, 'Autenticação em duas etapas ativada; outras sessões encerradas');
            const data = await response.json();
            return json({ ...data, recoveryCodes }, 200, { 'Set-Cookie': response.headers.get('Set-Cookie')! });
        }
        if (action === 'owner_totp_disable') {
            await ownerCredentials(request, env, body.password, body.otp);
            await db.batch([
                db.prepare("UPDATE lux_commerce_config SET owner_totp_secret = NULL, owner_totp_pending = NULL, owner_totp_pending_until = 0, owner_totp_last_step = -1, owner_recovery_hashes = '[]' WHERE id = 1"),
                db.prepare('DELETE FROM lux_commerce_sessions WHERE social_id = ?').bind(me.id),
                db.prepare('DELETE FROM lux_commerce_tickets WHERE social_id = ?').bind(me.id)
            ]);
            return newOwnerSession(request, env, 'Autenticação em duas etapas desativada; outras sessões encerradas');
        }
        if (action === 'owner_config') {
            if (typeof body.enabled !== 'boolean' || body.bank !== 'Nubank' || !validPixKey(body.pixKey)) return fail('Confira o banco e a chave Pix.');
            await db.batch([
                db.prepare('UPDATE lux_commerce_config SET enabled = ?, bank = ?, pix_key = ?, updated_at = ? WHERE id = 1').bind(Number(body.enabled), body.bank, body.pixKey, now),
                db.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) VALUES (?, ?, ?, ?, ?, ?)').bind(crypto.randomUUID(), me.id, me.id, 'config', body.enabled ? 'Pedidos abertos' : 'Pedidos pausados', now)
            ]);
            return json({ ok: true });
        }
        if (action === 'owner_review') {
            if (!uuid(body.id) || !Number.isInteger(body.version) || !['approve', 'reject'].includes(body.decision) || typeof body.note !== 'string' || body.note.length > 240) return fail('Decisão inválida.');
            const order = await db.prepare(`${selectOrders} WHERE o.id = ?`).bind(body.id).first();
            if (!order) return fail('Pedido não encontrado.', 404);
            if (order.status !== 'pending' || order.version !== body.version || !order.receipt_hash || body.receiptHash !== order.receipt_hash) return fail('Pedido ou comprovante alterado. Reabra antes de decidir.', 409);
            const approve = body.decision === 'approve';
            if (approve && (body.creditConfirmed !== true || typeof body.transactionId !== 'string' || !/^[A-Za-z0-9]{32}$/.test(body.transactionId))) return fail('Confira o crédito no banco e informe o EndToEndId de 32 caracteres da transação Pix.');
            if (approve && (!Number.isSafeInteger(body.receivedAmount) || body.receivedAmount !== order.amount)) return fail('O valor recebido no banco precisa corresponder ao valor deste pedido.');
            if (!approve && !body.note.trim()) return fail('Informe o motivo da rejeição.');
            const decision = crypto.randomUUID();
            const statements = [db.prepare("UPDATE lux_purchase_orders SET status = ?, version = version + 1, reviewed_by = ?, decision_id = ?, transaction_id = ?, note = ?, updated_at = ? WHERE id = ? AND status = 'pending' AND version = ? AND EXISTS (SELECT 1 FROM lux_purchase_receipts WHERE order_id = ? AND sha256 = ?)").bind(approve ? 'approved' : 'rejected', me.id, decision, approve ? body.transactionId : null, body.note.trim(), now, order.id, body.version, order.id, order.receipt_hash)];
            if (approve) statements.push(db.prepare('INSERT INTO lux_memberships(social_id, plan, expires_at, revision, change_id) SELECT social_id, plan, ? + days * 86400, 1, ? FROM lux_purchase_orders WHERE id = ? AND decision_id = ? ON CONFLICT(social_id) DO UPDATE SET plan = excluded.plan, expires_at = CASE WHEN lux_memberships.plan = excluded.plan THEN MAX(lux_memberships.expires_at, ?) + (excluded.expires_at - ?) ELSE excluded.expires_at END, revision = lux_memberships.revision + 1, change_id = excluded.change_id').bind(now, decision, order.id, decision, now, now));
            statements.push(db.prepare('INSERT INTO lux_purchase_audit(id, order_id, social_id, actor, action, detail, created_at) SELECT ?, id, social_id, ?, ?, ?, ? FROM lux_purchase_orders WHERE id = ? AND decision_id = ?').bind(decision, me.id, approve ? 'approve' : 'reject', `${catalog[order.plan as PaidPlan].name}: ${body.note.trim()}`, now, order.id, decision));
            let results;
            try { results = await db.batch(statements); } catch (error) {
                if (String(error).includes('UNIQUE')) return fail('Essa transação Pix já foi utilizada em outro pedido.', 409);
                throw error;
            }
            if (!results[0].meta.changes) return fail('O pedido já foi decidido em outra sessão.', 409);
            return json({ ok: true, membership: await membership(db, order.social_id) });
        }
        if (action === 'owner_find') {
            if (typeof body.query !== 'string' || body.query.trim().length < 3 || body.query.length > 64) return fail('Informe pelo menos 3 caracteres do usuário ou o código da conta.');
            const query = body.query.trim();
            const code = /^(.+)#([a-f0-9]{8})$/i.exec(query);
            const users = await db.prepare("SELECT u.id, u.username, COALESCE(p.display_name, '') AS display_name, CASE WHEN m.expires_at > ? THEN m.plan ELSE 'free' END AS plan, m.expires_at, COALESCE(m.revision, 0) AS revision FROM social_users u LEFT JOIN social_public_profiles p ON p.user_id = u.id LEFT JOIN lux_memberships m ON m.social_id = u.id WHERE u.removed_at = 0 AND (u.id = ? OR (? = 1 AND lower(u.username) = lower(?) AND lower(substr(u.id, 1, 8)) = lower(?)) OR (? = 0 AND instr(lower(u.username), lower(?)) > 0)) ORDER BY u.username, u.id LIMIT 20").bind(now, query, Number(Boolean(code)), code?.[1] || '', code?.[2] || '', Number(Boolean(code)), query).all();
            return json({ users: users.results.map(user => ({ ...user, friendCode: `${user.username}#${user.id.slice(0, 8)}` })) });
        }
        if (action === 'owner_grant') {
            if (!uuid(body.targetId) || !['free', 'plus', 'pro'].includes(body.plan) || !Number.isInteger(body.days) || body.days < 1 || body.days > 365 || !Number.isInteger(body.revision) || body.revision < 0 || (body.note !== undefined && (typeof body.note !== 'string' || body.note.length > 240))) return fail('Confira a conta e o prazo da alteração.');
            const target = await db.prepare('SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0').bind(body.targetId).first();
            if (!target) return fail('Perfil não encontrado.', 404);
            const friendCode = `${target.username}#${target.id.slice(0, 8)}`;
            const confirmation = typeof body.confirmationCode === 'string' ? body.confirmationCode.trim().toLowerCase() : '';
            if (confirmation !== target.id.toLowerCase() && confirmation !== friendCode.toLowerCase()) return fail('O código informado não corresponde à conta selecionada. Confira o código enviado pelo jogador.');
            const note = body.note?.trim() || 'Atribuição manual pela central do dono';
            const change = crypto.randomUUID();
            const results = await db.batch([
                db.prepare('INSERT INTO lux_memberships(social_id, plan, expires_at, revision, change_id) SELECT ?, ?, ?, 1, ? WHERE ? = 0 OR EXISTS (SELECT 1 FROM lux_memberships WHERE social_id = ? AND revision = ?) ON CONFLICT(social_id) DO UPDATE SET plan = excluded.plan, expires_at = excluded.expires_at, revision = lux_memberships.revision + 1, change_id = excluded.change_id WHERE lux_memberships.revision = ?').bind(target.id, body.plan === 'free' ? 'plus' : body.plan, body.plan === 'free' ? now : now + body.days * 86400, change, body.revision, target.id, body.revision, body.revision),
                db.prepare('INSERT INTO lux_purchase_audit(id, social_id, actor, action, detail, created_at) SELECT ?, social_id, ?, ?, ?, ? FROM lux_memberships WHERE social_id = ? AND change_id = ?').bind(change, me.id, 'grant', `${body.plan === 'free' ? 'Voltou ao Free' : `Luxmc ${body.plan}: ${body.days} dias`}. ${note}`, now, target.id, change)
            ]);
            if (!results[0].meta.changes) return fail('O plano mudou em outra sessão. Busque a conta novamente.', 409);
            return json({ ok: true, identity: { id: target.id, username: target.username, friendCode }, membership: await membership(db, target.id) });
        }
        return json({ error: 'Ação desconhecida.' }, 404);
    } catch (error) {
        const status = Number((error as { status?: number }).status) || 500;
        return json({ error: status < 500 ? (error as Error).message : 'Não foi possível concluir. Tente novamente.' }, status);
    }
}
