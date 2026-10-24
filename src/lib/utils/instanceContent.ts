import type { FileTreeEntry } from '$lib/api/types';

export function instanceContentEntries(entries: FileTreeEntry[], folder: string): FileTreeEntry[] {
    if (folder === 'mods') return entries.filter(entry => !entry.isDir && /\.jar(?:\.disabled)?$/i.test(entry.name));
    if (folder === 'resourcepacks' || folder === 'shaderpacks' || /^saves\/[^/]+\/datapacks$/.test(folder)) {
        return entries.filter(entry => !entry.name.startsWith('.') && (entry.isDir || /\.zip(?:\.disabled)?$/i.test(entry.name)));
    }
    return entries;
}
