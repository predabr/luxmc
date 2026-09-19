import { authenticate, cookie, digest, equalHash, json, limited, passwordHash, preferences, publicAccount, randomToken, readJson, sameOrigin, SESSION_SECONDS, validNickname, validPassword } from "../../../lib/accounts.js";

export async function onRequest({ request, env, params, waitUntil }) {
  if (request.method !== "POST") return json({ error: "Método não permitido." }, 405);
  if (!sameOrigin(request)) return json({ error: "Origem não permitida." }, 403);
  if (!env.SOCIAL_DB || typeof env.AUTH_PEPPER !== "string" || env.AUTH_PEPPER.length < 32) return json({ error: "As contas ainda não foram ativadas neste portal." }, 503);
  const db = env.SOCIAL_DB;
  const action = params.action;
  if (!["register", "login", "me", "logout", "sync", "password", "recover"].includes(action)) return json({ error: "Ação desconhecida." }, 404);
  try {
    const body = await readJson(request);
    const ip = await digest(request.headers.get("CF-Connecting-IP") || "local");
    if (await limited(db, `account:${ip}`, 60, 120)) return json({ error: "Muitas requisições. Aguarde um minuto." }, 429);
    const now = Math.floor(Date.now() / 1000);
    if (Math.random() < 0.02) waitUntil(Promise.all([
      db.prepare("DELETE FROM lux_sessions WHERE expires_at < ?").bind(now).run(),
      db.prepare("DELETE FROM social_limits WHERE expires_at < ?").bind(now).run()
    ]));
    if (["register", "login", "recover"].includes(action)) {
      if (!validNickname(body.username)) return json({ error: "Use um nickname de 3 a 16 letras, números ou _." }, 400);
      if (!validPassword(body.password)) return json({ error: "A senha deve ter entre 8 e 128 caracteres." }, 400);
      const username = body.username.trim();
      if (await limited(db, `auth:${ip}`, 900, 30) || await limited(db, `auth-name:${await digest(username.toLowerCase())}`, 900, 20)) return json({ error: "Muitas tentativas. Aguarde 15 minutos." }, 429);
      let user = await db.prepare("SELECT * FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(username).first();
      let recoveryCode;
      if (action === "register") {
        if (await limited(db, `signup:${ip}`, 3600, 5)) return json({ error: "Limite de cadastros atingido. Tente mais tarde." }, 429);
        if (user) return json({ error: "Esse nickname já está cadastrado." }, 409);
        const id = crypto.randomUUID();
        const socialId = crypto.randomUUID();
        const salt = randomToken();
        recoveryCode = randomToken();
        const password = await passwordHash(body.password, salt, env.AUTH_PEPPER);
        try {
          await db.batch([
            db.prepare("INSERT INTO social_users(id, token_hash, username) VALUES (?, ?, ?)").bind(socialId, await digest(randomToken()), username),
            db.prepare("INSERT INTO lux_accounts(id, username, password_hash, password_salt, recovery_hash, social_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
              .bind(id, username, password, salt, await digest(recoveryCode), socialId, now)
          ]);
        } catch (error) {
          if (await db.prepare("SELECT id FROM lux_accounts WHERE username = ? COLLATE NOCASE").bind(username).first()) return json({ error: "Esse nickname já está cadastrado." }, 409);
          throw error;
        }
        user = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(id).first();
      } else if (action === "recover") {
        if (typeof body.recoveryCode !== "string" || !/^[a-f0-9]{64}$/.test(body.recoveryCode) || !user || !equalHash(await digest(body.recoveryCode), user.recovery_hash)) return json({ error: "Nickname ou código de recuperação inválido." }, 401);
        const salt = randomToken();
        recoveryCode = randomToken();
        const result = await db.batch([
          db.prepare("UPDATE lux_accounts SET password_hash = ?, password_salt = ?, recovery_hash = ? WHERE id = ? AND recovery_hash = ?")
            .bind(await passwordHash(body.password, salt, env.AUTH_PEPPER), salt, await digest(recoveryCode), user.id, user.recovery_hash),
          db.prepare("DELETE FROM lux_sessions WHERE account_id = ?").bind(user.id)
        ]);
        if (!result[0].meta.changes) return json({ error: "Código de recuperação já utilizado." }, 401);
        user = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(user.id).first();
      } else {
        const check = await passwordHash(body.password, user?.password_salt || "luxmc-dummy-salt-for-missing-user", env.AUTH_PEPPER);
        if (!user || !equalHash(check, user.password_hash)) return json({ error: "Nickname ou senha incorretos." }, 401);
      }
      const token = randomToken();
      const client = body.client === "launcher" ? "launcher" : "web";
      const session = await db.prepare("INSERT INTO lux_sessions(token_hash, account_id, created_at, expires_at, client) SELECT ?, id, ?, ?, ? FROM lux_accounts WHERE id = ? AND password_hash = ?")
        .bind(await digest(token), now, now + SESSION_SECONDS, client, user.id, user.password_hash).run();
      if (!session.meta.changes) return json({ error: "A senha mudou. Entre novamente." }, 401);
      await db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash NOT IN (SELECT token_hash FROM lux_sessions WHERE account_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 20)").bind(user.id, user.id).run();
      return json({ account: publicAccount(user), ...(client === "launcher" ? { token } : {}), ...(recoveryCode ? { recoveryCode } : {}) }, 200, client === "web" ? { "Set-Cookie": cookie(token) } : {});
    }
    const user = await authenticate(request, db);
    if (!user) return json({ error: "Sua sessão expirou. Entre novamente." }, 401);
    if (action === "logout") {
      await db.prepare("DELETE FROM lux_sessions WHERE token_hash = ?").bind(user.session_hash).run();
      return json({ ok: true }, 200, { "Set-Cookie": cookie("", 0) });
    }
    if (action === "password") {
      if (!validPassword(body.password) || typeof body.currentPassword !== "string" || body.currentPassword.length > 128) return json({ error: "Confira a senha atual e use ao menos 8 caracteres na nova senha." }, 400);
      if (await limited(db, `password:${user.id}`, 900, 10)) return json({ error: "Muitas tentativas. Aguarde 15 minutos." }, 429);
      if (!equalHash(await passwordHash(body.currentPassword, user.password_salt, env.AUTH_PEPPER), user.password_hash)) return json({ error: "Senha atual incorreta." }, 401);
      const salt = randomToken();
      await db.batch([
        db.prepare("UPDATE lux_accounts SET password_hash = ?, password_salt = ? WHERE id = ?").bind(await passwordHash(body.password, salt, env.AUTH_PEPPER), salt, user.id),
        db.prepare("DELETE FROM lux_sessions WHERE account_id = ? AND token_hash != ?").bind(user.id, user.session_hash)
      ]);
      return json({ ok: true });
    }
    if (action === "sync" && body.preferences !== undefined) {
      if (!Number.isInteger(body.revision)) return json({ error: "Atualize as preferências antes de salvar." }, 400);
      const merged = { ...JSON.parse(user.preferences), ...preferences(body.preferences) };
      const result = await db.prepare("UPDATE lux_accounts SET preferences = ?, revision = revision + 1 WHERE id = ? AND revision = ?")
        .bind(JSON.stringify(merged), user.id, body.revision).run();
      const current = await db.prepare("SELECT * FROM lux_accounts WHERE id = ?").bind(user.id).first();
      if (!result.meta.changes) return json({ error: "Suas preferências mudaram em outro dispositivo. Confira e salve novamente.", account: publicAccount(current) }, 409);
      return json({ account: publicAccount(current) });
    }
    return json({ account: publicAccount(user) });
  } catch (error) {
    return json({ error: error.status ? error.message : "Não foi possível conectar à conta. Tente novamente." }, error.status || 503);
  }
}
