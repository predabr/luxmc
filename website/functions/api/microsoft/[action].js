import { cookie, digest, json, limited, randomToken, readJson, sameOrigin, SESSION_SECONDS } from "../../../lib/accounts.js";
import { commerceIdentity } from "../../../lib/membership.js";
import { minecraftProfileFromMicrosoftToken, microsoftBrowser, microsoftConfiguration, microsoftCookie } from "../../../lib/microsoft.js";
import { linkMicrosoftIdentity, microsoftLinkCookie, microsoftLinkPending, microsoftLinkToken, microsoftNonce, microsoftSubjectAccount, verifiedMicrosoftIdentity } from "../../../lib/microsoft-identity.js";
import { startMicrosoftLauncher, completeMicrosoftLauncher } from "../../../lib/microsoft-launcher.js";
async function onRequest({ request, env, params, waitUntil }) {
  const action = params.action;
  const configuration = microsoftConfiguration(env);
  if (action === "status" && request.method === "GET") return json({ enabled: Boolean(configuration) });
  if (!["start", "callback", "link_status", "link", "cancel_link", "launcher_start", "launcher_complete"].includes(action)) return json({ error: "A\xE7\xE3o desconhecida." }, 404);
  if (request.method !== (action === "callback" ? "GET" : "POST")) return json({ error: "M\xE9todo n\xE3o permitido." }, 405);
  if (!configuration || !env.SOCIAL_DB) return json({ error: "O login Microsoft do site ainda precisa ser habilitado pelo respons\xE1vel do Luxmc." }, 503);
  const db = env.SOCIAL_DB;
  const now = Math.floor(Date.now() / 1e3);
  const callbackState = new URL(request.url).searchParams.get("state") || "";
  const redirect = (reason = "", linking = false) => new Response(null, { status: 303, headers: { Location: `/conta${linking ? "?microsoft_link=1" : reason ? `?microsoft_error=${reason}` : ""}`, "Set-Cookie": microsoftCookie("", 0, callbackState), "Cache-Control": "no-store", "Referrer-Policy": "no-referrer" } });
  const sessionResponse = async (account, callback, payload = { ok: true }) => {
    const session = randomToken();
    await db.prepare("INSERT INTO lux_sessions(token_hash, account_id, created_at, expires_at, client) VALUES (?, ?, ?, ?, 'web')").bind(await digest(session), account.id, now, now + SESSION_SECONDS).run();
    await db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash NOT IN (SELECT token_hash FROM lux_sessions WHERE account_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 20)").bind(account.id, account.id).run();
    const response = callback ? redirect() : json(payload);
    response.headers.append("Set-Cookie", cookie(session));
    response.headers.append("Set-Cookie", microsoftLinkCookie("", 0));
    response.headers.append("Set-Cookie", "__Host-luxmc-commerce=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0");
    return response;
  };
  try {
    if (action !== "callback") {
      const launcher = action.startsWith("launcher_") && /^Bearer [a-f0-9]{64}$/.test(request.headers.get("Authorization") || "");
      if (!sameOrigin(request) || !launcher && request.headers.get("Origin") !== new URL(configuration.redirect).origin) return json({ error: "Origem n\xE3o permitida." }, 403);
      const polling = action === "link_status";
      if (await limited(db, `microsoft:${polling ? "status:" : ""}${await digest(request.headers.get("CF-Connecting-IP") || "local")}`, polling ? 60 : 900, polling ? 30 : 60)) return json({ error: "Muitas tentativas. Aguarde alguns minutos." }, 429);
    }
    if (action.startsWith("launcher_")) {
      const body = await readJson(request, 4e4);
      return json(action === "launcher_start" ? await startMicrosoftLauncher(request, db, body, configuration, now) : await completeMicrosoftLauncher(request, db, body, now));
    }
    if (["link_status", "link", "cancel_link"].includes(action)) {
      const pending2 = await microsoftLinkPending(request, db, now);
      if (action === "cancel_link") {
        const token = microsoftLinkToken(request);
        if (/^[a-f0-9]{64}$/.test(token)) await db.prepare("DELETE FROM lux_microsoft_link_pending WHERE token_hash = ?").bind(await digest(token)).run();
        return json({ ok: true }, 200, { "Set-Cookie": microsoftLinkCookie("", 0) });
      }
      if (!pending2) return action === "link_status" ? json({ pending: false }) : json({ error: "A confirma\xE7\xE3o da Microsoft expirou. Entre com Microsoft novamente." }, 401);
      if (action === "link_status") {
        const account3 = await microsoftSubjectAccount(db, pending2.subject_hash);
        if (account3) return await sessionResponse(account3, false, { pending: false, linked: true });
      }
      const identity2 = await commerceIdentity(request, db);
      if (action === "link_status") return json({ pending: true, confirmation: pending2.token_hash, microsoft: pending2.display_name, minecraft: pending2.minecraft_uuid ? { uuid: pending2.minecraft_uuid, username: pending2.minecraft_name } : null, profile: identity2 ? { username: identity2.username } : null });
      const body = await readJson(request);
      if (body.confirmation !== pending2.token_hash) return json({ error: "A conta Microsoft mudou em outra aba. Recarregue esta p\xE1gina antes de confirmar." }, 409);
      if (!identity2) return json({ error: "Entre no perfil Luxmc que deseja vincular. No launcher, use Meu plano e pedidos e volte \xE0 p\xE1gina Conta." }, 401);
      const account2 = await linkMicrosoftIdentity(db, { token_hash: pending2.token_hash, subject_hash: pending2.subject_hash }, identity2.id, now);
      if (!account2) throw new Error("Linked account unavailable");
      return await sessionResponse(account2, false);
    }
    if (action === "start") {
      const state2 = randomToken(), browser2 = randomToken(), verifier = randomToken();
      const current = await commerceIdentity(request, db);
      await db.prepare("INSERT INTO lux_microsoft_pending(state_hash, browser_hash, verifier, social_id, expires_at) VALUES (?, ?, ?, ?, ?)").bind(await digest(state2), await digest(browser2), verifier, current?.id || null, now + 600).run();
      waitUntil(db.batch([
        db.prepare("DELETE FROM lux_microsoft_pending WHERE expires_at < ?").bind(now),
        db.prepare("DELETE FROM lux_microsoft_link_pending WHERE expires_at < ?").bind(now)
      ]));
      const challenge = btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier))))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      const url2 = new URL("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize");
      url2.search = new URLSearchParams({ client_id: configuration.clientId, response_type: "code", redirect_uri: configuration.redirect, response_mode: "query", scope: "openid profile XboxLive.signin", state: state2, nonce: await microsoftNonce(verifier), code_challenge: challenge, code_challenge_method: "S256", prompt: "select_account" }).toString();
      return json({ url: url2.href }, 200, { "Set-Cookie": microsoftCookie(browser2, 600, state2) });
    }
    const url = new URL(request.url), state = url.searchParams.get("state") || "", browser = microsoftBrowser(request, state);
    if (!/^[a-f0-9]{64}$/.test(state) || !/^[a-f0-9]{64}$/.test(browser)) return redirect("expired");
    const pending = await db.prepare("DELETE FROM lux_microsoft_pending WHERE state_hash = ? AND browser_hash = ? AND expires_at > ? RETURNING verifier, social_id").bind(await digest(state), await digest(browser), now).first();
    if (!pending) return redirect("expired");
    if (url.searchParams.has("error")) return redirect("cancelled");
    const code = url.searchParams.get("code");
    if (!code || code.length > 16e3) return redirect("expired");
    const identity = await verifiedMicrosoftIdentity(code, pending.verifier, configuration);
    const account = await microsoftSubjectAccount(db, identity.subject);
    if (account) {
      if (pending.social_id && account.social_id !== pending.social_id) return redirect("conflict");
      return await sessionResponse(account, true);
    }
    const ticket = randomToken();
    await db.prepare("INSERT INTO lux_microsoft_link_pending(token_hash, subject_hash, display_name, expires_at) VALUES (?, ?, ?, ?)").bind(await digest(ticket), identity.subject, identity.label, now + 7 * 86400).run();
    if (identity.accessToken) {
      waitUntil(minecraftProfileFromMicrosoftToken(identity.accessToken).then((profile) => db.prepare("UPDATE lux_microsoft_link_pending SET minecraft_uuid = ?, minecraft_name = ? WHERE subject_hash = ? AND expires_at > ?").bind(profile.uuid, profile.username, identity.subject, now).run()).catch(() => {
      }));
    }
    const response = redirect("", true);
    response.headers.append("Set-Cookie", microsoftLinkCookie(ticket));
    return response;
  } catch (error) {
    console.error("Microsoft login failed", { action, service: error?.service || "identity", status: Number.isInteger(error?.status) ? error.status : null, providerCode: error?.providerCode || null, providerReason: error?.providerReason || null });
    const reason = error?.reason === "conflict" ? "conflict" : "unavailable";
    return action === "callback" ? redirect(reason) : json({ error: reason === "conflict" ? "Essa conta Microsoft ou esse perfil Luxmc j\xE1 possui outro v\xEDnculo. Entre na identidade original para continuar." : "N\xE3o foi poss\xEDvel concluir o acesso Microsoft. Tente novamente." }, reason === "conflict" ? 409 : 503);
  }
}
export {
  onRequest
};
