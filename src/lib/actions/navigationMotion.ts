import { tick } from 'svelte';

export function navigationMotion(node: HTMLElement, route: string) {
    const rail = document.createElement('span');
    rail.className = 'launcher-nav-indicator';
    rail.setAttribute('aria-hidden', 'true');
    node.append(rail);
    let frame = 0;
    let disposed = false;
    const update = () => {
        frame = 0;
        const active = node.querySelector<HTMLElement>('.launcher-nav-link[aria-current="page"]');
        if (!active) { rail.style.opacity = '0'; return; }
        const bounds = active.getBoundingClientRect();
        const parent = node.getBoundingClientRect();
        const height = Math.min(26, bounds.height);
        rail.style.height = `${height}px`;
        rail.style.transform = `translateY(${bounds.top - parent.top + (bounds.height - height) / 2}px)`;
        rail.style.opacity = bounds.height > 0 ? '1' : '0';
    };
    const schedule = () => { if (!disposed && !frame) frame = requestAnimationFrame(update); };
    const observe = new ResizeObserver(schedule);
    observe.observe(node);
    const changes = new MutationObserver(schedule);
    changes.observe(node, { subtree: true, attributes: true, attributeFilter: ['aria-current'], childList: true });
    node.addEventListener('scroll', schedule, { passive: true, capture: true });
    schedule();
    return {
        update(next: string) { if (next !== route) { route = next; void tick().then(schedule); } },
        destroy() { disposed = true; cancelAnimationFrame(frame); observe.disconnect(); changes.disconnect(); node.removeEventListener('scroll', schedule, true); rail.remove(); }
    };
}
