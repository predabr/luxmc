import { createServer } from "node:http";
import { readFile, stat, readdir } from "node:fs/promises";
import { resolve, join, extname, sep } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { randomBytes } from "node:crypto";
import { onRequest as account } from "../website/functions/api/account/[action].js";
import { onRequest as social } from "../website/functions/api/social/[action].js";
import { onRequest as release } from "../website/functions/api/latest-release.js";
import { onRequest as download } from "../website/functions/download/[platform].js";

const root = resolve("website"),
  port = Number(process.env.LUXMC_SITE_PORT || 8790);
const sqlite = new DatabaseSync(":memory:");
sqlite.exec("PRAGMA foreign_keys=ON");
for (const migration of (await readdir(join(root, "migrations")))
  .filter((name) => name.endsWith(".sql"))
  .sort())
  sqlite.exec(await readFile(join(root, "migrations", migration), "utf8"));
const db = {
  prepare(sql) {
    const statement = sqlite.prepare(sql);
    const bound = (values = []) => ({
      bind: (...next) => bound(next),
      first: async () => statement.get(...values) || null,
      all: async () => ({ results: statement.all(...values) }),
      run: async () => ({ meta: statement.run(...values) }),
      execute: () => ({ meta: statement.run(...values) }),
    });
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
    const api = url.pathname.match(/^\/api\/(account|social)\/([a-z-]+)$/);
    if (api) {
      handler = api[1] === "account" ? account : social;
      params = { action: api[2] };
    }
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
      outgoing.writeHead(response.status, Object.fromEntries(response.headers));
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
