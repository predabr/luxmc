import { parseJoinAddress } from "./joinAddress";

export type DeepLinkAction =
    | { kind: "install"; content: "mod" | "modpack"; id: string; source: "modrinth" | "curseforge" }
    | { kind: "skin"; url: string; model: "slim" | "classic" }
    | { kind: "server"; address: string }
    | { kind: "friend"; code: string };

export function parseDeepLink(value: string): DeepLinkAction {
    if (value.length > 8192) throw new Error("Link muito longo.");
    const url = new URL(value);
    if (url.protocol !== "luxmc:" || url.username || url.password || url.port || url.hash) throw new Error("Link Luxmc inválido.");
    const required = (key: string): string => {
        const values = url.searchParams.getAll(key);
        if (values.length !== 1 || !values[0]) throw new Error(`Parâmetro inválido: ${key}`);
        return values[0];
    };
    if (url.hostname === "install" && ["/mod", "/modpack"].includes(url.pathname)) {
        const source = required("source");
        const id = required("id");
        if (source !== "modrinth" && source !== "curseforge") throw new Error("Provedor desconhecido.");
        if (!/^[a-zA-Z0-9_-]{1,128}$/.test(id) || (source === "curseforge" && !/^\d+$/.test(id))) throw new Error("ID de projeto inválido.");
        return { kind: "install", content: url.pathname === "/mod" ? "mod" : "modpack", id, source };
    }
    if (url.hostname === "skin" && url.pathname === "/apply") {
        const image = new URL(required("url"));
        const model = required("model");
        if (image.protocol !== "https:" || image.username || image.password || (model !== "slim" && model !== "classic")) throw new Error("Skin deve usar HTTPS e modelo slim ou classic.");
        return { kind: "skin", url: image.href, model };
    }
    if (url.hostname === "join") {
        if (url.pathname === "/friend") {
            const code = required("code");
            if (code.length > 300 || /[\s/?#\\]/.test(code)) throw new Error("Código de amigo inválido.");
            return { kind: "friend", code };
        }
        let address: string;
        if (url.pathname === "/server") {
            const host = required("ip");
            const port = url.searchParams.has("port") ? required("port") : "25565";
            if (!/^\d{1,5}$/.test(port) || /[\s/@?#\\]/.test(host)) throw new Error("Servidor inválido.");
            address = `${host.includes(":") && !host.startsWith("[") ? `[${host}]` : host}:${port}`;
        } else {
            if (url.search) throw new Error("Link de servidor inválido.");
            address = decodeURIComponent(url.pathname.slice(1));
        }
        const target = parseJoinAddress(address);
        return { kind: "server", address: `${target.host}:${target.port}` };
    }
    throw new Error("Ação Luxmc desconhecida.");
}
