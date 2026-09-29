import { preloadData } from "$app/navigation";

const TTL_MS = 30_000;
const MAX_TRACKED = 64;
const DEBOUNCE_MS = 140;

const seen = new Map<string, number>();
let timer: ReturnType<typeof setTimeout> | undefined;
let queued: string | undefined;

function flush(url: string) {
	const now = Date.now();
	const at = seen.get(url);
	if (at !== undefined && now - at < TTL_MS) return;
	if (seen.size >= MAX_TRACKED) seen.clear();
	seen.set(url, now);
	void preloadData(url);
}

export function preloadRoute(url: string, immediate = false) {
	if (immediate) {
		if (timer) {
			clearTimeout(timer);
			timer = undefined;
			queued = undefined;
		}
		flush(url);
		return;
	}
	queued = url;
	if (timer) return;
	timer = setTimeout(() => {
		timer = undefined;
		const target = queued;
		queued = undefined;
		if (target) flush(target);
	}, DEBOUNCE_MS);
}
