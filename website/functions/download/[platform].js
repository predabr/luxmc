export async function onRequest(context) {
  const { platform } = context.params;
  const GITHUB_REPO = "predabr/luxmc";
  const fallback = `https://github.com/${GITHUB_REPO}/releases/latest`;

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
      return Response.redirect(fallback, 302);
    }

    const data = await res.json();
    const assets = data.assets || [];
    const p = (platform || "").toLowerCase();

    let targetAsset = null;
    if (p === "linux" || p === "appimage") {
      targetAsset = assets.find(a => a.name.endsWith(".AppImage"));
    } else if (p === "windows" || p === "exe") {
      targetAsset = assets.find(a => a.name.endsWith(".exe"));
    } else if (p === "deb" || p === "debian" || p === "ubuntu") {
      targetAsset = assets.find(a => a.name.endsWith(".deb"));
    } else if (p === "tar" || p === "tar.gz" || p === "archive") {
      targetAsset = assets.find(a => a.name.endsWith(".tar.gz"));
    } else if (p === "mac" || p === "macos" || p === "dmg") {
      targetAsset = assets.find(a => a.name.endsWith(".dmg"));
    } else if (p === "jar" || p === "universal") {
      targetAsset = assets.find(a => a.name.endsWith(".jar"));
    }

    if (targetAsset && targetAsset.browser_download_url) {
      return Response.redirect(targetAsset.browser_download_url, 302);
    }

    return Response.redirect(fallback, 302);
  } catch (e) {
    return Response.redirect(fallback, 302);
  }
}
