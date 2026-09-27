import { fetchMinecraftNews } from "$lib/api/news";
import { parseMinecraftNews, type MinecraftArticle } from "$lib/utils/minecraftNews";
import fallback from "$lib/data/minecraft-news.json";

let items = $state<MinecraftArticle[]>(parseMinecraftNews(fallback));
let loading = $state(false);
let error = $state("");
let updatedAt = $state(0);
let offline = $state(true);
let pending: Promise<void> | null = null;
let initialized = false;
let attemptedAt = 0;
const cacheKey = "luxmc_minecraft_news_v2";

export const newsState = {
    get items() { return items; },
    get loading() { return loading; },
    get error() { return error; },
    get updatedAt() { return updatedAt; },
    get offline() { return offline; },
    async load(force = false): Promise<void> {
        if (typeof window === "undefined") return;
        if (pending) return pending;
        if (!initialized) {
            initialized = true;
            try {
                const cached = JSON.parse(localStorage.getItem(cacheKey) || "null");
                const parsed = parseMinecraftNews(cached?.feed);
                if (parsed.length && Number.isFinite(cached?.fetchedAt) && cached.fetchedAt <= Date.now()) {
                    items = parsed;
                    updatedAt = cached.fetchedAt;
                    offline = Date.now() - updatedAt >= 30 * 60 * 1000;
                }
            } catch {}
        }
        if (!force && ((updatedAt > 0 && Date.now() - updatedAt < 30 * 60 * 1000) || Date.now() - attemptedAt < 30000)) return;
        attemptedAt = Date.now();
        loading = true;
        error = "";
        pending = (async () => {
            const feed = await fetchMinecraftNews();
            const parsed = parseMinecraftNews(feed);
            if (!parsed.length) throw new Error("O feed não retornou notícias válidas");
            items = parsed;
            updatedAt = Date.now();
            offline = false;
            try { localStorage.setItem(cacheKey, JSON.stringify({ feed, fetchedAt: updatedAt })); } catch {}
        })().catch(() => {
            offline = true;
            error = "Sem conexão com as notícias. Exibindo publicações salvas com suas datas originais.";
        }).finally(() => { loading = false; pending = null; });
        return pending;
    }
};
