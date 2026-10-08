import { json, validNickname } from "../../../lib/accounts.js";
async function onRequest({ request, env, params }) {
  if (!["GET", "HEAD"].includes(request.method))
    return json({ error: "M\xE9todo n\xE3o permitido." }, 405);
  if (!validNickname(params.name))
    return json({ error: "Jogador inv\xE1lido." }, 400);
  if (!env?.SOCIAL_DB)
    return json({ error: "Servi\xE7o indispon\xEDvel." }, 503);
  const asset = new URL(request.url).searchParams.get("asset") || "skin";
  if (!["skin", "cape"].includes(asset))
    return json({ error: "Textura inv\xE1lida." }, 400);
  const row = await env.SOCIAL_DB.prepare("SELECT p.skin, p.cape, p.model, p.updated_at FROM lux_appearance p JOIN lux_accounts a ON a.id = p.account_id WHERE a.username = ? COLLATE NOCASE").bind(params.name).first();
  if (!row || !row[asset])
    return json({ error: "Textura n\xE3o publicada." }, 404);
  const bytes = Uint8Array.from(atob(row[asset].slice(22)), (c) => c.charCodeAt(0));
  return new Response(request.method === "HEAD" ? null : bytes, { headers: { "Content-Type": "image/png", "Content-Length": String(bytes.length), "Cache-Control": "public, max-age=60", "X-Content-Type-Options": "nosniff", "X-Luxmc-Model": row.model, "Access-Control-Allow-Origin": "*" } });
}
export {
  onRequest
};
