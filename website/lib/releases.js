export const repository = "predabr/luxmc";
export const fallback = `https://github.com/${repository}/releases`;

export function assetFor(assets, platform) {
  const extensions = { linux: ".appimage", appimage: ".appimage", windows: ".exe", exe: ".exe", deb: ".deb", debian: ".deb", ubuntu: ".deb", tar: ".tar.gz", "tar.gz": ".tar.gz", archive: ".tar.gz", mac: ".dmg", macos: ".dmg", dmg: ".dmg", jar: ".jar", universal: ".jar" };
  const extension = extensions[platform];
  if (!extension || !Array.isArray(assets)) return null;
  return assets.filter(asset => typeof asset.name === "string" && asset.name.toLowerCase().endsWith(extension)
    && typeof asset.browser_download_url === "string" && safeAssetUrl(asset.browser_download_url))
    .sort((a, b) => Number(/(?:x86_64|x64|amd64)/i.test(b.name)) - Number(/(?:x86_64|x64|amd64)/i.test(a.name)))[0] || null;
}

export function safeAssetUrl(value) {
  try {
    const url = new URL(value);
    return url.protocol === "https:" && url.hostname === "github.com" && !url.username && !url.password && !url.port
      && (url.pathname.startsWith(`/${repository}/releases/download/`) || url.pathname.startsWith(`/${repository}/releases/latest/download/`));
  } catch { return false; }
}

export async function latestRelease() {
  const response = await fetch(`https://api.github.com/repos/${repository}/releases/latest`, {
    headers: { "User-Agent": "Luxmc-Cloudflare-Pages/1.8.0", Accept: "application/vnd.github+json" },
    signal: AbortSignal.timeout(10000),
    cf: { cacheTtl: 300, cacheEverything: true }
  });
  if (!response.ok) throw new Error("GitHub release unavailable");
  const data = await response.json();
  if (typeof data.tag_name !== "string" || !Array.isArray(data.assets)) throw new Error("Invalid release metadata");
  return data;
}
