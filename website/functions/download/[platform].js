import { assetFor, latestRelease, fallback, repository } from "../../lib/releases.js";

export async function onRequest({ request, params }) {
  if (request && !["GET", "HEAD"].includes(request.method)) return new Response("Method not allowed", { status: 405 });
  const platform = String(params.platform || "").toLowerCase();
  try {
    const data = await latestRelease();
    const asset = assetFor(data.assets, platform);
    if (asset && (platform === "windows" || platform === "exe")) {
      const range = request?.headers.get("Range");
      const source = await fetch(asset.browser_download_url, { method: request?.method === "HEAD" ? "HEAD" : "GET", headers: range ? { Range: range } : undefined });
      if (!source.ok) return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
      const headers = new Headers({
        "Content-Type": "application/octet-stream",
        "Content-Disposition": 'attachment; filename="Lux MC Launcher.exe"',
        "Cache-Control": "public, max-age=60",
        "X-Content-Type-Options": "nosniff"
      });
      for (const name of ["Content-Length", "Content-Range", "Accept-Ranges"]) {
        const value = source.headers.get(name);
        if (value) headers.set(name, value);
      }
      return new Response(request?.method === "HEAD" ? null : source.body, { status: source.status, headers });
    }
    const targetUrl = asset?.browser_download_url || `https://github.com/${repository}/releases/latest`;
    return new Response(null, {
      status: 302,
      headers: {
        Location: targetUrl,
        "Cache-Control": "public, max-age=60",
        "X-Content-Type-Options": "nosniff"
      }
    });
  } catch {
    return new Response(null, {
      status: 302,
      headers: {
        Location: `https://github.com/${repository}/releases/latest`,
        "Cache-Control": "no-store"
      }
    });
  }
}
