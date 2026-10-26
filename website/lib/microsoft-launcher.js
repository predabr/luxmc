import { digest, randomToken, sessionToken, validNickname } from "./accounts.js";
import { commerceIdentity } from "./membership.js";
import { linkMicrosoftIdentity, microsoftSubjectAccount, verifyMicrosoftToken } from "./microsoft-identity.js";
import { microsoftJson } from "./microsoft.js";
async function protectedProfile(db, id, now) {
  const row = await db.prepare("SELECT (SELECT COUNT(*) FROM social_relationships WHERE sender = ? OR recipient = ?) + (SELECT COUNT(*) FROM lux_memberships WHERE social_id = ? AND expires_at > ?) + (SELECT COUNT(*) FROM lux_accounts WHERE social_id = ?) + (SELECT COUNT(*) FROM social_public_profiles WHERE user_id = ? AND (description != '' OR banner != '' OR portrait != '' OR packs != '[]' OR display_name != '')) AS total").bind(id, id, id, now, id, id).first();
  return Number(row?.total || 0) > 0;
}
async function startMicrosoftLauncher(request, db, body, configuration, now) {
  if (typeof body.verifier !== "string" || !/^[A-Za-z0-9_-]{43,128}$/.test(body.verifier) || !/^[a-f0-9]{32}$/i.test(body.uuid || "") || !validNickname(body.username)) throw new Error("Invalid launcher identity");
  const identity = await verifyMicrosoftToken(body.idToken, await digest(`luxmc-microsoft-launcher:${body.verifier}`), configuration);
  const me = await commerceIdentity(request, db);
  if (!me) throw new Error("Authenticated launcher profile required");
  const mapped = await microsoftSubjectAccount(db, identity.subject);
  if (mapped && mapped.social_id !== me.id) {
    if (await protectedProfile(db, me.id, now)) throw Object.assign(new Error("Existing device profile needs an explicit merge"), { reason: "conflict" });
  }
  const ticket = randomToken(), serverId = randomToken().slice(0, 40);
  await db.prepare("INSERT INTO lux_microsoft_launcher_pending(token_hash, subject_hash, social_id, minecraft_uuid, username, server_id, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)").bind(await digest(ticket), identity.subject, me.id, body.uuid.toLowerCase(), body.username, serverId, now + 120).run();
  await db.prepare("DELETE FROM lux_microsoft_launcher_pending WHERE expires_at < ?").bind(now).run();
  return { ticket, serverId };
}
async function completeMicrosoftLauncher(request, db, body, now) {
  if (!/^[a-f0-9]{64}$/.test(body.ticket || "")) throw new Error("Invalid launcher ticket");
  const me = await commerceIdentity(request, db);
  if (!me) throw new Error("Authenticated launcher profile required");
  const pending = await db.prepare("SELECT * FROM lux_microsoft_launcher_pending WHERE token_hash = ? AND social_id = ? AND expires_at > ?").bind(await digest(body.ticket), me.id, now).first();
  if (!pending) throw new Error("Expired launcher ticket");
  const url = new URL("https://sessionserver.mojang.com/session/minecraft/hasJoined");
  url.search = new URLSearchParams({ username: pending.username, serverId: pending.server_id }).toString();
  const profile = await microsoftJson(url.href, {});
  if (String(profile.id).toLowerCase() !== pending.minecraft_uuid || String(profile.name).toLowerCase() !== pending.username.toLowerCase()) throw new Error("Minecraft ownership not confirmed");
  const bound = await db.prepare("SELECT account_id FROM lux_microsoft_accounts WHERE minecraft_uuid = ?").bind(pending.minecraft_uuid).first();
  const mapped = await microsoftSubjectAccount(db, pending.subject_hash);
  if (mapped && mapped.social_id !== me.id && await protectedProfile(db, me.id, now)) throw Object.assign(new Error("Existing device profile needs an explicit merge"), { reason: "conflict" });
  const existing = await db.prepare("SELECT id FROM lux_accounts WHERE social_id = ?").bind(me.id).first();
  if (bound && bound.account_id !== (mapped?.id || existing?.id)) throw Object.assign(new Error("Minecraft profile already linked"), { reason: "conflict" });
  const previousMinecraft = mapped ? await db.prepare("SELECT minecraft_uuid FROM lux_microsoft_accounts WHERE account_id = ?").bind(mapped.id).first() : null;
  if (previousMinecraft && previousMinecraft.minecraft_uuid !== pending.minecraft_uuid) throw Object.assign(new Error("Different Minecraft profile"), { reason: "conflict" });
  const consumed = await db.prepare("DELETE FROM lux_microsoft_launcher_pending WHERE token_hash = ? AND expires_at > ? RETURNING token_hash").bind(pending.token_hash, now).first();
  if (!consumed) throw new Error("Consumed launcher ticket");
  const account = mapped || await linkMicrosoftIdentity(db, { token_hash: "", subject_hash: pending.subject_hash }, me.id, now);
  if (!account) throw new Error("Profile unavailable");
  await db.prepare("INSERT INTO lux_microsoft_accounts(minecraft_uuid, account_id, created_at) VALUES (?, ?, ?) ON CONFLICT(minecraft_uuid) DO NOTHING").bind(pending.minecraft_uuid, account.id, now).run();
  const token = sessionToken(request);
  if (!token) throw new Error("Missing device identity");
  await db.prepare("INSERT INTO lux_microsoft_devices(token_hash, account_id, created_at) VALUES (?, ?, ?) ON CONFLICT(token_hash) DO NOTHING").bind(await digest(token), account.id, now).run();
  if (account.social_id !== me.id) await db.prepare("UPDATE social_users SET removed_at = ?, last_seen = 0, activity = 'offline' WHERE id = ?").bind(now, me.id).run();
  return { ok: true };
}
export {
  completeMicrosoftLauncher,
  startMicrosoftLauncher
};
