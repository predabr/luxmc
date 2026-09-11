export type CapeType =
	| "none"
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
}

const defaultSkin: SkinData = {
	id: "steve",
	name: "Steve Padrão",
	url: "https://mc-heads.net/body/Steve/300",
	skinUrl: "https://minotar.net/skin/Steve",
	avatarUrl: "https://mc-heads.net/avatar/Steve/100",
	type: "steve",
	hasCape: false,
	capeType: "none"
};

function createSkinStore() {
	let initial = { ...defaultSkin };
	if (typeof window !== "undefined") {
		const saved = localStorage.getItem("luxmc_active_skin_data");
		if (saved) {
			try {
				const parsed = JSON.parse(saved);
				if (parsed.id !== "frog_hoodie") {
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
				localStorage.setItem("luxmc_active_skin_data", JSON.stringify(current));
			}
		},
		setCape(capeType: CapeType) {
			current.capeType = capeType;
			current.hasCape = capeType !== "none";
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_active_skin_data", JSON.stringify(current));
			}
		}
	};
}

export const activeSkinStore = createSkinStore();
