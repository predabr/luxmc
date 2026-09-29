export function initSmoothScroll(): () => void {
	if (typeof window === "undefined") return () => {};

	let isScrollingTimer: ReturnType<typeof setTimeout> | null = null;
	let animId: number | null = null;
	let activeElement: HTMLElement | null = null;
	let currentScroll = 0;
	let targetScroll = 0;

	function findScrollable(start: HTMLElement | null): HTMLElement | null {
		let el: HTMLElement | null = start;
		while (el && el !== document.body && el !== document.documentElement) {
			const style = window.getComputedStyle(el);
			const overflowY = style.overflowY;
			if ((overflowY === "auto" || overflowY === "scroll") && el.scrollHeight > el.clientHeight + 2) {
				return el;
			}
			el = el.parentElement;
		}
		const main = document.querySelector("main");
		if (main && main.scrollHeight > main.clientHeight + 2) return main as HTMLElement;
		return document.documentElement;
	}

	function onWheel(e: WheelEvent) {
		// Ignore zoom (Ctrl/Cmd + wheel) or horizontal scrolls
		if (e.ctrlKey || e.metaKey || e.altKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;

		// Precision trackpads emit continuous tiny fractional deltas (< 10) or non-standard steps.
		// Mouse wheels on Linux/Windows emit discrete notches (usually >= 30 or deltaMode != 0).
		const isTrackpad = Math.abs(e.deltaY) < 12 && Number.isInteger(e.deltaY) === false;
		if (isTrackpad) return;

		const target = findScrollable(e.target as HTMLElement);
		if (!target) return;

		const maxScroll = target.scrollHeight - target.clientHeight;
		if (maxScroll <= 0) return;

		// Don't intercept if already at bounds and trying to scroll further
		if ((target.scrollTop <= 0 && e.deltaY < 0) || (target.scrollTop >= maxScroll && e.deltaY > 0)) {
			return;
		}

		e.preventDefault();

		// Disable pointer-events while scrolling to prevent expensive :hover / transition storms
		document.body.classList.add("is-scrolling");
		if (isScrollingTimer) clearTimeout(isScrollingTimer);
		isScrollingTimer = setTimeout(() => {
			document.body.classList.remove("is-scrolling");
		}, 100);

		if (activeElement !== target) {
			activeElement = target;
			currentScroll = target.scrollTop;
			targetScroll = target.scrollTop;
		}

		// Calculate delta
		let delta = e.deltaY;
		if (e.deltaMode === 1) delta *= 30; // lines
		else if (e.deltaMode === 2) delta *= target.clientHeight; // pages

		// Dampen huge wheel bursts, then scale for comfortable travel
		const step = Math.sign(delta) * Math.min(Math.abs(delta), 140);
		targetScroll = Math.max(0, Math.min(maxScroll, targetScroll + step * 1.1));

		if (animId === null) {
			const tick = () => {
				if (!activeElement) {
					animId = null;
					return;
				}

				const diff = targetScroll - currentScroll;
				if (Math.abs(diff) < 0.6) {
					currentScroll = targetScroll;
					activeElement.scrollTop = currentScroll;
					animId = null;
					return;
				}

				// Silky smooth responsive lerp (0.2 gives snappy 60fps response with zero lag)
				currentScroll += diff * 0.2;
				activeElement.scrollTop = currentScroll;
				animId = requestAnimationFrame(tick);
			};
			animId = requestAnimationFrame(tick);
		}
	}

	window.addEventListener("wheel", onWheel, { passive: false });

	return () => {
		window.removeEventListener("wheel", onWheel);
		if (animId !== null) cancelAnimationFrame(animId);
		if (isScrollingTimer) clearTimeout(isScrollingTimer);
		document.body.classList.remove("is-scrolling");
	};
}
