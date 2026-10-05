import type { Profile } from "$lib/stores/profiles.svelte";

export function quickProfiles(list: Profile[], query: string): Profile[] {
    const normalized = query.trim().toLocaleLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "");
    const ranked = [...list].sort((a, b) => Number(Boolean(b.favorite)) - Number(Boolean(a.favorite)) || (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0) || a.name.localeCompare(b.name));
    if (!normalized) return ranked.slice(0, 6);
    return ranked.filter(profile => [profile.name, profile.mcVersion, profile.loader].join(" ").toLocaleLowerCase().normalize("NFD").replace(/\p{Diacritic}/gu, "").includes(normalized));
}
