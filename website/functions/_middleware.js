async function onRequest(context) {
  const response = await context.next();
  if (!/^\/(?:api|download)\//.test(new URL(context.request.url).pathname)) return response;
  const secured = new Response(response.body, response);
  secured.headers.set("X-Content-Type-Options", "nosniff");
  secured.headers.set("Referrer-Policy", "strict-origin-when-cross-origin");
  secured.headers.set("Strict-Transport-Security", "max-age=31536000");
  secured.headers.set("X-Frame-Options", "DENY");
  secured.headers.set("Content-Security-Policy", "default-src 'none'; frame-ancestors 'none'");
  return secured;
}
export {
  onRequest
};
