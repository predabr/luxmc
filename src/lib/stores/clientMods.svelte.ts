import { browser } from "$app/environment";

export interface ClientModsConfig {
	// Combate & HUD
	armorHud: boolean;
	bossbar: boolean;
	comboDisplay: boolean;
	coordinates: boolean;
	cps: boolean;
	directionHud: boolean;
	fps: boolean;
	keystrokes: boolean;
	memory: boolean;
	packOverlay: boolean;
	pingDisplay: boolean;
	playTime: boolean;
	potionCounter: boolean;
	potionEffects: boolean;
	reachDisplay: boolean;
	scoreboard: boolean;
	serverAddress: boolean;
	speedometer: boolean;
	timeDisplay: boolean;
	toggleSprint: boolean;

	// Visual & Animações
	twoDItems: boolean;
	threeDSkinLayers: boolean;
	animations: boolean;
	autoFriend: boolean;
	autoGG: boolean;
	autoText: boolean;
	blockOverlay: boolean;
	chat: boolean;
	cosmetics: boolean;
	crosshair: boolean;
	damageTint: boolean;
	debugScreen: boolean;
	discordRP: boolean;
	emotes: boolean;
	fullbright: boolean;
	glintColorizer: boolean;
	hitbox: boolean;
	hitColor: boolean;
	inputFix: boolean;
	itemPhysics: boolean;
	motionBlur: boolean;
	nametags: boolean;
	oldAnimations: boolean;
	particles: boolean;
	perspective: boolean;
	shinyPots: boolean;
	skins: boolean;
	tab: boolean;
	timeChanger: boolean;
	tntTimer: boolean;
	waveyCapes: boolean;
	waypoints: boolean;
	weatherChanger: boolean;
	zoom: boolean;

	// Otimização & Proteção
	antiCrashGuard: boolean;
	autoTrimMemory: boolean;
	preflightModCheck: boolean;

	[key: string]: boolean;
}

const DEFAULT_CONFIG: ClientModsConfig = {
	armorHud: true,
	bossbar: true,
	comboDisplay: true,
	coordinates: true,
	cps: true,
	directionHud: true,
	fps: true,
	keystrokes: true,
	memory: true,
	packOverlay: false,
	pingDisplay: true,
	playTime: true,
	potionCounter: true,
	potionEffects: true,
	reachDisplay: true,
	scoreboard: true,
	serverAddress: true,
	speedometer: false,
	timeDisplay: true,
	toggleSprint: true,

	twoDItems: false,
	threeDSkinLayers: true,
	animations: true,
	autoFriend: false,
	autoGG: true,
	autoText: false,
	blockOverlay: true,
	chat: true,
	cosmetics: true,
	crosshair: true,
	damageTint: true,
	debugScreen: false,
	discordRP: true,
	emotes: true,
	fullbright: true,
	glintColorizer: false,
	hitbox: false,
	hitColor: false,
	inputFix: true,
	itemPhysics: true,
	motionBlur: false,
	nametags: true,
	oldAnimations: true,
	particles: true,
	perspective: true,
	shinyPots: true,
	skins: true,
	tab: true,
	timeChanger: false,
	tntTimer: true,
	waveyCapes: true,
	waypoints: true,
	weatherChanger: false,
	zoom: true,

	antiCrashGuard: true,
	autoTrimMemory: true,
	preflightModCheck: true,
};

export interface ArmorHudSettings {
	orientation: "vertical" | "horizontal";
	durabilityMode: "percent" | "numeric" | "bar" | "none";
	warningLowDurability: boolean;
	showMainHand: boolean;
	showOffHand: boolean;
	showItemCount: boolean;
	scale: number;
	position: "bottom-right" | "bottom-left" | "top-right" | "top-left";
}

export interface KeystrokesSettings {
	showCps: boolean;
	showSpace: boolean;
	showMouse: boolean;
	colorMode: "chroma" | "white" | "emerald" | "cyan";
	opacity: number;
}

export interface CpsSettings {
	showRight: boolean;
	colorMode: "chroma" | "white" | "emerald";
	suffix: string;
}

export interface ToggleSprintSettings {
	text: string;
	style: "text" | "icon";
	color: string;
}

export interface DirectionHudSettings {
	style: "bar" | "compact";
	showBiome: boolean;
}

export interface PotionEffectsSettings {
	showDuration: boolean;
	blinkOnExpire: boolean;
	compactMode: boolean;
}

export interface ClientModuleSettings {
	armorHud: ArmorHudSettings;
	keystrokes: KeystrokesSettings;
	cps: CpsSettings;
	toggleSprint: ToggleSprintSettings;
	directionHud: DirectionHudSettings;
	potionEffects: PotionEffectsSettings;
}

export const DEFAULT_MODULE_SETTINGS: ClientModuleSettings = {
	armorHud: {
		orientation: "vertical",
		durabilityMode: "percent",
		warningLowDurability: true,
		showMainHand: true,
		showOffHand: true,
		showItemCount: true,
		scale: 1.0,
		position: "bottom-right"
	},
	keystrokes: {
		showCps: true,
		showSpace: true,
		showMouse: true,
		colorMode: "emerald",
		opacity: 0.75
	},
	cps: {
		showRight: true,
		colorMode: "emerald",
		suffix: "CPS"
	},
	toggleSprint: {
		text: "[Correndo (Ativo)]",
		style: "text",
		color: "#34d399"
	},
	directionHud: {
		style: "bar",
		showBiome: true
	},
	potionEffects: {
		showDuration: true,
		blinkOnExpire: true,
		compactMode: false
	}
};

const STORAGE_KEY = "luxmc_client_mods_config";
const MODULE_SETTINGS_KEY = "luxmc_client_module_settings";

class ClientModsStore {
	config = $state<ClientModsConfig>({ ...DEFAULT_CONFIG });
	moduleSettings = $state<ClientModuleSettings>({ ...DEFAULT_MODULE_SETTINGS });
	isMenuOpen = $state<boolean>(false);
	selectedModuleForConfig = $state<string | null>(null);
	isPreviewingHud = $state<boolean>(false);

	constructor() {
		if (browser) {
			try {
				const saved = localStorage.getItem(STORAGE_KEY);
				if (saved) {
					this.config = { ...DEFAULT_CONFIG, ...JSON.parse(saved) };
				}
				const savedModSettings = localStorage.getItem(MODULE_SETTINGS_KEY);
				if (savedModSettings) {
					this.moduleSettings = { ...DEFAULT_MODULE_SETTINGS, ...JSON.parse(savedModSettings) };
				}
			} catch {}
		}
	}

	save() {
		if (browser) {
			try {
				localStorage.setItem(STORAGE_KEY, JSON.stringify(this.config));
				localStorage.setItem(MODULE_SETTINGS_KEY, JSON.stringify(this.moduleSettings));
			} catch {}
		}
	}

	toggleMenu() {
		this.isMenuOpen = !this.isMenuOpen;
		if (!this.isMenuOpen) {
			this.selectedModuleForConfig = null;
			this.isPreviewingHud = false;
		}
	}

	open() {
		this.isMenuOpen = true;
	}

	close() {
		this.isMenuOpen = false;
		this.selectedModuleForConfig = null;
		this.isPreviewingHud = false;
	}

	toggle(key: keyof ClientModsConfig) {
		if (typeof this.config[key] === "boolean") {
			this.config[key] = !this.config[key];
			this.save();
		}
	}

	update<K extends keyof ClientModsConfig>(key: K, value: ClientModsConfig[K]) {
		this.config[key] = value;
		this.save();
	}

	updateModuleSetting<M extends keyof ClientModuleSettings, K extends keyof ClientModuleSettings[M]>(
		moduleKey: M,
		settingKey: K,
		val: ClientModuleSettings[M][K]
	) {
		this.moduleSettings[moduleKey][settingKey] = val;
		this.save();
	}

	reset() {
		this.config = { ...DEFAULT_CONFIG };
		this.moduleSettings = { ...DEFAULT_MODULE_SETTINGS };
		this.selectedModuleForConfig = null;
		this.isPreviewingHud = false;
		this.save();
	}
}

export const clientMods = new ClientModsStore();
