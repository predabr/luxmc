import { preloadData } from "$app/navigation";

const TTL_MS = 30_000;
const MAX_TRACKED = 64;
const HOVER_DEBOUNCE_MS = 260;
const IDLE_TIMEOUT_MS = 400;

const seen = new Map<string, number>();
let timer: ReturnType<typeof setTimeout> | undefined;
let queued: string | undefined;
let idleHandle: number | undefined;

const hasIdle =
	typeof window !== "undefined" && typeof window.requestIdleCallback === "function";

function schedule(fn: () => void) {
	if (hasIdle) {
		if (idleHandle !== undefined) window.cancelIdleCallback(idleHandle);
		idleHandle = window.requestIdleCallback(
			() => {
				idleHandle = undefined;
				fn();
			},
			{ timeout: IDLE_TIMEOUT_MS },
		);
		return;
	}
	window.setTimeout(fn, 60);
}

function flush(url: string, defer = true) {
	const now = Date.now();
	const at = seen.get(url);
	if (at !== undefined && now - at < TTL_MS) return;
	if (seen.size >= MAX_TRACKED) seen.clear();
	seen.set(url, now);
	if (!defer) {
		void preloadData(url);
		return;
	}
	schedule(() => {
		void preloadData(url);
	});
}

export function preloadRoute(url: string, immediate = false) {
	if (immediate) {
		if (timer) {
			clearTimeout(timer);
			timer = undefined;
			queued = undefined;
		}
		flush(url, false);
		return;
	}
	queued = url;
	if (timer) return;
	timer = setTimeout(() => {
		timer = undefined;
		const target = queued;
		queued = undefined;
		if (target) flush(target);
	}, HOVER_DEBOUNCE_MS);
}
