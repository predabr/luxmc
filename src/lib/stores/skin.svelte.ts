import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { authSaveAppearance } from "$lib/api/auth";
import { account, saveCurrentAccount } from "./account.svelte";
import { toast } from "./toasts.svelte";

let pendingSave: Promise<void> = Promise.resolve();
export async function saveAppearance(skinUrl: string, variant: "classic" | "slim", capeUrl: string | null, avatarUrl?: string): Promise<void> {
    const current = account.value;
    if (!current) return Promise.reject(new Error(uiText("ui.bf88bb781c9789c1")));
    const request = pendingSave.catch(() => {}).then(() => authSaveAppearance(current.id, skinUrl, variant, capeUrl));
    pendingSave = request;
    await request;
    if (account.value?.id !== current.id) return;
    const updated = { ...account.value, skinUrl, skinVariant: variant, capeUrl, ...(avatarUrl ? { avatarUrl } : {}) };
    account.value = updated;
    await saveCurrentAccount(updated);
    return request;
}

export type CapeType =
	| "none"
	| "custom"
	| "migrator"
	| "optifine"
	| "mojang"
	| "minecon2011"
	| "minecon2012"
	| "minecon2013"
	| "minecon2015"
	| "minecon2016"
	| "cherry"
	| "vanilla"
	| "tiktok"
	| "twitch"
	| "luxmc";

export interface SkinData {
	id: string;
	name: string;
	url: string;
	skinUrl?: string;
	avatarUrl: string;
	type: "steve" | "alex";
	hasCape?: boolean;
	capeType?: CapeType;
	customCapeUrl?: string;
	custom?: boolean;
}

const defaultSkin: SkinData = {
	id: "steve",
	name: uiText("ui.ef2b3ce09384e183"),
	url: "https://mc-heads.net/body/Steve/300",
	skinUrl: "",
	avatarUrl: "https://mc-heads.net/avatar/Steve/100",
	type: "steve",
	hasCape: false,
	capeType: "none",
	customCapeUrl: "",
	custom: false
};

function createSkinStore() {
	let initial = { ...defaultSkin };
	if (typeof window !== "undefined") {
		const saved = localStorage.getItem("luxmc_active_skin_data");
		if (saved) {
			try {
				const parsed = JSON.parse(saved);
				if (parsed && typeof parsed === "object") {
					initial = { ...defaultSkin, ...parsed };
				}
			} catch {}
		}
	}
	let current = $state<SkinData>(initial);

	return {
		get current() {
			return current;
		},
		setSkin(skin: Partial<SkinData>) {
			current = { ...current, ...skin };
			if (typeof window !== "undefined") {
				try { localStorage.setItem("luxmc_active_skin_data", JSON.stringify(current)); } catch {}
			}
		},
		setCape(capeType: CapeType, customUrl?: string) {
			current.capeType = capeType;
			current.hasCape = capeType !== "none";
			if (customUrl !== undefined) {
				current.customCapeUrl = customUrl;
			}
			if (typeof window !== "undefined") {
				try { localStorage.setItem("luxmc_active_skin_data", JSON.stringify(current)); } catch {}
			}
		}
	};
}

export const activeSkinStore = createSkinStore();
