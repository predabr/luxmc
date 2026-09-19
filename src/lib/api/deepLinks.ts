import { api, listen, type UnlistenFn } from "./client";

export async function listenDeepLinks(onLinks: (urls: string[]) => void): Promise<UnlistenFn> {
    let disposed = false;
    const drain = async (): Promise<void> => {
        const urls = await api.invoke<string[]>("deep_links_take");
        if (!disposed && urls.length) onLinks(urls);
    };
    const unsubscribe = await listen("deep-link-pending", () => { void drain().catch(console.error); });
    try { await drain(); } catch (error) { unsubscribe(); throw error; }
    return () => { disposed = true; unsubscribe(); };
}

export async function openPortal(section: "home" | "catalog" | "news" | "skins" = "home"): Promise<void> {
    await api.invoke<void>("open_portal", { section });
}
