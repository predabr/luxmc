export interface SkinData {
	id: string;
	name: string;
	url: string;
	skinUrl?: string;
	avatarUrl: string;
	type: "steve" | "alex";
	hasCape?: boolean;
	capeType?: "mojang" | "optifine" | "migrator" | "none";
}

const defaultSkin: SkinData = {
	id: "frog_hoodie",
	name: "Frog Hoodie",
	url: "https://mc-heads.net/body/Spect3rBW/300",
	skinUrl: "https://minotar.net/skin/Spect3rBW",
	avatarUrl: "https://mc-heads.net/avatar/Spect3rBW/100",
	type: "alex",
	hasCape: true,
	capeType: "migrator"
};

function createSkinStore() {
	let initial = { ...defaultSkin };
	if (typeof window !== "undefined") {
		const saved = localStorage.getItem("luxmc_active_skin_data");
		if (saved) {
			try {
				initial = { ...defaultSkin, ...JSON.parse(saved) };
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
		setCape(capeType: "mojang" | "optifine" | "migrator" | "none") {
			current.capeType = capeType;
			current.hasCape = capeType !== "none";
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_active_skin_data", JSON.stringify(current));
			}
		}
	};
}

export const activeSkinStore = createSkinStore();
