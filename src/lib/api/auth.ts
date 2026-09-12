import { api } from "./client";

export async function authBegin(): Promise<{ state: string; verifier: string; url: string }> {
	return api.invoke("auth_begin");
}

export async function authLogin(): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
	skinUrl?: string;
	skinVariant?: string;
	capeUrl?: string;
}> {
	return api.invoke("auth_login");
}

export async function authComplete(code: string, state: string, verifier: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
	skinUrl?: string;
	skinVariant?: string;
	capeUrl?: string;
}> {
	return api.invoke("auth_complete", { code, stateToken: state, verifier });
}

export async function authRefresh(refreshToken: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
	skinUrl?: string;
	skinVariant?: string;
	capeUrl?: string;
}> {
	return api.invoke("auth_refresh", { refreshToken });
}

export async function authAccounts(): Promise<Array<{
	id: string;
	username: string;
	uuid: string;
	refreshToken: string;
	accessToken: string | null;
	expiresAt: string | null;
	createdAt: string;
	updatedAt: string;
}>> {
	return api.invoke("auth_accounts");
}

export async function authRemove(uuid: string): Promise<void> {
	return api.invoke("auth_remove", { uuid });
}

export async function authDevLogin(): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_dev_login");
}

export async function authOfflineLogin(username: string): Promise<{
	id: string;
	username: string;
	uuid: string;
	accessToken: string;
	refreshToken: string;
	expiresAt: number;
}> {
	return api.invoke("auth_offline_login", { username });
}

export async function authSwitchAccount(uuid: string): Promise<void> {
	return api.invoke("auth_switch_account", { uuid });
}

export async function authGetClientId(): Promise<string> {
	return api.invoke("auth_get_client_id");
}

export async function authGetTenantId(): Promise<string> {
	return api.invoke("auth_get_tenant_id");
}

export async function authSetClientId(clientId: string): Promise<void> {
	return api.invoke("auth_set_client_id", { clientId });
}

export async function authChangeSkin(uuid: string, variant: string, skinUrl: string): Promise<void> {
	return api.invoke("auth_change_skin", { uuid, variant, skinUrl });
}
