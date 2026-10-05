import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
export interface MinecraftArticle {
    id: string;
    title: string;
    date: string;
    summary: string;
    image: string;
    link: string;
    category: string;
}

function officialUrl(value: unknown, image = false): string {
    if (typeof value !== "string") return "";
    try {
        const url = new URL(value, "https://launchercontent.mojang.com");
        const allowed = image ? ["launchercontent.mojang.com", "www.minecraft.net"] : ["www.minecraft.net", "minecraft.net", "aka.ms"];
        return url.protocol === "https:" && !url.username && !url.password && allowed.includes(url.hostname) ? url.href : "";
    } catch { return ""; }
}

export function parseMinecraftNews(feed: unknown): MinecraftArticle[] {
    if (!feed || typeof feed !== "object" || !("entries" in feed) || !Array.isArray(feed.entries)) return [];
    const articles = new Map<string, MinecraftArticle>();
    for (const entry of feed.entries) {
        if (!entry || typeof entry !== "object" || typeof entry.title !== "string" || typeof entry.date !== "string") continue;
        const link = officialUrl(entry.readMoreLink);
        if (!link || !/^\d{4}-\d{2}-\d{2}$/.test(entry.date) || !Number.isFinite(Date.parse(entry.date))) continue;
        const id = typeof entry.id === "string" ? entry.id : link;
        articles.set(id, {
            id, title: entry.title.slice(0, 200), date: entry.date,
            summary: typeof entry.text === "string" ? entry.text.slice(0, 1200) : uiText("ui.ecd7d76c453b1c5e"),
            image: officialUrl(entry.newsPageImage?.url || entry.playPageImage?.url, true) || "/news_1.jpg",
            link, category: typeof entry.category === "string" ? entry.category.slice(0, 80) : "Minecraft"
        });
    }
    return [...articles.values()].sort((a, b) => b.date.localeCompare(a.date)).slice(0, 40);
}
