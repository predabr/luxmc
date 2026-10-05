export function createAnimationClock(start: number, interval = 1000 / 60) {
    let previous = start;
    return {
        reset(now: number) { previous = now; },
        take(now: number): number {
            const elapsed = now - previous;
            if (!Number.isFinite(elapsed) || elapsed < interval - 1) return 0;
            previous = now;
            return Math.min(elapsed / 1000, .05);
        }
    };
}
