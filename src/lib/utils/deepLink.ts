import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { parseJoinAddress } from "./joinAddress";

export type DeepLinkAction =
    | { kind: "install"; content: "mod" | "modpack"; id: string; source: "modrinth" | "curseforge" }
    | { kind: "skin"; url: string; model: "slim" | "classic" }
    | { kind: "server"; address: string }
    | { kind: "friend"; code: string }
    | { kind: "world"; invitation: string }
    | { kind: "lan"; invitation: string };

export function parseDeepLink(value: string): DeepLinkAction {
    if (value.length > 8192) throw new Error("Link muito longo.");
    const url = new URL(value);
    if (url.protocol !== "luxmc:" || url.username || url.password || url.port || url.hash) throw new Error(uiText("ui.2fadb6de07b84f8d"));
    const required = (key: string): string => {
        const values = url.searchParams.getAll(key);
        if (values.length !== 1 || !values[0]) throw new Error(uiText("ui.ee4e1be6a8726710", {arg0: (key)}));
        return values[0];
    };
    if (url.hostname === "install" && ["/mod", "/modpack"].includes(url.pathname)) {
        const source = required("source");
        const id = required("id");
        if (source !== "modrinth" && source !== "curseforge") throw new Error("Provedor desconhecido.");
        if (!/^[a-zA-Z0-9_-]{1,128}$/.test(id) || (source === "curseforge" && !/^\d+$/.test(id))) throw new Error(uiText("ui.6faabb1aa8689fc0"));
        return { kind: "install", content: url.pathname === "/mod" ? "mod" : "modpack", id, source };
    }
    if (url.hostname === "skin" && url.pathname === "/apply") {
        const image = new URL(required("url"));
        const model = required("model");
        if (image.protocol !== "https:" || image.username || image.password || (model !== "slim" && model !== "classic")) throw new Error(uiText("ui.50076fdba0ac8b32"));
        return { kind: "skin", url: image.href, model };
    }
    if (url.hostname === "join") {
        if (url.pathname === "/lan") {
            const invitation = required("invitation");
            if (!/^luxmc-lan:[A-Za-z0-9_-]{1,1000}$/.test(invitation)) throw new Error("Convite de rede inválido.");
            return { kind: "lan", invitation };
        }
        if (url.pathname === "/world") {
            const invitation = required("invitation");
            if (!/^(?:LUX-[0-9A-F]{4}\|)?luxmc-world:[A-Za-z0-9_-]{1,4096}$/.test(invitation)) {
                throw new Error(uiText("ui.7cb3d24f182eb001"));
            }
            return { kind: "world", invitation };
        }
        if (url.pathname === "/friend") {
            const code = required("code");
            if (code.length > 300 || /[\s/?#\\]/.test(code)) throw new Error(uiText("ui.9101c32df3651f3b"));
            return { kind: "friend", code };
        }
        let address: string;
        if (url.pathname === "/server") {
            const host = required("ip");
            const port = url.searchParams.has("port") ? required("port") : "25565";
            if (!/^\d{1,5}$/.test(port) || /[\s/@?#\\]/.test(host)) throw new Error(uiText("ui.40ce9b31937901e4"));
            address = `${host.includes(":") && !host.startsWith("[") ? `[${host}]` : host}:${port}`;
        } else {
            if (url.search) throw new Error(uiText("ui.f64190b69a82e97a"));
            address = decodeURIComponent(url.pathname.slice(1));
        }
        const target = parseJoinAddress(address);
        return { kind: "server", address: `${target.host}:${target.port}` };
    }
    throw new Error(uiText("ui.d162457b1fa49b49"));
}
