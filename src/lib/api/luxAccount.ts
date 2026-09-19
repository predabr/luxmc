import { z } from "zod";
import { api } from "./client";
import type { AuthAccount } from "./auth";

const preferencesSchema = z.object({
    theme: z.enum(["default-dark", "default-light"]).nullish(),
    accentTheme: z.enum(["gold", "cyan", "emerald", "rose", "violet", "orange", "blue"]).nullish(),
    language: z.enum(["pt-BR", "en", "es"]).nullish(),
    animations: z.boolean().nullish()
});
const portalSchema = z.object({ id: z.string().uuid(), username: z.string(), socialId: z.string().uuid(), preferences: preferencesSchema, revision: z.number().int() });
export type CloudPreferences = { theme?: "default-dark" | "default-light"; accentTheme?: "gold" | "cyan" | "emerald" | "rose" | "violet" | "orange" | "blue"; language?: "pt-BR" | "en" | "es"; animations?: boolean };
export type PortalAccount = z.infer<typeof portalSchema>;
export function luxAccountLogin(username: string, password: string): Promise<AuthAccount> {
    return api.invoke<AuthAccount>("lux_account_login", { username, password });
}
export async function luxAccountSync(accountId: string, preferences?: CloudPreferences, revision?: number): Promise<PortalAccount> {
    return portalSchema.parse(await api.invoke<unknown>("lux_account_sync", { accountId, preferences, revision }));
}
export function luxAccountLogout(accountId: string): Promise<void> { return api.invoke<void>("lux_account_logout", { accountId }); }
