import { api } from "./client";

export function fetchMinecraftNews(): Promise<unknown> {
    return api.invoke("minecraft_news");
}
