import { instanceContentIcons } from '$lib/api/instances';
import { rememberContentIcons } from '$lib/utils/contentPrefetch';
import { settings } from '$lib/stores/settings.svelte';
import { appState } from '$lib/stores/app.svelte';

type Target = { profileId: string; folder: string; name: string; iconKey?: string };
const attempts = new Map<string, { time: number; request: Promise<string | null> }>();

function recover(target: Target) {
    const key = JSON.stringify([target.profileId, target.folder, target.name, target.iconKey]);
    const previous = attempts.get(key);
    if (previous && Date.now() - previous.time < 30000) return previous.request;
    if (attempts.size >= 128) attempts.delete(attempts.keys().next().value!);
    const request = instanceContentIcons(target.profileId, target.folder, [target.name], true, true).then(icons => {
        rememberContentIcons(icons);
        return icons.find(icon => icon.name === target.name)?.icon ?? null;
    }).catch(() => null);
    attempts.set(key, { time: Date.now(), request });
    return request;
}

export function contentImage(node: HTMLImageElement, initial: Target) {
    let target = initial;
    let generation = 0;
    let failed = '';
    let reveal: Animation | undefined;
    const loaded = () => {
        node.style.visibility = 'visible';
        reveal?.cancel();
        if (settings.value.animations === false || appState.performanceMode || appState.isLaunching || appState.isGameRunning || document.hidden || matchMedia('(prefers-reduced-motion: reduce)').matches && settings.value.respectReducedMotion !== false) return;
        reveal = node.animate([{ opacity: 0, scale: '.94' }, { opacity: 1, scale: '1' }], { duration: settings.value.motionSpeed === 'fast' ? 120 : 200, easing: 'ease-out' });
        reveal.onfinish = () => { reveal?.cancel(); reveal = undefined; };
    };
    const error = async () => {
        const source = node.getAttribute('src') ?? '';
        node.style.visibility = 'hidden';
        if (!source.startsWith('https://') || source === failed) return;
        failed = source;
        const current = ++generation;
        const recovered = await recover(target);
        if (current !== generation || !node.isConnected || node.getAttribute('src') !== source || !recovered || recovered === source) return;
        node.src = recovered;
    };
    node.addEventListener('load', loaded);
    node.addEventListener('error', error);
    if (node.complete && !node.naturalWidth && node.getAttribute('src')) void error();
    return {
        update(next: Target) {
            if (JSON.stringify(next) !== JSON.stringify(target)) { generation++; failed = ''; }
            target = next;
        },
        destroy() { generation++; reveal?.cancel(); node.removeEventListener('load', loaded); node.removeEventListener('error', error); }
    };
}
