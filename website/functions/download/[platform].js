import { assetFor, latestRelease, fallback } from "../../lib/releases.js";

export async function onRequest({ request, params }) {
  if (request && !["GET", "HEAD"].includes(request.method)) return new Response("Method not allowed", { status: 405 });
  const platform = String(params.platform || "").toLowerCase();
  try {
    const data = await latestRelease();
    const asset = assetFor(data.assets, platform);
    return new Response(null, { status: 302, headers: { Location: asset?.browser_download_url || fallback, "Cache-Control": "public, max-age=300", "X-Content-Type-Options": "nosniff" } });
  } catch {
    return new Response(null, { status: 302, headers: { Location: fallback, "Cache-Control": "no-store" } });
  }
}
