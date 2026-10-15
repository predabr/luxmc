import { instanceContentIcons, instanceFileTree } from '$lib/api/instances';
import type { FileTreeEntry } from '$lib/api/types';
import { settings } from '$lib/stores/settings.svelte';
import { appState } from '$lib/stores/app.svelte';

const icons = new Map<string, { icon: string | null; resolved: boolean }>();
const pending = new Map<string, Promise<void>>();
const warmed = new Map<string, number>();
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
        return cached ? { ...entry, icon: cached.icon, iconResolved: cached.resolved } : entry;
    });
}
export function prefetchInstanceContent(profileId: string, refresh = false): Promise<void> {
    if (pending.has(profileId)) return pending.get(profileId)!;
    if (typeof window === 'undefined' || !profileId || settings.value.preloadContentIcons === false || document.hidden || appState.isLaunching || appState.isGameRunning || !refresh && Date.now() - (warmed.get(profileId) || 0) < 120000) return Promise.resolve();
    const request = (async () => {
        for (const folder of ['mods', 'resourcepacks', 'shaderpacks']) {
            if (document.hidden || appState.isLaunching || appState.isGameRunning) return;
            const entries = await instanceFileTree(profileId, folder);
            const missing = entries.filter(entry => entry.iconKey && !icons.has(entry.iconKey) && !entry.icon);
            for (const entry of entries) if (entry.iconKey && entry.icon) remember(entry.iconKey, entry.icon, true);
            const batches: string[][] = [];
            for (let index = 0; index < Math.min(missing.length, 1000); index += 32) batches.push(missing.slice(index, index + 32).map(entry => entry.name));
            await Promise.all(Array.from({ length: 2 }, async () => {
                while (batches.length && !document.hidden && !appState.isLaunching && !appState.isGameRunning) {
                    const names = batches.shift()!;
                    const result = await instanceContentIcons(profileId, folder, names);
                    for (const entry of result) if (entry.resolved) remember(entry.iconKey, entry.icon, true);
                }
            }));
        }
        warmed.set(profileId, Date.now());
        if (warmed.size > 20) warmed.delete(warmed.keys().next().value!);
    })().catch(() => {}).finally(() => pending.delete(profileId));
    pending.set(profileId, request);
    return request;
}
