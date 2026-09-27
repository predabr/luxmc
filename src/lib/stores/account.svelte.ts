import { browser } from "$app/environment";
import { LazyStore } from "@tauri-apps/plugin-store";

export interface Account {
	id: string;
	username: string;
	uuid: string;
	minecraftToken: string;
	expiresAt: number;
	skinUrl?: string | null;
	skinVariant?: string | null;
	capeUrl?: string | null;
	avatarUrl?: string | null;
}

const STORE_FILE = "auth.json";
const STORE_KEY = "current_account";

let store: LazyStore | null = null;
function getStore(): LazyStore {
	if (!store) store = new LazyStore(STORE_FILE);
	return store;
}

export async function saveCurrentAccount(acc: Account | null): Promise<void> {
	if (!browser) return;
	account.value = acc;
	try {
		const s = getStore();
		if (acc) {
			await s.set(STORE_KEY, acc);
		} else {
			await s.delete(STORE_KEY);
		}
		await s.save();
	} catch {
		// Ignore store persistence error
	}
	try {
		localStorage.removeItem("luxmc_current_account");
	} catch {}
}

export async function loadCurrentAccount(): Promise<Account | null> {
	if (!browser) return null;
	try {
		const s = getStore();
		const stored = await s.get<Account>(STORE_KEY);
		if (stored) {
			account.value = stored;
			try {
				localStorage.removeItem("luxmc_current_account");
			} catch {}
			return stored;
		}
	} catch {}

	try {
		const legacy = localStorage.getItem("luxmc_current_account");
		if (legacy) {
			const parsed = JSON.parse(legacy) as Account;
			if (parsed && parsed.id) {
				account.value = parsed;
				await saveCurrentAccount(parsed);
				return parsed;
			}
		}
	} catch {}

	return null;
}

export async function removeCurrentAccount(): Promise<void> {
	await saveCurrentAccount(null);
}

function createAccountStore() {
	let value = $state<Account | null>(null);
	return {
		get value() {
			return value;
		},
		set value(a: Account | null) {
			value = a;
		},
		clear() {
			value = null;
			void removeCurrentAccount();
		}
	};
}

export const account = createAccountStore();

