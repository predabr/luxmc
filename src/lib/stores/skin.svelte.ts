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
	let current = $state<SkinData>({ ...defaultSkin });

	return {
		get current() {
			return current;
		},
		setSkin(skin: Partial<SkinData>) {
			current = { ...current, ...skin };
		},
		setCape(capeType: "mojang" | "optifine" | "migrator" | "none") {
			current.capeType = capeType;
			current.hasCape = capeType !== "none";
		}
	};
}

export const activeSkinStore = createSkinStore();
