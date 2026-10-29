import { authenticate, digest, sessionToken } from "./accounts.js";
import { profileBenefits } from "./profile-style.js";
const COMMERCE_COOKIE = "__Host-luxmc-commerce";
const catalog = {
  plus: { name: "Luxmc Plus", amount: 2e3, days: 30, collections: 12, presets: 6, profile: profileBenefits.plus },
  pro: { name: "Luxmc Pro", amount: 3e3, days: 30, collections: 24, presets: 20, profile: profileBenefits.pro }
};
const nowSeconds = () => Math.floor(Date.now() / 1e3);
const fail = (message, status = 400) => {
  throw Object.assign(new Error(message), { status });
};
async function commerceIdentity(request, db) {
  if (!request.headers.get("Authorization")) {
    const scoped = request.headers.get("Cookie")?.split(";").map((value) => value.trim()).find((value) => value.startsWith(`${COMMERCE_COOKIE}=`))?.slice(COMMERCE_COOKIE.length + 1);
    if (scoped && /^[a-f0-9]{64}$/.test(scoped)) {
      const session = await db.prepare("SELECT u.id, u.username FROM lux_commerce_sessions s JOIN social_users u ON u.id = s.social_id WHERE s.token_hash = ? AND s.expires_at > ? AND u.removed_at = 0").bind(await digest(scoped), nowSeconds()).first();
      if (session) return session;
    }
  }
  const registered = await authenticate(request, db);
  if (registered) return db.prepare("SELECT id, username FROM social_users WHERE id = ? AND removed_at = 0").bind(registered.social_id).first();
  const token = sessionToken(request);
  if (token && request.headers.has("Authorization")) return db.prepare("SELECT id, username FROM social_users WHERE token_hash = ? AND removed_at = 0").bind(await digest(token)).first();
  return null;
}
async function membership(db, id) {
  const row = await db.prepare("SELECT plan, expires_at, revision FROM lux_memberships WHERE social_id = ?").bind(id).first();
  const plan = row && row.expires_at > nowSeconds() ? row.plan : "free";
  return { plan, expiresAt: row ? row.expires_at : null, revision: row?.revision || 0, collections: plan === "free" ? 6 : catalog[plan].collections, presets: plan === "free" ? 0 : catalog[plan].presets };
}
function appearancePreferences(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return fail("Prefer\xEAncias inv\xE1lidas.");
  const record = value;
  const allowed = { density: ["compact", "comfortable", "spacious"], motionStyle: ["subtle", "expressive"], motionSpeed: ["fast", "balanced"], wallpaperFps: [15, 24, 30, 60], pauseSkinWhileGaming: [true, false], preloadContentIcons: [true, false], respectReducedMotion: [true, false], gifQuality: ["balanced", "high"] };
  const clean = {};
  for (const [key, values] of Object.entries(allowed)) {
    if (Object.hasOwn(record, key)) {
      if (!values.includes(record[key])) return fail("Prefer\xEAncia inv\xE1lida.");
      clean[key] = record[key];
    }
  }
  if (Object.hasOwn(record, "interfaceOpacity")) {
    if (typeof record.interfaceOpacity !== "number" || !Number.isFinite(record.interfaceOpacity) || record.interfaceOpacity < 0.05 || record.interfaceOpacity > 1) return fail("Opacidade inv\xE1lida.");
    clean.interfaceOpacity = record.interfaceOpacity;
  }
  return clean;
}
export {
  COMMERCE_COOKIE,
  appearancePreferences,
  catalog,
  commerceIdentity,
  fail,
  membership,
  nowSeconds
};
