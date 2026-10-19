import { instanceContentIcons, instanceFileTree } from '$lib/api/instances';
import type { FileTreeEntry } from '$lib/api/types';
import { settings } from '$lib/stores/settings.svelte';
import { appState } from '$lib/stores/app.svelte';

const icons = new Map<string, { icon: string | null; resolved: boolean }>();
type IconResult = { iconKey: string; icon: string | null; resolved: boolean };
const listeners = new Set<(entries: IconResult[]) => void>();
export function subscribeContentIcons(listener: (entries: IconResult[]) => void) {
    listeners.add(listener);
    return () => { listeners.delete(listener); };
}
const pending = new Map<string, Promise<void>>();
const warmed = new Map<string, number>();
const downloaded = new Set<string>();
const images: string[] = [];
let downloading = 0;
let prefetchTail = Promise.resolve();
function drainImages() {
    while (images.length && downloading < 4 && settings.value.preloadContentIcons !== false && !document.hidden && !appState.isLaunching && !appState.isGameRunning) {
        const source = images.shift()!;
        const image = new Image();
        image.decoding = 'async'; downloading++;
        const timer = setTimeout(() => finish(false), 10000);
        let done = false;
        function finish(success: boolean) {
            if (done) return; done = true;
            clearTimeout(timer); image.onload = null; image.onerror = null;
            if (!success) { downloaded.delete(source); image.src = ''; }
            downloading--; drainImages();
        }
        image.onload = () => finish(true); image.onerror = () => finish(false); image.src = source;
    }
}
export function warmContentImages(entries: Array<{icon?: string | null}>) {
    if (typeof window === 'undefined' || settings.value.preloadContentIcons === false) return;
    for (const entry of entries) {
        if (!entry.icon?.startsWith('https://') || downloaded.has(entry.icon) || images.length >= 256) continue;
        if (downloaded.size >= 1500) downloaded.delete(downloaded.values().next().value!);
        downloaded.add(entry.icon); images.push(entry.icon);
    }
    drainImages();
}
export function rememberContentIcons(entries: IconResult[]) {
    const changed: IconResult[] = [];
    for (const entry of entries) if (entry.resolved) {
        const previous = icons.get(entry.iconKey);
        if (!entry.icon && previous?.icon) continue;
        if (entry.icon && entry.icon !== previous?.icon) changed.push(entry);
        remember(entry.iconKey, entry.icon, true);
    }
    if (changed.length) for (const listener of listeners) listener(changed);
    warmContentImages(entries);
}
let bytes = 0;
function remember(key: string, icon: string | null, resolved: boolean) {
    const old = icons.get(key);
    bytes -= old?.icon?.length || 0;
    icons.delete(key);
    icons.set(key, { icon, resolved });
    bytes += icon?.length || 0;
    while (icons.size > 1500 || bytes > 6 * 1024 * 1024) {
        const first = icons.keys().next().value;
        if (!first) break;
        bytes -= icons.get(first)?.icon?.length || 0;
        icons.delete(first);
    }
}
export function hydrateContentIcons(entries: FileTreeEntry[]): FileTreeEntry[] {
    return entries.map(entry => {
        const cached = entry.iconKey ? icons.get(entry.iconKey) : undefined;
        return cached && !entry.icon ? { ...entry, icon: cached.icon, iconResolved: cached.resolved } : entry;
    });
}
export function prefetchInstanceContent(profileId: string, refresh = false): Promise<void> {
    warmContentImages([]);
    if (pending.has(profileId)) return pending.get(profileId)!;
    if (pending.size >= 20) return Promise.resolve();
    if (typeof window === 'undefined' || !profileId || settings.value.preloadContentIcons === false || document.hidden || appState.isLaunching || appState.isGameRunning || !refresh && Date.now() - (warmed.get(profileId) || 0) < 120000) return Promise.resolve();
    const request = prefetchTail.then(async () => {
        const lookups: Array<{ folder: string; names: string[] }> = [];
        for (const folder of ['mods', 'resourcepacks', 'shaderpacks']) {
            if (document.hidden || appState.isLaunching || appState.isGameRunning) return;
            const entries = await instanceFileTree(profileId, folder);
            warmContentImages(entries);
            const missing = entries.filter(entry => entry.iconKey && !icons.has(entry.iconKey) && !entry.icon);
            rememberContentIcons(entries.filter(entry => entry.iconKey && entry.icon).map(entry => ({ iconKey: entry.iconKey!, icon: entry.icon!, resolved: true })));
            const batches: string[][] = [];
            for (let index = 0; index < Math.min(missing.length, 1000); index += 32) batches.push(missing.slice(index, index + 32).map(entry => entry.name));
            await Promise.all(Array.from({ length: 2 }, async () => {
                while (batches.length && !document.hidden && !appState.isLaunching && !appState.isGameRunning) {
                    const names = batches.shift()!;
                    const result = await instanceContentIcons(profileId, folder, names);
                    rememberContentIcons(result);
                    const remote = result.filter(entry => !entry.resolved).map(entry => entry.name);
                    if (remote.length) lookups.push({folder, names: remote});
                }
            }));
        }
        await Promise.all(Array.from({length: 2}, async () => {
            while (lookups.length && !document.hidden && !appState.isLaunching && !appState.isGameRunning) {
                const lookup = lookups.shift()!;
                rememberContentIcons(await instanceContentIcons(profileId, lookup.folder, lookup.names, true));
            }
        }));
        if (lookups.length) return;
        warmed.set(profileId, Date.now());
        if (warmed.size > 20) warmed.delete(warmed.keys().next().value!);
    }).catch(() => {}).finally(() => pending.delete(profileId));
    prefetchTail = request;
    pending.set(profileId, request);
    return request;
}
