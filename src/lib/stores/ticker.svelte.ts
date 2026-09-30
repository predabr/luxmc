const TICK_MS = 60_000;

let now = $state(Date.now());
let timer: ReturnType<typeof setInterval> | null = null;
let subscribers = 0;

export function startTicker() {
	subscribers += 1;
	if (timer) return;
	now = Date.now();
	timer = setInterval(() => {
		now = Date.now();
	}, TICK_MS);
}

export function stopTicker() {
	if (subscribers === 0) return;
	subscribers -= 1;
	if (subscribers > 0 || !timer) return;
	clearInterval(timer);
	timer = null;
}

export function tickerNow(): number {
	return now;
}
