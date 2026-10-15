import { createServer } from "node:http";
import { readFile, stat, readdir } from "node:fs/promises";
import { resolve, join, extname, sep } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { randomBytes } from "node:crypto";
import { onRequest as account } from "../website/functions/api/account/[action].js";
import { onRequest as commerce } from "../website/functions/api/commerce/[action].js";
import { onRequest as receipts } from "../website/functions/api/receipts/[id].js";
import { onRequest as purchaseReceipts } from "../website/functions/api/purchase-receipts/[id].js";
import { onRequest as social } from "../website/functions/api/social/[action].js";
import { onRequest as release } from "../website/functions/api/latest-release.js";
import { onRequest as download } from "../website/functions/download/[platform].js";
import { onRequest as session } from "../website/functions/api/session.js";
import { onRequest as microsoft } from "../website/functions/api/microsoft/[action].js";

const root = resolve("website"),
  port = Number(process.env.LUXMC_SITE_PORT || 8790);
const securityHeaders={};
for(const line of (await readFile(join(root,"_headers"),"utf8")).split(/\r?\n/).slice(1)) {
  if(!line.trim()) break;
  const separator=line.indexOf(':');
  if(separator>0) securityHeaders[line.slice(0,separator).trim()]=line.slice(separator+1).trim();
}
const sqlite = new DatabaseSync(":memory:");
sqlite.exec("PRAGMA foreign_keys=ON");
for (const migration of (await readdir(join(root, "migrations")))
  .filter((name) => name.endsWith(".sql"))
  .sort())
  sqlite.exec(await readFile(join(root, "migrations", migration), "utf8"));
const db = {
  prepare(sql) {
    const statement = sqlite.prepare(sql);
    const bound = (input = []) => { const values = input.map(value => value instanceof ArrayBuffer ? new Uint8Array(value) : value); return ({
      bind: (...next) => bound(next),
      first: async () => statement.get(...values) || null,
      all: async () => ({ results: statement.all(...values) }),
      run: async () => ({ meta: statement.run(...values) }),
      execute: () => ({ meta: statement.run(...values) }),
    }); };
    return bound();
  },
  async batch(statements) {
    sqlite.exec("BEGIN");
    try {
      const results = statements.map((statement) => statement.execute());
      sqlite.exec("COMMIT");
      return results;
    } catch (error) {
      sqlite.exec("ROLLBACK");
      throw error;
    }
  },
};
const env = { SOCIAL_DB: db, AUTH_PEPPER: randomBytes(40).toString("hex") };
const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".woff2": "font/woff2",
  ".png": "image/png",
  ".webp": "image/webp",
  ".svg": "image/svg+xml",
  ".mp4": "video/mp4",
  ".txt": "text/plain; charset=utf-8",
  ".xml": "application/xml",
};
const server = createServer(async (incoming, outgoing) => {
  try {
    const url = new URL(incoming.url, `http://127.0.0.1:${port}`);
    let handler,
      params = {};
    if (url.pathname === '/api/session') {handler=session;params={};}
    const api = url.pathname.match(/^\/api\/(account|social|commerce|microsoft)\/([a-z_-]+)$/);
    if (api) {
      handler = api[1] === "account" ? account : api[1] === "commerce" ? commerce : api[1] === 'microsoft' ? microsoft : social;
      params = { action: api[2] };
    }
    const receipt = url.pathname.match(/^\/api\/receipts\/([a-f0-9-]{36})$/);
    if (receipt) { handler = receipts; params = {id:receipt[1]}; }
    const purchaseReceipt = url.pathname.match(/^\/api\/purchase-receipts\/([a-f0-9-]{36})$/);
    if (purchaseReceipt) { handler = purchaseReceipts; params = {id:purchaseReceipt[1]}; }
    if (url.pathname === "/api/latest-release") handler = release;
    const platform = url.pathname.match(/^\/download\/([a-z.]+)$/);
    if (platform) {
      handler = download;
      params = { platform: platform[1] };
    }
    if (handler) {
      const chunks = [];
      for await (const chunk of incoming) chunks.push(chunk);
      const request = new Request(url, {
        method: incoming.method,
        headers: incoming.headers,
        body: ["GET", "HEAD"].includes(incoming.method)
          ? undefined
          : Buffer.concat(chunks),
      });
      const response = await handler({
        request,
        env,
        params,
        waitUntil: (promise) => void Promise.resolve(promise).catch(() => {}),
      });
      outgoing.writeHead(response.status, {...securityHeaders,...Object.fromEntries(response.headers), ...(response.headers.getSetCookie().length ? {'Set-Cookie':response.headers.getSetCookie()} : {})});
      outgoing.end(Buffer.from(await response.arrayBuffer()));
      return;
    }
    if (!["GET", "HEAD"].includes(incoming.method)) {
      outgoing.writeHead(405);
      outgoing.end();
      return;
    }
    const pathname = decodeURIComponent(url.pathname);
    let path = resolve(
      root,
      "." + (pathname === "/" ? "/index.html" : pathname),
    );
    if (!path.startsWith(root + sep)) {
      outgoing.writeHead(403);
      outgoing.end();
      return;
    }
    if (!extname(path)) path += ".html";
    let status = 200;
    try {
      if (!(await stat(path)).isFile()) throw new Error("Not a file");
    } catch {
      path = join(root, "404.html");
      status = 404;
    }
    const content = await readFile(path);
    outgoing.writeHead(status, {
      ...securityHeaders,
      "Content-Type": mime[extname(path)] || "application/octet-stream",
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
      "Referrer-Policy": "strict-origin-when-cross-origin",
      "Content-Length": content.length,
    });
    outgoing.end(incoming.method === "HEAD" ? undefined : content);
  } catch (error) {
    console.error(error);
    outgoing.writeHead(500, { "Content-Type": "application/json" });
    outgoing.end(JSON.stringify({ error: "Preview local indisponível." }));
  }
});
server.listen(port, "127.0.0.1", () =>
  console.log(
    `LuxMC website: http://127.0.0.1:${port}\nAccount data is temporary and local. Production services are not modified.`,
  ),
);
process.on("SIGINT", () =>
  server.close(() => {
    sqlite.close();
    process.exit(0);
  }),
);
