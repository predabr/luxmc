import { settings } from '$lib/stores/settings.svelte';

const seen = new Map<string, number>();
const observers = new Map<Element | null, { observer: IntersectionObserver; targets: Map<Element, () => void> }>();

function observe(node: HTMLElement, play: () => void) {
    const root = node.closest('main');
    let group = observers.get(root);
    if (!group) {
        const targets = new Map<Element, () => void>();
        const observer = new IntersectionObserver(entries => {
            for (const entry of entries) if (entry.isIntersecting) targets.get(entry.target)?.();
        }, { root, threshold: .08 });
        group = { observer, targets };
        observers.set(root, group);
    }
    const current = group;
    let active = true;
    current.targets.set(node, play);
    current.observer.observe(node);
    return () => {
        if (!active) return;
        active = false;
        current.observer.unobserve(node);
        current.targets.delete(node);
        if (!current.targets.size) { current.observer.disconnect(); observers.delete(root); }
    };
}

export function catalogReveal(node: HTMLElement, key?: string) {
    let animation: Animation | null = null;
    let stop: (() => void) | undefined;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    const cancel = () => { animation?.cancel(); animation = null; };
    const play = () => {
        stop?.();
        const preference = settings.value;
        if (preference.animations === false || preference.performanceMode || document.hidden || (preference.respectReducedMotion !== false && reduced.matches)) return;
        if (key) {
            if ((seen.get(key) ?? 0) > Date.now() - 120_000) return;
            if (seen.size >= 300) seen.delete(seen.keys().next().value!);
            seen.set(key, Date.now());
        }
        const expressive = preference.motionStyle !== 'subtle';
        animation = node.animate([
            { opacity: 0, transform: `translateY(${expressive ? 18 : 8}px) scale(${expressive ? .985 : 1})` },
            { opacity: 1, transform: 'translateY(0) scale(1)' }
        ], { duration: (expressive ? 380 : 200) * (preference.motionSpeed === 'fast' ? .65 : 1), easing: 'cubic-bezier(.16,1,.3,1)' });
        animation.id = 'luxmc-catalog-reveal';
        animation.onfinish = cancel;
    };
    const visibility = () => { if (document.hidden) cancel(); };
    const motion = () => { if (reduced.matches && settings.value.respectReducedMotion !== false) cancel(); };
    stop = observe(node, play);
    document.addEventListener('visibilitychange', visibility);
    reduced.addEventListener('change', motion);
    return {
        update(next?: string) { if (key !== next) { key = next; stop?.(); cancel(); stop = observe(node, play); } },
        destroy() { stop?.(); cancel(); document.removeEventListener('visibilitychange', visibility); reduced.removeEventListener('change', motion); }
    };
}
