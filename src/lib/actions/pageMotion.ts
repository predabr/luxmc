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
			if (reduced.matches || document.hidden || document.documentElement.classList.contains('no-anim')) return;
			animations.push(node.animate([{ opacity: .65 }, { opacity: 1 }], { duration: 220, easing: 'ease-out' }));
			const heading = node.querySelector('h1');
			if (heading) animations.push(heading.animate([{ transform: 'translateY(8px)' }, { transform: 'translateY(0)' }], { duration: 280, easing: 'cubic-bezier(.2,.8,.2,1)' }));
		});
	}
	enter();
	return {
		update(next: string) { if (route !== next) { route = next; enter(); } },
		destroy: cancel
	};
}
