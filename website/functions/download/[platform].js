import { assetFor, latestRelease, fallback, repository } from "../../lib/releases.js";

export async function onRequest({ request, params }) {
  if (request && !["GET", "HEAD"].includes(request.method)) return new Response("Method not allowed", { status: 405 });
  const platform = String(params.platform || "").toLowerCase();
  try {
    const data = await latestRelease();
    const asset = assetFor(data.assets, platform);
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
