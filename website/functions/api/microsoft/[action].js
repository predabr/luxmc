import { cookie, digest, json, limited, randomToken, sameOrigin, SESSION_SECONDS } from "../../../lib/accounts.js";
import { commerceIdentity } from "../../../lib/membership.js";
import { microsoftAccount, microsoftBrowser, microsoftConfiguration, microsoftCookie, microsoftProfile } from "../../../lib/microsoft.js";
async function onRequest({ request, env, params, waitUntil }) {
  const action = params.action;
  const configuration = microsoftConfiguration(env);
  if (action === "status" && request.method === "GET") return json({ enabled: Boolean(configuration) });
  if (!["start", "callback"].includes(action)) return json({ error: "A\xE7\xE3o desconhecida." }, 404);
  if (request.method !== (action === "start" ? "POST" : "GET")) return json({ error: "M\xE9todo n\xE3o permitido." }, 405);
  if (!configuration || !env.SOCIAL_DB) return json({ error: "O login Microsoft do site ainda precisa ser habilitado pelo respons\xE1vel do Luxmc." }, 503);
  const db = env.SOCIAL_DB;
  const now = Math.floor(Date.now() / 1e3);
  const redirect = (reason = "") => new Response(null, { status: 303, headers: { Location: `/conta${reason ? `?microsoft_error=${reason}` : ""}`, "Set-Cookie": microsoftCookie("", 0), "Cache-Control": "no-store", "Referrer-Policy": "no-referrer" } });
  try {
    if (action === "start") {
      if (!sameOrigin(request) || request.headers.get("Origin") !== new URL(configuration.redirect).origin) return json({ error: "Origem n\xE3o permitida." }, 403);
      if (await limited(db, `microsoft:${await digest(request.headers.get("CF-Connecting-IP") || "local")}`, 900, 20)) return json({ error: "Muitas tentativas. Aguarde alguns minutos." }, 429);
      const state2 = randomToken(), browser2 = randomToken(), verifier = randomToken();
      const current = await commerceIdentity(request, db);
      await db.prepare("INSERT INTO lux_microsoft_pending(state_hash, browser_hash, verifier, social_id, expires_at) VALUES (?, ?, ?, ?, ?)").bind(await digest(state2), await digest(browser2), verifier, current?.id || null, now + 600).run();
      waitUntil(db.prepare("DELETE FROM lux_microsoft_pending WHERE expires_at < ?").bind(now).run());
      const challenge = btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier))))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      const url2 = new URL("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize");
      url2.search = new URLSearchParams({ client_id: configuration.clientId, response_type: "code", redirect_uri: configuration.redirect, response_mode: "query", scope: "XboxLive.signin", state: state2, code_challenge: challenge, code_challenge_method: "S256", prompt: "select_account" }).toString();
      return json({ url: url2.href }, 200, { "Set-Cookie": microsoftCookie(browser2) });
    }
    const url = new URL(request.url), state = url.searchParams.get("state") || "", browser = microsoftBrowser(request);
    if (!/^[a-f0-9]{64}$/.test(state) || !/^[a-f0-9]{64}$/.test(browser)) return redirect("expired");
    const pending = await db.prepare("DELETE FROM lux_microsoft_pending WHERE state_hash = ? AND browser_hash = ? AND expires_at > ? RETURNING verifier, social_id").bind(await digest(state), await digest(browser), now).first();
    if (!pending) return redirect("expired");
    if (url.searchParams.has("error")) return redirect("cancelled");
    const code = url.searchParams.get("code");
    if (!code || code.length > 16e3) return redirect("expired");
    const profile = await microsoftProfile(code, pending.verifier, configuration);
    const account = await microsoftAccount(db, profile, pending.social_id, now);
    if (!account) return redirect("unavailable");
    const session = randomToken();
    await db.prepare("INSERT INTO lux_sessions(token_hash, account_id, created_at, expires_at, client) VALUES (?, ?, ?, ?, 'web')").bind(await digest(session), account.id, now, now + SESSION_SECONDS).run();
    await db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash NOT IN (SELECT token_hash FROM lux_sessions WHERE account_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 20)").bind(account.id, account.id).run();
    const response = redirect();
    response.headers.append("Set-Cookie", cookie(session));
    response.headers.append("Set-Cookie", "__Host-luxmc-commerce=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0");
    return response;
  } catch (error) {
    return action === "callback" ? redirect(["minecraft", "conflict"].includes(error?.reason) ? error.reason : "unavailable") : json({ error: "N\xE3o foi poss\xEDvel iniciar o login Microsoft. Tente novamente." }, 503);
  }
}
export {
  onRequest
};
