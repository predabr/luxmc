import type { SiteContext } from '../../../lib/types.js';
import { hex, json, limited, sameOrigin } from '../../../lib/accounts.js';
import { commerceIdentity, fail, nowSeconds } from '../../../lib/membership.js';
import { requireOwnerSession, verifyCsrf } from '../../../lib/owner-security.js';

function fileType(bytes: Uint8Array): string | null {
    const prefix = new TextDecoder('latin1').decode(bytes.slice(0, 16));
    if (bytes.length >= 33 && bytes[0] === 137 && prefix.slice(1, 8) === 'PNG\r\n\u001a\n') {
        const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
        const width = view.getUint32(16), height = view.getUint32(20);
        return width > 0 && height > 0 && width * height <= 20000000 ? 'image/png' : null;
    }
    if (bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255 && bytes.at(-2) === 255 && bytes.at(-1) === 217) return 'image/jpeg';
    if (prefix.startsWith('RIFF') && prefix.slice(8,12) === 'WEBP' && new DataView(bytes.buffer, bytes.byteOffset).getUint32(4, true) + 8 === bytes.length) return 'image/webp';
    if (/^%PDF-1\.[0-9]/.test(prefix) && new TextDecoder().decode(bytes.slice(-1024)).includes('%%EOF')) return 'application/pdf';
    return null;
}
export async function onRequest({ request, env, params }: SiteContext) {
    if (!['GET', 'POST'].includes(request.method)) return json({ error: 'Método inválido.' }, 405);
    if (!sameOrigin(request)) return json({ error: 'Origem não permitida.' }, 403);
    try {
        const db = env.SOCIAL_DB;
        const me = await commerceIdentity(request, db);
        if (!me) return fail('Entre novamente.', 401);
        if (!/^[a-f0-9-]{36}$/.test(params.id)) return fail('Pedido inválido.');
        const order = await db.prepare('SELECT social_id, status, version FROM lux_purchase_orders WHERE id = ?').bind(params.id).first();
        if (!order || (order.social_id !== me.id && me.id !== env.OWNER_SOCIAL_ID)) return fail('Comprovante indisponível.', 404);
        if (me.id === env.OWNER_SOCIAL_ID && order.social_id !== me.id) await requireOwnerSession(request, db, me.id);
        if (request.method === 'GET') {
            const receipt = await db.prepare('SELECT bytes, mime FROM lux_purchase_receipts WHERE order_id = ?').bind(params.id).first();
            if (!receipt) return fail('Comprovante não encontrado.', 404);
            return new Response(new Uint8Array(receipt.bytes), { headers: { 'Content-Type': receipt.mime, 'Content-Disposition': `${receipt.mime === 'application/pdf' ? 'attachment' : 'inline'}; filename="comprovante.${receipt.mime === 'application/pdf' ? 'pdf' : receipt.mime.split('/')[1]}"`, 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff', 'Content-Security-Policy': "default-src 'none'; sandbox; frame-ancestors 'none'", 'X-Robots-Tag': 'noindex, nofollow', 'Referrer-Policy': 'no-referrer' } });
        }
        if (order.social_id !== me.id) return fail('Somente o comprador pode enviar o comprovante.', 403);
        await verifyCsrf(request);
        if (!['awaiting_receipt', 'pending'].includes(order.status)) return fail('Este pedido já foi concluído.', 409);
        if (await limited(db, `receipt:${me.id}`, 86400, 8)) return fail('Limite diário de envios atingido.', 429);
        if (Number(request.headers.get('Content-Length')) > 1048576) return fail('Envie um arquivo de até 1 MB.', 413);
        const reader = request.body?.getReader();
        if (!reader) return fail('Arquivo vazio.');
        const chunks: Uint8Array[] = []; let length = 0;
        for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            length += value.byteLength;
            if (length > 1048576) { await reader.cancel(); return fail('Envie um arquivo de até 1 MB.', 413); }
            chunks.push(value);
        }
        if (length < 12) return fail('Arquivo inválido.');
        const bytes = new Uint8Array(length); let offset = 0;
        for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
        const mime = fileType(bytes);
        if (!mime || request.headers.get('Content-Type') !== mime) return fail('Use um comprovante PNG, JPEG, WebP ou PDF válido.', 415);
        const space = await db.prepare('SELECT COALESCE(SUM(size), 0) AS used FROM lux_purchase_receipts').bind().first();
        if (space && space.used + length > 100 * 1048576) return fail('O armazenamento está cheio. Contate o Luxmc.', 503);
        const sha256 = hex(await crypto.subtle.digest('SHA-256', bytes));
        const marker = crypto.randomUUID();
        const now = nowSeconds();
        const results = await db.batch([
            db.prepare("UPDATE lux_purchase_orders SET status = 'pending', version = version + 1, updated_at = ?, decision_id = ? WHERE id = ? AND social_id = ? AND version = ? AND status IN ('awaiting_receipt','pending') AND (SELECT COALESCE(SUM(size), 0) FROM lux_purchase_receipts) + ? - COALESCE((SELECT size FROM lux_purchase_receipts WHERE order_id = ?), 0) <= ?").bind(now, marker, params.id, me.id, order.version, length, params.id, 100 * 1048576),
            db.prepare('INSERT INTO lux_purchase_receipts(order_id, bytes, mime, size, sha256, uploaded_at) SELECT id, ?, ?, ?, ?, ? FROM lux_purchase_orders WHERE id = ? AND decision_id = ? ON CONFLICT(order_id) DO UPDATE SET bytes = excluded.bytes, mime = excluded.mime, size = excluded.size, sha256 = excluded.sha256, uploaded_at = excluded.uploaded_at').bind(bytes.buffer, mime, length, sha256, now, params.id, marker)
        ]);
        if (!results[0].meta.changes) return fail('O pedido mudou. Atualize antes de enviar.', 409);
        return json({ ok: true });
    } catch (error) {
        const status = Number((error as { status?: number }).status) || 500;
        return json({ error: status < 500 ? (error as Error).message : 'Não foi possível guardar o comprovante.' }, status);
    }
}
