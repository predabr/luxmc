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

export async function socialFriendAction(accountId: string, action: "invite" | "accept" | "remove", targetId: string): Promise<void> {
	const result = await api.invoke<unknown>("social_request", { accountId, request: { action, targetId } });
	z.object({ ok: z.literal(true) }).parse(result);
}
