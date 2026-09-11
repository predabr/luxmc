import { api } from "./client";
import type { ServerStatus } from "./types";

export async function serverPing(host: string, port: number): Promise<ServerStatus> {
	return api.invoke<ServerStatus>("server_ping", { host, port });
}

export async function serverAdd(host: string, port: number, name: string) {
	return api.invoke("server_add", { host, port, name });
}

export async function serverList() {
	return api.invoke("server_list");
}

export async function serverRemove(id: string) {
	return api.invoke("server_remove", { id });
}

export async function serverFavorite(id: string, favorite: boolean) {
	return api.invoke("server_favorite", { id, favorite });
}
