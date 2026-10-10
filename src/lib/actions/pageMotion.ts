import { tick } from 'svelte';
import { settings } from '$lib/stores/settings.svelte';

const surfaces = '[data-motion-surface], .surface-glass, .catalog-card, [data-bedrock-instance], .friend-row, .instance-overview-section, header, h1, h2';

export function pageMotion(node: HTMLElement, route: string) {
	let frame = 0;
	let generation = 0;
	let animations: Animation[] = [];
	const reduced = matchMedia('(prefers-reduced-motion: reduce)');
	function cancel() {
		generation++;
		cancelAnimationFrame(frame);
		animations.forEach(animation => animation.cancel());
		animations = [];
	}
	function play(target: HTMLElement, frames: Keyframe[], duration: number, delay = 0) {
		const animation = target.animate(frames, { duration, delay, easing: 'cubic-bezier(.16,1,.3,1)', fill: 'backwards' });
		animation.id = 'luxmc-page-entry';
		animations.push(animation);
		animation.onfinish = () => {
			animation.cancel();
			animations = animations.filter(item => item !== animation);
		};
	}
	async function enter() {
		cancel();
		const current = generation;
		await tick();
		if (current !== generation || !node.isConnected) return;
		frame = requestAnimationFrame(() => {
			const root = document.documentElement;
			if (!root.classList.contains('settings-ready')) return;
			if ((reduced.matches && !root.classList.contains('force-ui-motion')) || document.hidden || root.classList.contains('no-ui-motion')) return;
			const economical = root.classList.contains('no-anim');
            const expressive = settings.value.motionStyle !== 'subtle';
            const speed = settings.value.motionSpeed === 'fast' ? .65 : 1;
			play(node, [{ opacity: .15 }, { opacity: 1 }], economical ? 160 : 360 * speed);
			if (economical) return;
			const bounds = node.closest('[data-scroll-root]')?.getBoundingClientRect() ?? node.getBoundingClientRect();
			const candidates = Array.from(node.querySelectorAll<HTMLElement>(surfaces));
			if (!node.hasAttribute('data-page-route')) {
				candidates.push(...Array.from(node.children).filter((child): child is HTMLElement => child instanceof HTMLElement && !child.querySelector('[role="dialog"]')));
			}
			const targets = [...new Set(candidates)].filter(target => {
				const rect = target.getBoundingClientRect();
				return rect.width > 0 && rect.height > 0 && rect.bottom > bounds.top && rect.top < bounds.bottom && !target.closest('[role="dialog"]');
			}).filter((target, _index, all) => !all.some(parent => parent !== target && parent.contains(target))).slice(0, 12);
			targets.forEach((target, index) => play(target, [
				{ opacity: 0, transform: `translateY(${expressive ? 32 : 12}px)` },
				{ opacity: 1, transform: 'translateY(0)' }
			], (expressive ? 560 : 320) * speed, Math.min(index * (expressive ? 45 : 20), 180) * speed));
		});
	}
	void enter();
	const ready = () => { void enter(); };
	document.addEventListener('luxmc-settings-ready', ready);
	return {
		update(next: string) { if (route !== next) { route = next; void enter(); } },
		destroy() { cancel(); document.removeEventListener('luxmc-settings-ready', ready); }
	};
}
