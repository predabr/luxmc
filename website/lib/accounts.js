export const SESSION_SECONDS = 30 * 86400;
export const COOKIE = "__Host-luxmc";
export const hex = bytes => Array.from(new Uint8Array(bytes), b => b.toString(16).padStart(2, "0")).join("");
export const randomToken = () => hex(crypto.getRandomValues(new Uint8Array(32)));
export const digest = async value => hex(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value)));
export const json = (value, status = 200, extra = {}) => new Response(JSON.stringify(value), { status, headers: { "Content-Type": "application/json", "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff", ...extra } });
export const publicAccount = row => ({ id: row.id, username: row.username, socialId: row.social_id, preferences: JSON.parse(row.preferences), revision: row.revision });
export const validNickname = value => typeof value === "string" && /^[A-Za-z0-9_]{3,16}$/.test(value);
export const validPassword = value => typeof value === "string" && value.length >= 8 && value.length <= 128;
const ALLOWED_ORIGIN_PATTERNS = [
  "tauri://localhost",
  "http://localhost:1420",
  "http://127.0.0.1:1420"
];

export const sameOrigin = request => {
  const origin = request.headers.get("Origin");
  if (!origin) return true;
  if (ALLOWED_ORIGIN_PATTERNS.includes(origin)) return true;
  try {
    const reqOrigin = new URL(request.url).origin;
    if (origin === reqOrigin) return true;
    const originHost = new URL(origin).hostname;
    if (originHost === "luxmc.top" || originHost.endsWith(".luxmc.top") || originHost.endsWith(".pages.dev") || originHost === "localhost" || originHost === "127.0.0.1") {
      return true;
    }
  } catch {}
  return false;
};

export const cookie = (token, age = SESSION_SECONDS) => `${COOKIE}=${token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=${age}`;

export async function ensureTables(db) {
  try {
    await db.batch([
      db.prepare(`
        CREATE TABLE IF NOT EXISTS lux_accounts (
          id TEXT PRIMARY KEY,
          username TEXT UNIQUE COLLATE NOCASE,
          password_hash TEXT NOT NULL,
          password_salt TEXT NOT NULL,
          recovery_hash TEXT,
          social_id TEXT,
          preferences TEXT NOT NULL DEFAULT '{}',
          revision INTEGER NOT NULL DEFAULT 0,
          created_at INTEGER NOT NULL
        )
      `),
      db.prepare(`
        CREATE TABLE IF NOT EXISTS lux_sessions (
          token_hash TEXT PRIMARY KEY,
          account_id TEXT NOT NULL,
          created_at INTEGER NOT NULL,
          expires_at INTEGER NOT NULL,
          client TEXT NOT NULL DEFAULT 'web'
        )
      `),
      db.prepare(`
        CREATE TABLE IF NOT EXISTS social_users (
          id TEXT PRIMARY KEY,
          token_hash TEXT UNIQUE,
          username TEXT UNIQUE COLLATE NOCASE,
          avatar TEXT,
          bio TEXT,
          created_at INTEGER DEFAULT (unixepoch())
        )
      `),
      db.prepare(`
        CREATE TABLE IF NOT EXISTS social_limits (
          bucket TEXT PRIMARY KEY,
          count INTEGER NOT NULL,
          expires_at INTEGER NOT NULL
        )
      `)
    ]);
  } catch (err) {
    console.warn("ensureTables notice:", err);
  }
}

export async function limited(db, key, seconds, maximum) {
  const now = Math.floor(Date.now() / 1000);
  const row = await db.prepare("INSERT INTO social_limits(bucket, count, expires_at) VALUES (?, 1, ?) ON CONFLICT(bucket) DO UPDATE SET count = count + 1 RETURNING count")
    .bind(`${key}:${Math.floor(now / seconds)}`, now + seconds).first();
  return row.count > maximum;
}

export async function readJson(request, limit = 8192) {
  if (!request.headers.get("Content-Type")?.startsWith("application/json")) throw Object.assign(new Error("Envie JSON."), { status: 415 });
  if (Number(request.headers.get("Content-Length")) > limit) throw Object.assign(new Error("Pedido muito grande."), { status: 413 });
  const reader = request.body?.getReader();
  if (!reader) throw Object.assign(new Error("Pedido vazio."), { status: 400 });
  const chunks = [];
  let size = 0;
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    size += value.byteLength;
    if (size > limit) { await reader.cancel(); throw Object.assign(new Error("Pedido muito grande."), { status: 413 }); }
    chunks.push(value);
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
  try {
    const value = JSON.parse(new TextDecoder().decode(bytes));
    if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error();
    return value;
  } catch { throw Object.assign(new Error("JSON inválido."), { status: 400 }); }
}

export async function passwordHash(password, salt, pepper) {
  const material = await crypto.subtle.importKey("raw", new TextEncoder().encode(password), "PBKDF2", false, ["deriveBits"]);
  const derived = await crypto.subtle.deriveBits({ name: "PBKDF2", salt: new TextEncoder().encode(salt), iterations: 100000, hash: "SHA-256" }, material, 256);
  const key = await crypto.subtle.importKey("raw", new TextEncoder().encode(pepper), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return hex(await crypto.subtle.sign("HMAC", key, derived));
}

export function equalHash(a, b) {
  let diff = a.length ^ b.length;
  for (let i = 0; i < Math.max(a.length, b.length); i++) diff |= (a.charCodeAt(i) || 0) ^ (b.charCodeAt(i) || 0);
  return diff === 0;
}

export function sessionToken(request) {
  const authorization = request.headers.get("Authorization");
  if (authorization) return authorization.match(/^Bearer ([a-f0-9]{64})$/)?.[1] || null;
  return request.headers.get("Cookie")?.split(";").map(value => value.trim()).find(value => value.startsWith(`${COOKIE}=`))?.slice(COOKIE.length + 1) || null;
}

export async function authenticate(request, db) {
  const token = sessionToken(request);
  if (!token || !/^[a-f0-9]{64}$/.test(token)) return null;
  return db.prepare("SELECT a.*, s.token_hash AS session_hash FROM lux_sessions s JOIN lux_accounts a ON a.id = s.account_id WHERE s.token_hash = ? AND s.expires_at > ?")
    .bind(await digest(token), Math.floor(Date.now() / 1000)).first();
}

export function preferences(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw Object.assign(new Error("Preferências inválidas."), { status: 400 });
  const options = { theme: ["default-dark", "default-light"], accentTheme: ["gold", "cyan", "emerald", "rose", "violet", "orange", "blue"], language: ["pt-BR", "en"], animations: [true, false] };
  const result = {};
  for (const [key, allowed] of Object.entries(options)) {
    if (Object.hasOwn(value, key)) {
      if (!allowed.includes(value[key])) throw Object.assign(new Error("Preferência inválida."), { status: 400 });
      result[key] = value[key];
    }
  }
  return result;
}
