import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import type { ModSearchResultItem } from "$lib/api/types";

export function catalogCompatibility(
    item: Pick<ModSearchResultItem, "versions" | "categories">,
    profile: { mcVersion: string; loader: string },
    contentType: string,
    checkVersion = true
): string | null {
    if (checkVersion && item.versions.length && !item.versions.includes(profile.mcVersion)) return uiText("ui.05340c02313158ea");
    if (contentType !== "Mod") return null;
    if (profile.loader === "vanilla") return uiText("ui.b8ff192088b83be5");
    const loaders = item.categories.map(category => category.toLowerCase()).filter(category => ["fabric", "forge", "neoforge", "quilt"].includes(category));
    if (loaders.length && !loaders.includes(profile.loader.toLowerCase())) return uiText("ui.b97c43453ff94b4e");
    return null;
}
