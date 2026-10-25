import type { PackArtwork } from '$lib/api/social';

type LibraryEntry = {name: string; icon: string};
export function favouriteKey(name: string): string {
    return name.normalize('NFKC').trim().toLowerCase().replace(/^the\s+/, '').replace(/\s+modpack$/, '').replace(/\s+/g, ' ');
}
export function safeFavouriteIcon(value: string): boolean {
    try {
        const url = new URL(value);
        return url.protocol === 'https:' && !url.username && !url.password && !url.port && ['cdn.modrinth.com','media.forgecdn.net','mediafilez.forgecdn.net','edge.forgecdn.net'].includes(url.hostname);
    } catch { return false; }
}
export function favouriteArtwork(name: string, saved: PackArtwork[], library: LibraryEntry[] = []): PackArtwork {
    const stored = saved.find(item => item.name === name && safeFavouriteIcon(item.icon));
    if (stored) return stored;
    const instance = library.find(item => item.name === name) || library.find(item => favouriteKey(item.name) === favouriteKey(name));
    return {name, title:instance?.name || name, icon:instance && (safeFavouriteIcon(instance.icon) || /^\/[a-zA-Z0-9_./-]+$/.test(instance.icon) && !instance.icon.includes('..') && !instance.icon.startsWith('//')) ? instance.icon : '/grass_block.png'};
}
export function distinctFavourites(names: string[], saved: PackArtwork[], library: LibraryEntry[] = []): PackArtwork[] {
    const seen = new Set<string>();
    return names.map(name => favouriteArtwork(name, saved, library)).filter(item => {
        const key = favouriteKey(item.title);
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
    });
}
