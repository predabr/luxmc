const SCROLL_IDLE_MS = 80;
const LERP_FACTOR = 0.18;
const STEP_MULTIPLIER = 1.8;
const MAX_STEP = 320;
const STOP_THRESHOLD = 0.5;

export function initSmoothScroll(): () => void {
	if (typeof window === "undefined") return () => {};

	let isScrollingTimer: ReturnType<typeof setTimeout> | null = null;
	let animId: number | null = null;
	let activeElement: HTMLElement | null = null;
	let currentScroll = 0;
	let targetScroll = 0;

	function markScrolling() {
		document.body.classList.add("is-scrolling");
		if (isScrollingTimer) clearTimeout(isScrollingTimer);
		isScrollingTimer = setTimeout(() => {
			document.body.classList.remove("is-scrolling");
		}, SCROLL_IDLE_MS);
	}

	function hasRoom(el: HTMLElement) {
		return el.scrollHeight - el.clientHeight > 2;
	}

	function findScrollable(start: EventTarget | null): HTMLElement | null {
		if (!(start instanceof Element)) {
			const fallback = document.querySelector<HTMLElement>("main");
			return fallback && hasRoom(fallback) ? fallback : document.documentElement;
		}

		const dialog = start.closest<HTMLElement>("[role='dialog'], [data-no-page-scroll]");
		if (dialog) {
			const inner = start.closest<HTMLElement>("[data-scroll-root], .custom-scrollbar");
			return inner && dialog.contains(inner) && hasRoom(inner) ? inner : null;
		}

		const explicit = start.closest<HTMLElement>("[data-scroll-root], .custom-scrollbar");
		if (explicit && hasRoom(explicit)) return explicit;
		const main = document.querySelector<HTMLElement>("main");
		if (main && hasRoom(main)) return main;
		return document.documentElement;
	}

	function onWheel(e: WheelEvent) {
		if (e.defaultPrevented) return;
		if (e.ctrlKey || e.metaKey || e.altKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
		if (Math.abs(e.deltaY) < 0.01) return;

		const target = findScrollable(e.target);
		if (!target) return;

		const maxScroll = target.scrollHeight - target.clientHeight;
		if (maxScroll <= 0) return;

		if ((target.scrollTop <= 0 && e.deltaY < 0) || (target.scrollTop >= maxScroll && e.deltaY > 0)) {
			return;
		}

		e.preventDefault();
		markScrolling();

		if (activeElement !== target) {
			activeElement = target;
			currentScroll = target.scrollTop;
			targetScroll = target.scrollTop;
		}

		let delta = e.deltaY;
		if (e.deltaMode === 1) delta *= 40;
		else if (e.deltaMode === 2) delta *= target.clientHeight * 0.9;

		const step = Math.max(-MAX_STEP, Math.min(MAX_STEP, delta * STEP_MULTIPLIER));
		targetScroll = Math.max(0, Math.min(maxScroll, targetScroll + step));

		if (animId === null) {
			const tick = () => {
				if (!activeElement) {
					animId = null;
					return;
				}

				const diff = targetScroll - currentScroll;
				if (Math.abs(diff) < STOP_THRESHOLD) {
					currentScroll = targetScroll;
					activeElement.scrollTop = currentScroll;
					animId = null;
					return;
				}

				currentScroll += diff * LERP_FACTOR;
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
