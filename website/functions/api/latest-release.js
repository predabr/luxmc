import { latestRelease } from "../../lib/releases.js";

export async function onRequest({ request }) {
  const headers = { "Content-Type": "application/json", "X-Content-Type-Options": "nosniff" };
  if (request && !["GET", "HEAD"].includes(request.method)) return new Response(JSON.stringify({ error: "Method not allowed" }), { status: 405, headers });
  try {
    return new Response(JSON.stringify(await latestRelease()), { headers: { ...headers, "Cache-Control": "public, max-age=300, s-maxage=300, stale-while-revalidate=600" } });
  } catch {
    return new Response(JSON.stringify({ error: "Release metadata temporarily unavailable" }), { status: 502, headers: { ...headers, "Cache-Control": "no-store" } });
  }
}
