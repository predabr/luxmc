import { settings } from '$lib/stores/settings.svelte';
import { appState } from '$lib/stores/app.svelte';

export function buttonFeedback(node: HTMLElement) {
    const reduced = matchMedia('(prefers-reduced-motion: reduce)');
    const active = new Map<HTMLElement, { animation: Animation; wave: HTMLElement }>();
    const clear = () => { for (const { animation, wave } of active.values()) { animation.cancel(); wave.remove(); } active.clear(); };
    const click = (event: MouseEvent) => {
        if (settings.value.animations === false || appState.performanceMode || appState.isLaunching || appState.isGameRunning || document.hidden || reduced.matches && settings.value.respectReducedMotion !== false) return;
        const button = (event.target as Element)?.closest<HTMLElement>('.launcher-button--primary, .launcher-button--play, .launcher-button--secondary');
        if (!button || !node.contains(button) || button.matches(':disabled, [aria-disabled="true"]')) return;
        const previous = active.get(button);
        previous?.animation.cancel(); previous?.wave.remove();
        const wave = document.createElement('span');
        wave.className = 'launcher-button-wave';
        wave.setAttribute('aria-hidden', 'true');
        const rect = button.getBoundingClientRect();
        const size = Math.max(rect.width, rect.height) * 2;
        Object.assign(wave.style, { width: `${size}px`, height: `${size}px`, left: `${(event.detail ? event.clientX - rect.left : rect.width / 2) - size / 2}px`, top: `${(event.detail ? event.clientY - rect.top : rect.height / 2) - size / 2}px` });
        button.append(wave);
        const animation = wave.animate([{ transform: 'scale(0)', opacity: .5 }, { transform: 'scale(1)', opacity: 0 }], { duration: settings.value.motionSpeed === 'fast' ? 300 : 480, easing: 'cubic-bezier(.16,1,.3,1)' });
        active.set(button, { animation, wave });
        animation.onfinish = () => { wave.remove(); if (active.get(button)?.animation === animation) active.delete(button); animation.cancel(); };
    };
    const visibility = () => { if (document.hidden) clear(); };
    node.addEventListener('click', click);
    document.addEventListener('visibilitychange', visibility);
    return { destroy() { clear(); node.removeEventListener('click', click); document.removeEventListener('visibilitychange', visibility); } };
}
