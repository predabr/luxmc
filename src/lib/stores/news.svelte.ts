import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
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
const refreshInterval = 30 * 60 * 1000;

export const newsState = {
    get items() { return items; },
    get loading() { return loading; },
    get error() { return error; },
    get updatedAt() { return updatedAt; },
    get offline() { return offline; },
    startAutoRefresh(): () => void {
        const refresh = () => {
            if (document.visibilityState === "visible" && navigator.onLine) void this.load();
        };
        const reconnect = () => { if (navigator.onLine) void this.load(offline); };
        refresh();
        const timer = window.setInterval(refresh, 60 * 1000);
        document.addEventListener("visibilitychange", refresh);
        window.addEventListener("focus", refresh);
        window.addEventListener("online", reconnect);
        return () => {
            window.clearInterval(timer);
            document.removeEventListener("visibilitychange", refresh);
            window.removeEventListener("focus", refresh);
            window.removeEventListener("online", reconnect);
        };
    },
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
                    offline = Date.now() - updatedAt >= refreshInterval;
                }
            } catch {}
        }
        if (!force && ((updatedAt > 0 && Date.now() - updatedAt < refreshInterval) || (attemptedAt > 0 && Date.now() - attemptedAt < 5 * 60 * 1000))) return;
        attemptedAt = Date.now();
        loading = true;
        error = "";
        pending = (async () => {
            const feed = await fetchMinecraftNews();
            const parsed = parseMinecraftNews(feed);
            if (!parsed.length) throw new Error(uiText("ui.52ced34f823cf101"));
            items = parsed;
            updatedAt = Date.now();
            offline = false;
            try { localStorage.setItem(cacheKey, JSON.stringify({ feed, fetchedAt: updatedAt })); } catch {}
        })().catch(() => {
            offline = true;
            error = uiText("ui.3f5246e4324ab21b");
        }).finally(() => { loading = false; pending = null; });
        return pending;
    }
};
