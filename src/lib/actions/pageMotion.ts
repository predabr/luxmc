import { tick } from 'svelte';
import { settings } from '$lib/stores/settings.svelte';

const surfaces = '[data-motion-surface], .surface-glass, .luxmc-card, [data-bedrock-instance], .friend-row, .instance-library-card, .instance-settings-group, .instance-overview-section, header, h1, h2';

export function pageMotion(node: HTMLElement, route: string) {
    node.dataset.motionScope = 'page';
    let frame = 0;
    let generation = 0;
    const animations = new Set<Animation>();
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    function cancel() {
        generation++;
        cancelAnimationFrame(frame);
        for (const animation of animations) animation.cancel();
        animations.clear();
    }
    function enabled() {
        const root = document.documentElement;
        return root.classList.contains('settings-ready') && !document.hidden &&
            !root.classList.contains('no-ui-motion') && !root.classList.contains('no-anim') &&
            !(reduced.matches && settings.value.respectReducedMotion !== false);
    }
    async function enter() {
        cancel();
        const current = generation;
        await tick();
        if (current !== generation || !node.isConnected || !enabled()) return;
        frame = requestAnimationFrame(() => {
            if (current !== generation || !node.isConnected || !enabled()) return;
            const candidates = Array.from(node.querySelectorAll<HTMLElement>(surfaces));
            const candidateSet = new Set(candidates);
            const bounds = node.closest('[data-scroll-root]')?.getBoundingClientRect() ?? node.getBoundingClientRect();
            const targets: HTMLElement[] = [];
            for (const target of candidates) {
                if (target.closest('[role="dialog"]') || target.closest('[data-motion-scope]') !== node) continue;
                let parent = target.parentElement;
                while (parent && parent !== node && !candidateSet.has(parent)) parent = parent.parentElement;
                if (parent && parent !== node) continue;
                const rect = target.getBoundingClientRect();
                if (rect.width && rect.height && rect.bottom > bounds.top && rect.top < bounds.bottom) targets.push(target);
                if (targets.length === 4) break;
            }
            if (!targets.length && !node.hasAttribute('data-page-route') && !node.querySelector('[data-motion-scope]')) targets.push(node);
            const expressive = settings.value.motionStyle !== 'subtle';
            const duration = settings.value.motionSpeed === 'fast' ? 120 : 180;
            for (const target of targets) {
                const animation = target.animate([
                    { opacity: .8, translate: `0 ${expressive ? 6 : 2}px` },
                    { opacity: 1, translate: '0 0' }
                ], { duration, easing: 'cubic-bezier(.2,.8,.2,1)' });
                animation.id = 'luxmc-page-entry';
                animations.add(animation);
                animation.onfinish = () => { animation.cancel(); animations.delete(animation); };
            }
        });
    }
    const ready = () => { void enter(); };
    const interrupt = () => { if (!enabled()) cancel(); };
    document.addEventListener('luxmc-settings-ready', ready);
    document.addEventListener('visibilitychange', interrupt);
    reduced.addEventListener('change', interrupt);
    void enter();
    return {
        update(next: string) { if (route !== next) { route = next; void enter(); } },
        destroy() {
            cancel();
            delete node.dataset.motionScope;
            document.removeEventListener('luxmc-settings-ready', ready);
            document.removeEventListener('visibilitychange', interrupt);
            reduced.removeEventListener('change', interrupt);
        }
    };
}
