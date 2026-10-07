import { assetFor, latestRelease, fallback, repository } from "../../lib/releases.js";

async function localInstaller(request) {
  const source = await fetch(new URL("/releases/Lux%20MC%20Launcher.exe", request.url), {
    method: request.method === "HEAD" ? "HEAD" : "GET",
    headers: request.headers.has("Range") ? { Range: request.headers.get("Range"), ...(request.headers.has("If-Range") ? {"If-Range":request.headers.get("If-Range")} : {}) } : undefined
  });
  if (!source.ok) return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
  const headers = new Headers(source.headers);
  headers.set("Content-Type", "application/octet-stream");
  headers.set("Content-Disposition", 'attachment; filename="Lux MC Launcher.exe"');
  headers.set("Cache-Control", "no-store");
  headers.set("X-Content-Type-Options", "nosniff");
  const length = Number(source.headers.get("Content-Length"));
  const range = request.headers.get("Range");
  const ifRange = request.headers.get("If-Range");
  const rangeAllowed = !ifRange || (!ifRange.startsWith("W/") && (ifRange.startsWith('"') ? ifRange === source.headers.get("ETag") : Number.isFinite(Date.parse(ifRange)) && Date.parse(ifRange) === Date.parse(source.headers.get("Last-Modified") || "")));
  if(range && !rangeAllowed && source.status === 206) {
    await source.body?.cancel();
    const fullHeaders = new Headers(request.headers);
    fullHeaders.delete("Range");
    fullHeaders.delete("If-Range");
    return localInstaller(new Request(request,{headers:fullHeaders}));
  }
  if (Number.isSafeInteger(length) && length > 0) headers.set("Accept-Ranges", "bytes");
  if (range && rangeAllowed && source.status === 200 && Number.isSafeInteger(length) && length > 0) {
    const match = /^bytes=(\d*)-(\d*)$/.exec(range.trim());
    const start = match?.[1] ? Number(match[1]) : Math.max(0, length - Number(match?.[2]));
    const end = match?.[1] ? (match[2] ? Math.min(length - 1, Number(match[2])) : length - 1) : length - 1;
    if (!match || (!match[1] && !match[2]) || !Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || start >= length || end < start) {
      await source.body?.cancel();
      headers.set("Content-Range", `bytes */${length}`);
      headers.delete("Content-Length");
      return new Response(null, {status:416,headers});
    }
    headers.set("Content-Range", `bytes ${start}-${end}/${length}`);
    headers.set("Content-Length", String(end-start+1));
    if(request.method === "HEAD") return new Response(null,{status:206,headers});
    const reader = source.body.getReader();
    let position = 0;
    const body = new ReadableStream({
      async pull(controller) {
        try {
          while(position <= end) {
            const {done,value} = await reader.read();
            if(done) throw Error("Incomplete installer response");
            const selected = value.subarray(Math.max(0,start-position),Math.min(value.length,end-position+1));
            position += value.length;
            if(selected.length) controller.enqueue(selected);
            if(position > end) { controller.close(); await reader.cancel(); return; }
            if(selected.length) return;
          }
        } catch(error) { controller.error(error); await reader.cancel().catch(()=>{}); }
      },
      cancel(reason) { return reader.cancel(reason); }
    });
    return new Response(body,{status:206,headers});
  }
  return new Response(request.method === "HEAD" ? null : source.body, { status: source.status, headers });
}

export async function onRequest({ request, params, env }) {
  if (request && !["GET", "HEAD"].includes(request.method)) return new Response("Method not allowed", { status: 405 });
  const platform = String(params.platform || "").toLowerCase();
  try {
    const data = await latestRelease();
    const asset = assetFor(data.assets, platform);
    if ((platform === "windows" || platform === "exe") && env?.WINDOWS_LOCAL_VERSION === data.tag_name.replace(/^v/, "")) {
      return await localInstaller(request);
    }
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
    if ((platform === "windows" || platform === "exe") && env?.WINDOWS_LOCAL_VERSION) {
      try { return await localInstaller(request); }
      catch { return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } }); }
    }
    return new Response(null, {
      status: 302,
      headers: {
        Location: `https://github.com/${repository}/releases/latest`,
        "Cache-Control": "no-store"
      }
    });
  }
}
