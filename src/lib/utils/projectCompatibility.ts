import type { ModProjectDetails, ModSearchResultItem, ModVersion } from '$lib/api/types';

const knownLoaders = new Set(['fabric', 'forge', 'neoforge', 'quilt', 'legacyfabric', 'babric', 'liteloader', 'rift', 'ornithe', 'risugami', 'modloader', 'vanilla']);

export function loaderName(value: string): string {
    return value.trim().toLowerCase().replace(/[\s_-]/g, '');
}

export function newestGameVersions(values: string[]): string[] {
    return [...new Set(values.filter(Boolean))].sort((a, b) => b.localeCompare(a, undefined, { numeric: true }));
}

export function projectCompatibility(item: ModSearchResultItem, details: ModProjectDetails | null, versions: ModVersion[]) {
    const loaders = [...new Set([
        ...(details?.loaders ?? []),
        ...item.categories.filter(category => knownLoaders.has(loaderName(category))),
        ...versions.flatMap(version => version.loaders ?? [])
    ].map(loaderName).filter(Boolean))].sort();
    const gameVersions = newestGameVersions([...item.versions, ...(details?.gameVersions ?? []), ...versions.flatMap(version => version.gameVersions ?? [])]);
    const rows = loaders.map(loader => ({
        loader,
        gameVersions: newestGameVersions(versions.filter(version => version.loaders?.some(value => loaderName(value) === loader)).flatMap(version => version.gameVersions ?? []))
    }));
    return { loaders, gameVersions, rows };
}
