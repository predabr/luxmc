import { authenticate, cookie, digest, equalHash, json, limited, passwordHash, preferences, publicAccount, randomToken, readJson, sameOrigin, SESSION_SECONDS, validNickname, validPassword } from "../../../lib/accounts.js";
import { texture } from "../../../lib/appearance.js";
import { nicknameAvailability } from "../../../lib/nicknames.js";
async function onRequest({ request, env, params, waitUntil }) {
  try {
    if (request.method !== "POST")
      return json({ error: "M\xE9todo n\xE3o permitido." }, 405);
    if (!sameOrigin(request))
      return json({ error: "Origem n\xE3o permitida." }, 403);
    if (!env || !env.SOCIAL_DB)
      return json({ error: "O banco D1 (SOCIAL_DB) n\xE3o est\xE1 vinculado \xE0s Fun\xE7\xF5es no painel do Cloudflare Pages. Vincule o D1 em Settings -> Functions -> D1 Bindings e fa\xE7a um novo deploy." }, 503);
    if (typeof env.AUTH_PEPPER !== "string" || env.AUTH_PEPPER.length < 32)
      return json({ error: "A vari\xE1vel AUTH_PEPPER (segredo com no m\xEDnimo 32 caracteres) n\xE3o est\xE1 configurada no Cloudflare Pages. Adicione em Settings -> Environment Variables e fa\xE7a um novo deploy." }, 503);
    const db = env.SOCIAL_DB;
    const pepper = env.AUTH_PEPPER;
    const action = params.action;
    if (!["register", "nickname", "login", "me", "logout", "sync", "password", "recover", "appearance"].includes(action))
      return json({ error: "A\xE7\xE3o desconhecida." }, 404);
    try {
      const body = await readJson(request, action === "appearance" ? 365e3 : 8192);
      const ip = await digest(request.headers.get("CF-Connecting-IP") || "local");
      if (await limited(db, `account:${ip}`, 60, 120))
        return json({ error: "Muitas requisi\xE7\xF5es. Aguarde um minuto." }, 429);
      const now = Math.floor(Date.now() / 1e3);
      if (action === "nickname") {
        if (!validNickname(body.username))
          return json({ error: "Use um nickname de 3 a 16 letras, n\xFAmeros ou _." }, 400);
        if (await limited(db, `nickname:${ip}`, 60, 12))
          return json({ error: "Aguarde um minuto para consultar outro nome." }, 429);
        return json(await nicknameAvailability(db, body.username));
      }
      if (Math.random() < 0.02)
        waitUntil(Promise.all([
          db.prepare("DELETE FROM lux_sessions WHERE expires_at < ?").bind(now).run(),
          db.prepare("DELETE FROM social_limits WHERE expires_at < ?").bind(now).run()
        ]));
      if (["register", "login", "recover"].includes(action)) {
        if (!validNickname(body.username))
          return json({ error: "Use um nickname de 3 a 16 letras, n\xFAmeros ou _." }, 400);
        if (!validPassword(body.password))
          return json({ error: "A senha deve ter entre 8 e 128 caracteres." }, 400);
        const username = body.username.trim();
        if (await limited(db, `auth:${ip}`, 900, 30) || await limited(db, `auth-name:${await digest(username.toLowerCase())}`, 900, 20))
          return json({ error: "Muitas tentativas. Aguarde 15 minutos." }, 429);
        let user2 = await db.prepare("SELECT * FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(username).first();
        let recoveryCode;
        if (action === "register") {
          if (await limited(db, `signup:${ip}`, 3600, 5))
            return json({ error: "Limite de cadastros atingido. Tente mais tarde." }, 429);
          const availability = await nicknameAvailability(db, username);
          if (!availability.available)
            return json({ error: availability.reason === "official" ? "Esse nome pertence a um jogador oficial do Minecraft. Escolha outro nickname." : "Esse nickname j\xE1 est\xE1 cadastrado.", ...availability }, 409);
          const id = crypto.randomUUID();
          const socialId = crypto.randomUUID();
          const salt = randomToken();
          recoveryCode = randomToken();
          const password = await passwordHash(body.password, salt, pepper);
          try {
            await db.batch([
              db.prepare("INSERT INTO social_users(id, token_hash, username) VALUES (?, ?, ?)").bind(socialId, await digest(randomToken()), username),
              db.prepare("INSERT INTO lux_accounts(id, username, password_hash, password_salt, recovery_hash, social_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)").bind(id, username, password, salt, await digest(recoveryCode), socialId, now)
            ]);
          } catch (error) {
            if (await db.prepare("SELECT id FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(username).first())
              return json({ error: "Esse nickname j\xE1 est\xE1 cadastrado." }, 409);
            throw error;
          }
          user2 = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(id).first();
        } else if (action === "recover") {
          if (typeof body.recoveryCode !== "string" || !/^[a-f0-9]{64}$/.test(body.recoveryCode) || !user2 || !equalHash(await digest(body.recoveryCode), user2.recovery_hash))
            return json({ error: "Nickname ou c\xF3digo de recupera\xE7\xE3o inv\xE1lido." }, 401);
          const salt = randomToken();
          recoveryCode = randomToken();
          const result = await db.batch([
            db.prepare("UPDATE lux_accounts SET password_hash = ?, password_salt = ?, recovery_hash = ? WHERE id = ? AND recovery_hash = ?").bind(await passwordHash(body.password, salt, pepper), salt, await digest(recoveryCode), user2.id, user2.recovery_hash),
            db.prepare("DELETE FROM lux_sessions WHERE account_id = ?").bind(user2.id)
          ]);
          if (!result[0].meta.changes)
            return json({ error: "C\xF3digo de recupera\xE7\xE3o j\xE1 utilizado." }, 401);
          user2 = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(user2.id).first();
        } else {
          const check = await passwordHash(body.password, user2?.password_salt || "luxmc-dummy-salt-for-missing-user", pepper);
          if (!user2 || !equalHash(check, user2.password_hash))
            return json({ error: "Nickname ou senha incorretos." }, 401);
        }
        if (!user2) return json({ error: "Conta indispon\xEDvel. Entre novamente." }, 401);
        const token = randomToken();
        const client = body.client === "launcher" ? "launcher" : "web";
        const session = await db.prepare("INSERT INTO lux_sessions(token_hash, account_id, created_at, expires_at, client) SELECT ?, id, ?, ?, ? FROM lux_accounts WHERE id = ? AND password_hash = ?").bind(await digest(token), now, now + SESSION_SECONDS, client, user2.id, user2.password_hash).run();
        if (!session.meta.changes)
          return json({ error: "A senha mudou. Entre novamente." }, 401);
        await db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash NOT IN (SELECT token_hash FROM lux_sessions WHERE account_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 20)").bind(user2.id, user2.id).run();
        return json({ account: publicAccount(user2), ...client === "launcher" ? { token } : {}, ...recoveryCode ? { recoveryCode } : {} }, 200, client === "web" ? { "Set-Cookie": cookie(token) } : {});
      }
      const user = await authenticate(request, db);
      if (!user)
        return json({ error: "Sua sess\xE3o expirou. Entre novamente." }, 401);
      if (action === "appearance") {
        if (!["classic", "slim"].includes(body.model))
          return json({ error: "Modelo inv\xE1lido." }, 400);
        if (await limited(db, `appearance:${user.id}`, 60, 10))
          return json({ error: "Aguarde um minuto antes de publicar novamente." }, 429);
        const skin = texture(body.skin), cape = texture(body.cape ?? null, true);
        await db.prepare("INSERT INTO lux_appearance(account_id, skin, cape, model, updated_at) VALUES (?, ?, ?, ?, ?) ON CONFLICT(account_id) DO UPDATE SET skin = excluded.skin, cape = excluded.cape, model = excluded.model, updated_at = excluded.updated_at").bind(user.id, skin, cape, body.model, now).run();
        return json({ ok: true });
      }
      if (action === "logout") {
        await db.prepare("DELETE FROM lux_sessions WHERE token_hash = ?").bind(user.session_hash).run();
        return json({ ok: true }, 200, { "Set-Cookie": cookie("", 0) });
      }
      if (action === "password") {
        if (!validPassword(body.password) || typeof body.currentPassword !== "string" || body.currentPassword.length > 128)
          return json({ error: "Confira a senha atual e use ao menos 8 caracteres na nova senha." }, 400);
        if (await limited(db, `password:${user.id}`, 900, 10))
          return json({ error: "Muitas tentativas. Aguarde 15 minutos." }, 429);
        if (!equalHash(await passwordHash(body.currentPassword, user.password_salt, pepper), user.password_hash))
          return json({ error: "Senha atual incorreta." }, 401);
        const salt = randomToken();
        await db.batch([
          db.prepare("UPDATE lux_accounts SET password_hash = ?, password_salt = ? WHERE id = ?").bind(await passwordHash(body.password, salt, pepper), salt, user.id),
          db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash != ?").bind(user.id, user.session_hash)
        ]);
        return json({ ok: true });
      }
      if (action === "sync" && body.preferences !== void 0) {
        if (!Number.isInteger(body.revision))
          return json({ error: "Atualize as prefer\xEAncias antes de salvar." }, 400);
        const merged = { ...JSON.parse(user.preferences), ...preferences(body.preferences) };
        const result = await db.prepare("UPDATE lux_accounts SET preferences = ?, revision = revision + 1 WHERE id = ? AND revision = ?").bind(JSON.stringify(merged), user.id, body.revision).run();
        const current = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(user.id).first();
        if (!result.meta.changes)
          return json({ error: "Suas prefer\xEAncias mudaram em outro dispositivo. Confira e salve novamente.", account: publicAccount(current) }, 409);
        return json({ account: publicAccount(current) });
      }
      return json({ account: publicAccount(user) });
    } catch (error) {
      return json({ error: error.status ? error.message : `Erro no servidor (${error.message || error}).` }, error.status || 500);
    }
  } catch (globalError) {
    return json({ error: `Falha interna no servidor (${globalError.message || globalError}).` }, 500);
  }
}
export {
  onRequest
};
