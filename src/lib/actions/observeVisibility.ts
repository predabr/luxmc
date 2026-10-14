let observer: IntersectionObserver | undefined;
const callbacks = new Map<Element, { visible: boolean; callback: (visible: boolean) => void }>();

export function observeVisibility(node: HTMLElement, callback?: (visible: boolean) => void) {
    let enabled = false;
    const attach = (next?: (visible: boolean) => void) => {
        const previous = callbacks.get(node);
        if (previous?.visible) previous.callback(false);
        if (enabled) observer?.unobserve(node);
        callbacks.delete(node);
        enabled = Boolean(next);
        if (!next) return;
        if (typeof IntersectionObserver === 'undefined') { callbacks.set(node, { visible: true, callback: next }); next(true); return; }
        observer ??= new IntersectionObserver(entries => {
            for (const entry of entries) {
                const current = callbacks.get(entry.target);
                if (current && current.visible !== entry.isIntersecting) { current.visible = entry.isIntersecting; current.callback(entry.isIntersecting); }
            }
        }, { rootMargin: '120px' });
        callbacks.set(node, { visible: previous?.visible ?? false, callback: next });
        if (previous?.visible) next(true);
        observer.observe(node);
    };
    attach(callback);
    return { update: attach, destroy() { attach(); if (!callbacks.size) { observer?.disconnect(); observer = undefined; } } };
}
