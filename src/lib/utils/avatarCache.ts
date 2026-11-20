const CACHE_PREFIX = "luxmc.avatar.";

function cacheKey(url: string): string {
	let hash = 0;
	for (let index = 0; index < url.length; index += 1) {
		hash = (hash * 31 + url.charCodeAt(index)) | 0;
	}
	return CACHE_PREFIX + Math.abs(hash).toString(36);
}

export function getCachedAvatar(url?: string | null): string | null {
	if (!url || typeof window === "undefined") return null;
	if (url.startsWith("data:")) return url;
	try {
		return localStorage.getItem(cacheKey(url));
	} catch {
		return null;
	}
}

export async function primeAvatar(url?: string | null): Promise<void> {
	if (!url || typeof window === "undefined") return;
	if (url.startsWith("data:") || getCachedAvatar(url)) return;
	try {
		const response = await fetch(url, { cache: "force-cache" });
		if (!response.ok) return;
		const blob = await response.blob();
		if (blob.size === 0 || blob.size > 400_000) return;
		const dataUrl = await new Promise<string>((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => resolve(String(reader.result));
			reader.onerror = () => reject(new Error("read failed"));
			reader.readAsDataURL(blob);
		});
		localStorage.setItem(cacheKey(url), dataUrl);
	} catch {
		/* cache is best effort */
	}
}
