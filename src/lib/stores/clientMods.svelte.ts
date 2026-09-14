import { browser } from "$app/environment";

export interface ClientModsConfig {
	perspectiveMod: boolean;
	perspectiveKey: string;
	perspectiveSmooth: boolean;

	cosmeticsEnabled: boolean;
	animatedCapes: boolean;
	threeDSkinLayers: boolean;
	wingsEnabled: boolean;

	tntTimer: boolean;
	tntFlashColor: string;
	tntShowTicks: boolean;

	keystrokes: boolean;
	keystrokesShowMouse: boolean;
	keystrokesShowCps: boolean;
	keystrokesPosition: "top-left" | "top-right" | "bottom-left" | "bottom-right";

	customCrosshair: boolean;
	crosshairStyle: "cross" | "dot" | "circle" | "chevron";
	crosshairColor: string;
	crosshairSize: number;

	fpsHud: boolean;
	pingHud: boolean;
	cpsHud: boolean;

	toggleSprint: boolean;
	toggleSneak: boolean;
	armorStatusHud: boolean;
	potionEffectsHud: boolean;

	fullbright: boolean;

	antiCrashGuard: boolean;
	autoTrimMemory: boolean;
	preflightModCheck: boolean;
}

const DEFAULT_CONFIG: ClientModsConfig = {
	perspectiveMod: true,
	perspectiveKey: "v",
	perspectiveSmooth: true,

	cosmeticsEnabled: true,
	animatedCapes: true,
	threeDSkinLayers: true,
	wingsEnabled: false,

	tntTimer: true,
	tntFlashColor: "#ffffff",
	tntShowTicks: false,

	keystrokes: true,
	keystrokesShowMouse: true,
	keystrokesShowCps: true,
	keystrokesPosition: "top-left",

	customCrosshair: false,
	crosshairStyle: "cross",
	crosshairColor: "#ffffff",
	crosshairSize: 8,

	fpsHud: true,
	pingHud: true,
	cpsHud: true,

	toggleSprint: true,
	toggleSneak: false,
	armorStatusHud: true,
	potionEffectsHud: true,

	fullbright: false,

	antiCrashGuard: true,
	autoTrimMemory: true,
	preflightModCheck: true,
};

const STORAGE_KEY = "luxmc_client_mods_config";

class ClientModsStore {
	config = $state<ClientModsConfig>({ ...DEFAULT_CONFIG });
	isMenuOpen = $state<boolean>(false);

	constructor() {
		if (browser) {
			try {
				const saved = localStorage.getItem(STORAGE_KEY);
				if (saved) {
					this.config = { ...DEFAULT_CONFIG, ...JSON.parse(saved) };
				}
			} catch {}
		}
	}

	save() {
		if (browser) {
			try {
				localStorage.setItem(STORAGE_KEY, JSON.stringify(this.config));
			} catch {}
		}
	}

	toggleMenu() {
		this.isMenuOpen = !this.isMenuOpen;
	}

	open() {
		this.isMenuOpen = true;
	}

	close() {
		this.isMenuOpen = false;
	}

	update<K extends keyof ClientModsConfig>(key: K, value: ClientModsConfig[K]) {
		this.config[key] = value;
		this.save();
	}

	reset() {
		this.config = { ...DEFAULT_CONFIG };
		this.save();
	}
}

export const clientMods = new ClientModsStore();
