import { z } from "zod";
import { api } from "./client";

const identitySchema = z.object({ id: z.string(), username: z.string() });
const friendSchema = identitySchema.extend({
	status: z.enum(["online", "in_game", "offline", "pending"]),
	incoming: z.boolean(),
	lastSeen: z.string().nullable(),
	activity: z.string().nullable(),
	mcVersion: z.string().nullable(),
	loader: z.string().nullable(),
	serverIp: z.string().nullable(),
	serverPort: z.number().int().min(1).max(65535).nullable()
});
export type SocialIdentity = z.infer<typeof identitySchema>;
export type Friend = z.infer<typeof friendSchema>;
export interface Presence {
	status: "online" | "in_game" | "offline";
	instanceName?: string;
	mcVersion?: string;
	loader?: string;
	serverHost?: string;
	serverPort?: number;
}

export async function socialRegister(accountId: string, username: string): Promise<SocialIdentity> {
	const result = await api.invoke<unknown>("social_request", { accountId, request: { action: "register", username } });
	return z.object({ me: identitySchema }).parse(result).me;
}

export async function socialSync(accountId: string, presence: Presence): Promise<{ me: SocialIdentity; friends: Friend[] }> {
	const result = await api.invoke<unknown>("social_request", { accountId, request: { action: "sync", ...presence } });
	return z.object({ me: identitySchema, friends: z.array(friendSchema) }).parse(result);
}

export async function socialSearch(accountId: string, query: string): Promise<SocialIdentity[]> {
	const result = await api.invoke<unknown>("social_request", { accountId, request: { action: "search", query } });
	return z.object({ users: z.array(identitySchema) }).parse(result).users;
}

export async function socialFriendAction(accountId: string, action: "invite" | "accept" | "remove" | "block" | "unblock", targetId: string): Promise<void> {
	const result = await api.invoke<unknown>("social_request", { accountId, request: { action, targetId } });
	z.object({ ok: z.literal(true) }).parse(result);
}

export async function socialCreateRoom(accountId: string, host: string, port: number): Promise<{ code: string; expiresAt: number }> {
    return z.object({ code: z.string().regex(/^\d{6}$/), expiresAt: z.number() }).parse(await api.invoke("social_request", { accountId, request: { action: "room_create", host, port } }));
}
export async function socialJoinRoom(accountId: string, code: string): Promise<{ host: string; port: number }> {
    return z.object({ host: z.string(), port: z.number().int().min(1).max(65535) }).parse(await api.invoke("social_request", { accountId, request: { action: "room_join", code } }));
}
export async function socialCloseRoom(accountId: string): Promise<void> {
    await api.invoke("social_request", { accountId, request: { action: "room_close" } });
}

export const friendsSnapshotSchema = z.object({ friends: z.array(friendSchema).max(200) });
export async function socialStreamTicket(accountId: string): Promise<string | null> {
    const result = z.object({ url: z.string().url().nullable() }).parse(await api.invoke("social_request", { accountId, request: { action: "stream_ticket" } }));
    if (result.url && new URL(result.url).protocol !== "wss:") throw new Error("Conexão social insegura");
    return result.url;
}
