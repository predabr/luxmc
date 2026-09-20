export async function onRequest() {
  return new Response("google-site-verification: google13eed2225453caeb.html", {
    status: 200,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "no-cache"
    }
  });
}
