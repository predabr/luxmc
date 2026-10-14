import { modsProjectDetails } from "$lib/api/mods";
import { isRenderableBanner, isProjectArtwork } from "./curatedBanners";

const cache = new Map<string, { expires: number; urls: string[] }>();
const pending = new Map<string, Promise<string[]>>();
const queue: Array<() => void> = [];
let active = 0;

function drain() {
	while (active < 3 && queue.length) queue.shift()!();
}

export function descriptionArtwork(body: string): string[] {
    const html = [...body.matchAll(/<img\b[^>]*>/gi)].flatMap(match => {
        const url = match[0].match(/\bsrc\s*=\s*["']([^"']+)["']/i)?.[1]?.replace(/&amp;/g, '&');
        if (!url) return [];
        const before = body.slice(0, match.index);
        const anchor = before.lastIndexOf('<a ');
        const link = anchor > before.lastIndexOf('</a>') ? before.slice(anchor).match(/\bhref\s*=\s*["']([^"']+)["']/i)?.[1] : undefined;
        const title = [...match[0].matchAll(/\b(?:alt|title)\s*=\s*["']([^"']*)["']/gi)].map(value => value[1]).join(' ');
        return isProjectArtwork({ url, title, link }) ? [url] : [];
    });
    const markdown = [...body.matchAll(/!\[([^\]]*)\]\(<?([^\s)>]+)>?(?:\s+["'][^"']*["'])?\)/g)].flatMap(match => {
        const url = match[2].replace(/&amp;/g, '&');
        const link = body.slice((match.index || 0) + match[0].length).match(/^\]\(([^)]+)\)/)?.[1];
        return isProjectArtwork({ url, title: match[1], link }) ? [url] : [];
    });
    return [...new Set([...html, ...markdown])].slice(0, 8);
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
                const gallery = matches ? (details.gallery || []).filter(isProjectArtwork).map(image => image.url) : [];
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

const landscapeCache = new Map<string, { expires: number; landscape: boolean }>();
const landscapePending = new Map<string, Promise<boolean>>();
function checkLandscape(url: string): Promise<boolean> {
    const cached = landscapeCache.get(url);
    if (cached && cached.expires > Date.now()) return Promise.resolve(cached.landscape);
    const pending = landscapePending.get(url);
    if (pending) return pending;
    const request = new Promise<boolean>(resolve => {
			const image = new Image();
            const finish = (landscape: boolean) => {
                clearTimeout(timeout); image.onload = null; image.onerror = null;
                if (landscapeCache.size >= 256) landscapeCache.delete(landscapeCache.keys().next().value!);
                landscapeCache.set(url, { landscape, expires: Date.now() + (landscape ? 600_000 : 60_000) });
                resolve(landscape);
            };
			const timeout = setTimeout(() => { finish(false); image.src = ''; }, 3000);
			image.referrerPolicy = "no-referrer";
			image.onload = () => { const aspect = image.naturalWidth / Math.max(1, image.naturalHeight); finish(image.naturalWidth >= 480 && image.naturalHeight >= 140 && aspect > 1.2 && aspect < 4); };
			image.onerror = () => finish(false);
			image.src = url;
    }).finally(() => landscapePending.delete(url));
    landscapePending.set(url, request);
    return request;
}
export async function firstLandscape(urls: string[]): Promise<string | undefined> {
	if (typeof Image === "undefined") return undefined;
    const candidates = [...new Set(urls)].filter(isRenderableBanner).slice(0, 8);
	for (let offset = 0; offset < candidates.length; offset += 3) {
        const batch = candidates.slice(offset, offset + 3);
        const found = await Promise.all(batch.map(checkLandscape));
        const index = found.indexOf(true);
        if (index >= 0) return batch[index];
	}
	return undefined;
}
