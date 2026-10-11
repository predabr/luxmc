import { settings } from '$lib/stores/settings.svelte';

export function catalogReveal(node: HTMLElement) {
    let animation: Animation | null = null;
    let observer: IntersectionObserver | null = null;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    const play = () => {
        observer?.disconnect();
        const preference = settings.value;
        if (preference.animations === false || preference.performanceMode || document.hidden || (preference.respectReducedMotion !== false && reduced.matches)) return;
        const expressive = preference.motionStyle !== 'subtle';
        animation = node.animate([
            { opacity: 0, transform: `translateY(${expressive ? 22 : 10}px) scale(${expressive ? .98 : 1})` },
            { opacity: 1, transform: 'translateY(0) scale(1)' }
        ], { duration: (expressive ? 420 : 220) * (preference.motionSpeed === 'fast' ? .65 : 1), easing: 'cubic-bezier(.16,1,.3,1)' });
        animation.id = 'luxmc-catalog-reveal';
        animation.onfinish = () => { animation?.cancel(); animation = null; };
    };
    observer = new IntersectionObserver(entries => { if (entries.some(entry => entry.isIntersecting)) play(); }, { root: node.closest('main'), threshold: .08 });
    observer.observe(node);
    return { destroy() { observer?.disconnect(); animation?.cancel(); } };
}
