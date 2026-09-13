import { api } from "./client";
import type { ServerStatus } from "./types";

export interface ServerRow {
	id: string;
	name: string;
	host: string;
	port: number;
	favorite: boolean;
	createdAt: string;
}

export async function serverPing(host: string, port: number): Promise<ServerStatus> {
	return api.invoke<ServerStatus>("server_ping", { host, port });
}

export async function serverAdd(host: string, port: number, name: string): Promise<ServerRow> {
	return api.invoke<ServerRow>("server_add", { host, port, name });
}

export async function serverList(): Promise<ServerRow[]> {
	return api.invoke<ServerRow[]>("server_list");
}

export async function serverRemove(id: string): Promise<void> {
	return api.invoke("server_remove", { id });
}

export async function serverFavoritesList(): Promise<ServerRow[]> {
	return api.invoke<ServerRow[]>("server_favorites_list");
}

export async function serverFavorite(id: string, favorite: boolean): Promise<void> {
	return api.invoke("server_favorite", { id, favorite });
}
