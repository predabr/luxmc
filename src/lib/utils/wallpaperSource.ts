import { convertFileSrc } from "@tauri-apps/api/core";
import { prepareWallpaperPoster } from "$lib/api/wallpaper";

export function wallpaperLocalPath(value: string): string | null {
    if (value.startsWith("/") || /^[A-Za-z]:[\\/]/.test(value)) return value;
    try {
        const url = new URL(value);
        if (url.hostname === "127.0.0.1" && url.port === "49152" && url.pathname === "/media") return url.searchParams.get("path");
        if (url.protocol === "asset:" || url.hostname === "asset.localhost") return decodeURIComponent(url.pathname);
    } catch {}
    return null;
}

export function resolveWallpaperImageUrl(value: string): string {
	const path = wallpaperLocalPath(value);
	if (!path) return value;
	const stripped = path.startsWith("/") ? path.slice(1) : path;
	return convertFileSrc(stripped);
}

export function resolveWallpaperVideoUrl(value: string): string {
	const path = wallpaperLocalPath(value);
	if (!path) return value;
	const stripped = path.startsWith("/") ? path.slice(1) : path;
	return convertFileSrc(stripped);
}

export function resolveWallpaperStreamUrl(value: string, port: number): string {
    const path = wallpaperLocalPath(value);
    return path ? `http://127.0.0.1:${port}/media?path=${encodeURIComponent(path)}` : value;
}

export const wallpaperPosters = new Map<string, string>();
const pendingPosters = new Map<string, Promise<string>>();

export function loadWallpaperPoster(value: string): Promise<string> {
    const path = wallpaperLocalPath(value);
    if (!path) return Promise.resolve("");
    const cached = wallpaperPosters.get(path);
    if (cached) return Promise.resolve(cached);
    const pending = pendingPosters.get(path);
    if (pending) return pending;
    const request = prepareWallpaperPoster(path).then(poster => {
        if (!poster.startsWith("data:image/png;base64,")) throw new Error("Prévia inválida");
        if (wallpaperPosters.size >= 12) wallpaperPosters.delete(wallpaperPosters.keys().next().value!);
        wallpaperPosters.set(path, poster);
        return poster;
    }).finally(() => pendingPosters.delete(path));
    pendingPosters.set(path, request);
    return request;
}
