export async function onRequestGet({ request }) {
  const video = new URL(request.url).searchParams.get('video') || '';
  if (!/^[A-Za-z0-9_-]{11}$/.test(video)) return new Response('Vídeo inválido', { status: 400 });
  const html = `<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="referrer" content="strict-origin-when-cross-origin"><title>Vídeo do modpack — Luxmc</title></head><body style="margin:0"><iframe title="Vídeo do modpack" src="https://www.youtube-nocookie.com/embed/${video}?autoplay=1&amp;rel=0&amp;playsinline=1" style="position:fixed;inset:0;width:100%;height:100%;border:0" allow="autoplay; encrypted-media; picture-in-picture; fullscreen" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe></body></html>`;
  return new Response(html, { headers: {
    'Content-Type': 'text/html; charset=utf-8',
    'Referrer-Policy': 'strict-origin-when-cross-origin',
    'Content-Security-Policy': "default-src 'none'; style-src 'unsafe-inline'; frame-src https://www.youtube-nocookie.com; frame-ancestors 'self' http://tauri.localhost https://tauri.localhost tauri: http://localhost:1420 http://127.0.0.1:1420",
    'X-Content-Type-Options': 'nosniff',
    'Cache-Control': 'public, max-age=300'
  } });
}
