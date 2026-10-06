import { json, validNickname } from "../../../lib/accounts.js";

export async function onRequest({ request, env, params }) {
  if (!["GET", "HEAD"].includes(request.method)) return json({ error: "Método não permitido." }, 405);
  if (!validNickname(params.name)) return json({ error: "Jogador inválido." }, 400);
  if (!env?.SOCIAL_DB) return json({ error: "Serviço indisponível." }, 503);
  const asset = new URL(request.url).searchParams.get("asset") || "skin";
  if (!["skin", "cape"].includes(asset)) return json({ error: "Textura inválida." }, 400);
  const row = await env.SOCIAL_DB.prepare("SELECT p.skin, p.cape, p.model, p.updated_at FROM lux_appearance p JOIN lux_accounts a ON a.id = p.account_id WHERE a.username = ? COLLATE NOCASE").bind(params.name).first();
  if (!row || !row[asset]) return json({ error: "Textura não publicada." }, 404);
  const bytes = Uint8Array.from(atob(row[asset].slice(22)), c => c.charCodeAt(0));
  return new Response(request.method === "HEAD" ? null : bytes, { headers: { "Content-Type": "image/png", "Content-Length": String(bytes.length), "Cache-Control": "public, max-age=60", "X-Content-Type-Options": "nosniff", "X-Luxmc-Model": row.model, "Access-Control-Allow-Origin": "*" } });
}
