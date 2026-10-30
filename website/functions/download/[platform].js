import { assetFor, latestRelease, repository } from "../../lib/releases.js";
function localIsCurrent(local, remote) {
  const remoteVersion = String(remote).replace(/^v/, "");
  if (/-revision\.|-build\./.test(String(local)) || /-revision\.|-build\./.test(remoteVersion)) return String(local).replace(/^v/, "") === remoteVersion;
  if (!/^\d+\.\d+\.\d+$/.test(local || ""))
    return false;
  const parts = String(remote).replace(/^v/, "").split(".").map(Number);
  if (parts.length !== 3 || parts.some((value) => !Number.isSafeInteger(value)))
    return false;
  const current = local.split(".").map(Number);
  for (let index = 0; index < 3; index++) {
    if (current[index] !== parts[index])
      return current[index] > parts[index];
  }
  return true;
}
function rangeValidatorMatches(request, source) {
  const validator = request.headers.get("If-Range");
  return !validator || !validator.startsWith("W/") && (validator.startsWith('"') ? validator === source.headers.get("ETag") : Number.isFinite(Date.parse(validator)) && Date.parse(validator) === Date.parse(source.headers.get("Last-Modified") || ""));
}
async function localInstaller(request, assets, expectedLength) {
  const controller = new AbortController();
  let timer;
  const init = {
    method: request.method === "HEAD" ? "HEAD" : "GET",
    signal: controller.signal,
    headers: request.headers.has("Range") ? { Range: request.headers.get("Range"), ...request.headers.has("If-Range") ? { "If-Range": request.headers.get("If-Range") } : {} } : void 0
  };
  const assetRequest = new Request(new URL("/releases/Lux%20MC%20Launcher.exe", request.url), init);
  let source;
  try {
    source = await Promise.race([
      assets ? assets.fetch(assetRequest) : fetch(assetRequest.url, init),
      new Promise((_, reject) => {
        timer = setTimeout(() => {
          controller.abort();
          reject(new Error("Installer response timed out"));
        }, 8e3);
      })
    ]);
  } catch {
    return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
  } finally {
    clearTimeout(timer);
  }
  if (!source.ok || source.headers.get("Content-Type")?.includes("text/html"))
    return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
  const headers = new Headers(source.headers);
  headers.set("Content-Type", "application/octet-stream");
  headers.set("Content-Disposition", 'attachment; filename="Lux MC Launcher.exe"');
  headers.set("Cache-Control", "no-store");
  headers.set("X-Content-Type-Options", "nosniff");
  let length = Number(source.headers.get("Content-Length"));
  if ((!Number.isSafeInteger(length) || length <= 0) && source.status === 200 && typeof expectedLength === "number" && Number.isSafeInteger(expectedLength) && expectedLength > 0) {
    length = expectedLength;
    headers.set("Content-Length", String(length));
  }
  const range = request.headers.get("Range");
  const rangeAllowed = rangeValidatorMatches(request, source);
  if (range && !rangeAllowed && source.status === 206) {
    await source.body?.cancel();
    const fullHeaders = new Headers(request.headers);
    fullHeaders.delete("Range");
    fullHeaders.delete("If-Range");
    return localInstaller(new Request(request, { headers: fullHeaders }), assets, expectedLength);
  }
  if (Number.isSafeInteger(length) && length > 0)
    headers.set("Accept-Ranges", "bytes");
  if (range && rangeAllowed && source.status === 200 && Number.isSafeInteger(length) && length > 0) {
    const match = /^bytes=(\d*)-(\d*)$/.exec(range.trim());
    const start = match?.[1] ? Number(match[1]) : Math.max(0, length - Number(match?.[2]));
    const end = match?.[1] ? match[2] ? Math.min(length - 1, Number(match[2])) : length - 1 : length - 1;
    if (!match || !match[1] && !match[2] || !Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || start >= length || end < start) {
      await source.body?.cancel();
      headers.set("Content-Range", `bytes */${length}`);
      headers.delete("Content-Length");
      return new Response(null, { status: 416, headers });
    }
    headers.set("Content-Range", `bytes ${start}-${end}/${length}`);
    headers.set("Content-Length", String(end - start + 1));
    if (request.method === "HEAD")
      return new Response(null, { status: 206, headers });
    if (!source.body) return new Response("Installer stream unavailable", { status: 502 });
    const reader = source.body.getReader();
    let position = 0;
    const body = new ReadableStream({
      async pull(controller2) {
        try {
          while (position <= end) {
            const { done, value } = await reader.read();
            if (done)
              throw Error("Incomplete installer response");
            const selected = value.subarray(Math.max(0, start - position), Math.min(value.length, end - position + 1));
            position += value.length;
            if (selected.length)
              controller2.enqueue(selected);
            if (position > end) {
              controller2.close();
              await reader.cancel();
              return;
            }
            if (selected.length)
              return;
          }
        } catch (error) {
          controller2.error(error);
          await reader.cancel().catch(() => {
          });
        }
      },
      cancel(reason) {
        return reader.cancel(reason);
      }
    });
    return new Response(body, { status: 206, headers });
  }
  return new Response(request.method === "HEAD" ? null : source.body, { status: source.status, headers });
}
async function onRequest({ request, params, env }) {
  if (request && !["GET", "HEAD"].includes(request.method))
    return new Response("Method not allowed", { status: 405 });
  const platform = String(params.platform || "").toLowerCase();
  const localRelease = env?.WINDOWS_LOCAL_BUILD || env?.WINDOWS_LOCAL_VERSION;
  try {
    const data = await latestRelease();
    const asset = assetFor(data.assets, platform);
    if ((platform === "windows" || platform === "exe") && localIsCurrent(localRelease, data.tag_name)) {
      const local = await localInstaller(request, env?.ASSETS, asset?.size);
      if (local.status !== 502) return local;
    }
    if (asset && (platform === "windows" || platform === "exe")) {
      const range = request?.headers.get("Range");
      const ifRange = request?.headers.get("If-Range");
      const method = request?.method === "HEAD" ? "HEAD" : "GET";
      let source = await fetch(asset.browser_download_url, { method, headers: range ? { Range: range, ...ifRange ? { "If-Range": ifRange } : {} } : void 0 });
      if (range && source.status === 206 && !rangeValidatorMatches(request, source)) {
        await source.body?.cancel();
        source = await fetch(asset.browser_download_url, { method });
      }
      if (!source.ok)
        return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
      const headers = new Headers({
        "Content-Type": "application/octet-stream",
        "Content-Disposition": 'attachment; filename="Lux MC Launcher.exe"',
        "Cache-Control": "public, max-age=60",
        "X-Content-Type-Options": "nosniff"
      });
      for (const name of ["Content-Length", "Content-Range", "Accept-Ranges", "ETag", "Last-Modified"]) {
        const value = source.headers.get(name);
        if (value)
          headers.set(name, value);
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
    if ((platform === "windows" || platform === "exe") && localRelease) {
      try {
        return await localInstaller(request, env?.ASSETS);
      } catch {
        return new Response("Installer temporarily unavailable", { status: 502, headers: { "Cache-Control": "no-store" } });
      }
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
export {
  onRequest
};
