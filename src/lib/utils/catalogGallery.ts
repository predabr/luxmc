import { modsProjectDetails } from "$lib/api/mods";
import { isRenderableBanner } from "./curatedBanners";

const cache = new Map<string, { expires: number; urls: string[] }>();
const pending = new Map<string, Promise<string[]>>();
const queue: Array<() => void> = [];
let active = 0;

function drain() {
	while (active < 3 && queue.length) queue.shift()!();
}

export function descriptionArtwork(body: string): string[] {
    const html = [...body.matchAll(/<img\b[^>]*\bsrc\s*=\s*["']([^"']+)["']/gi)].map(match => match[1]);
    const markdown = [...body.matchAll(/!\[[^\]]*\]\(<?([^\s)>]+)>?(?:\s+["'][^"']*["'])?\)/g)].map(match => match[1]);
    return [...new Set([...html, ...markdown].map(url => url.replace(/&amp;/g, '&')).filter(url => isRenderableBanner(url) && !/(?:shields\.io|\/partners\/|promocard|donation|sponsor|badge|_important|_summary|_features)/i.test(url)))].slice(0, 8);
}

export function catalogGallery(source: string, projectId: string): Promise<string[]> {
	if (!projectId || !["modrinth", "curseforge"].includes(source)) return Promise.resolve([]);
	const key = `${source}:${projectId}`;
	const cached = cache.get(key);
	if (cached && cached.expires > Date.now()) return Promise.resolve(cached.urls);
	const request = pending.get(key);
	if (request) return request;
	const next = new Promise<string[]>(resolve => {
		queue.push(() => {
			active += 1;
			void modsProjectDetails(projectId, source).then(async details => {
                const matches = details?.id === projectId && (!details.source || details.source === source);
                const gallery = matches ? (details.gallery || []).map(image => image.url).filter(isRenderableBanner) : [];
                const authorCover = matches && !gallery.length ? await firstLandscape(descriptionArtwork(details.body || '')) : undefined;
				const urls = matches
					? [...new Set([...gallery, ...(authorCover ? [authorCover] : []), ...(details.iconUrl && isRenderableBanner(details.iconUrl) ? [details.iconUrl] : [])])].slice(0, 8)
					: [];
				if (cache.size >= 128) cache.delete(cache.keys().next().value!);
				cache.set(key, { urls, expires: Date.now() + (urls.length ? 600_000 : 60_000) });
				resolve(urls);
			}).catch(() => {
				cache.set(key, { urls: [], expires: Date.now() + 30_000 });
				resolve([]);
			}).finally(() => {
				active -= 1;
				pending.delete(key);
				drain();
			});
		});
	});
	pending.set(key, next);
	drain();
	return next;
}

export function latestMinecraftVersion(versions: string[]): string | undefined {
	return versions.filter(version => /^(?:\d+\.\d+(?:\.\d+)?(?:[- ](?:pre|rc)\d+)?|\d{2}w\d{2}[a-z]|[ab]\d+\.\d+(?:\.\d+)?(?:_\d+)?)$/i.test(version))
		.sort((a, b) => b.localeCompare(a, undefined, { numeric: true }))[0];
}

export async function firstLandscape(urls: string[]): Promise<string | undefined> {
	if (typeof Image === "undefined") return undefined;
	for (const url of urls.slice(0, 8)) {
		if (!isRenderableBanner(url)) continue;
		const landscape = await new Promise<boolean>(resolve => {
			const image = new Image();
			const timeout = setTimeout(() => { image.src = ""; resolve(false); }, 5000);
			image.referrerPolicy = "no-referrer";
			image.onload = () => { clearTimeout(timeout); const aspect = image.naturalWidth / Math.max(1, image.naturalHeight); resolve(image.naturalWidth >= 480 && image.naturalHeight >= 140 && aspect > 1.2 && aspect < 4); };
			image.onerror = () => { clearTimeout(timeout); resolve(false); };
			image.src = url;
		});
		if (landscape) return url;
	}
	return undefined;
}
