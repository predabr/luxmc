export function pageMotion(node: HTMLElement, route: string) {
	let frame = 0;
	let animations: Animation[] = [];
	const reduced = matchMedia('(prefers-reduced-motion: reduce)');
	function cancel() {
		cancelAnimationFrame(frame);
		animations.forEach(animation => animation.cancel());
		animations = [];
	}
	function enter() {
		cancel();
		frame = requestAnimationFrame(() => {
			const root = document.documentElement;
            if (!root.classList.contains('settings-ready')) return;
            if ((reduced.matches && !root.classList.contains('force-ui-motion')) || document.hidden || root.classList.contains('no-ui-motion')) return;
            const economical = root.classList.contains('no-anim');
			animations.push(node.animate([{ opacity: .65 }, { opacity: 1 }], { duration: economical ? 140 : 220, easing: 'ease-out' }));
			const heading = node.querySelector('h1');
			if (heading && !economical) animations.push(heading.animate([{ transform: 'translateY(8px)' }, { transform: 'translateY(0)' }], { duration: 280, easing: 'cubic-bezier(.2,.8,.2,1)' }));
		});
	}
	enter();
	document.addEventListener('luxmc-settings-ready', enter);
	return {
		update(next: string) { if (route !== next) { route = next; enter(); } },
		destroy() { cancel(); document.removeEventListener('luxmc-settings-ready', enter); }
	};
}
