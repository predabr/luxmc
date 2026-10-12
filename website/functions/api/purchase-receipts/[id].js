import { json, sameOrigin } from "../../../lib/accounts.js";
import { catalog, commerceIdentity, fail } from "../../../lib/membership.js";
import { requireOwnerSession } from "../../../lib/owner-security.js";
const escape = (value) => String(value).replace(/[&<>"']/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[character]);
async function onRequest({ request, env, params }) {
  if (request.method !== "GET") return json({ error: "Use GET." }, 405);
  if (!sameOrigin(request)) return json({ error: "Origem n\xE3o permitida." }, 403);
  try {
    const db = env.SOCIAL_DB;
    if (!db) return fail("Servi\xE7o indispon\xEDvel.", 503);
    const me = await commerceIdentity(request, db);
    if (!me) return fail("Entre novamente.", 401);
    if (!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/.test(params.id)) return fail("Pedido inv\xE1lido.");
    const order = await db.prepare("SELECT o.*, u.username FROM lux_purchase_orders o JOIN social_users u ON u.id = o.social_id WHERE o.id = ?").bind(params.id).first();
    if (!order || order.social_id !== me.id && me.id !== env.OWNER_SOCIAL_ID) return fail("Recibo indispon\xEDvel.", 404);
    if (me.id === env.OWNER_SOCIAL_ID) await requireOwnerSession(request, db, me.id);
    if (order.status !== "approved" || !order.transaction_id || !order.reviewed_by || !order.decision_id) return fail("O recibo fica dispon\xEDvel ap\xF3s a confirma\xE7\xE3o do pagamento.", 409);
    const amount = (order.amount / 100).toLocaleString("pt-BR", { style: "currency", currency: "BRL" });
    const approved = new Date(order.updated_at * 1e3).toLocaleString("pt-BR", { timeZone: "America/Sao_Paulo" });
    const rows = [["Conta beneficiada", `${order.username}#${order.social_id.slice(0, 8)}`], ["Plano", catalog[order.plan].name], ["Per\xEDodo adquirido", `${order.days} dias`], ["Forma de pagamento", `Pix \xB7 ${order.bank}`], ["Confirmado em", `${approved} (Bras\xEDlia)`], ["Pedido", order.id], ["Transa\xE7\xE3o Pix (EndToEndId)", order.transaction_id]];
    const html = `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="robots" content="noindex,nofollow"><title>Recibo Luxmc \xB7 ${escape(order.id)}</title><link rel="stylesheet" href="/design.css"><link rel="stylesheet" href="/commerce.css"></head><body><main class="service-receipt"><p class="eyebrow">LUXMC \xB7 PAGAMENTO CONFERIDO</p><h1>Seu plano, confirmado.</h1><p>Recibo da assinatura Luxmc</p><strong class="receipt-total">${escape(amount)}</strong><dl>${rows.map(([label, value]) => `<div><dt>${escape(label)}</dt><dd>${escape(value)}</dd></div>`).join("")}</dl><p>Cr\xE9dito conferido manualmente pelo respons\xE1vel pelo Luxmc. Este recibo registra a assinatura liberada; o comprovante banc\xE1rio original \xE9 emitido pelo banco.</p><p class="receipt-print">Para salvar em PDF ou imprimir, use a op\xE7\xE3o Imprimir do navegador (Ctrl + P).</p><a href="/planos">Voltar aos planos</a></main></body></html>`;
    return new Response(html, { headers: { "Content-Type": "text/html; charset=utf-8", "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff", "Referrer-Policy": "no-referrer", "Content-Security-Policy": "default-src 'none'; style-src 'self'; font-src 'self'; script-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'" } });
  } catch (error) {
    const status = Number(error.status) || 500;
    return json({ error: status < 500 ? error.message : "Recibo indispon\xEDvel." }, status);
  }
}
export {
  onRequest
};
