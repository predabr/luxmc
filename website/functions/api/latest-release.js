export async function onRequest(context) {
  const GITHUB_REPO = "predabr/luxmc";
  try {
    const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`, {
      headers: {
        "User-Agent": "Luxmc-Cloudflare-Pages/1.0",
        "Accept": "application/vnd.github.v3+json"
      },
      cf: {
        cacheTtl: 300,
        cacheEverything: true
      }
    });

    if (!res.ok) {
      return new Response(JSON.stringify({ error: "Failed to fetch upstream release", status: res.status }), {
        status: res.status,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
          "Cache-Control": "public, max-age=60"
        }
      });
    }

    const data = await res.json();
    return new Response(JSON.stringify(data), {
      headers: {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*",
        "Cache-Control": "public, max-age=300, s-maxage=300, stale-while-revalidate=600"
      }
    });
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), {
      status: 500,
      headers: {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*"
      }
    });
  }
}
