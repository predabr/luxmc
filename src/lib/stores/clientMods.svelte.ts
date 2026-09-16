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

	[key: string]: any;
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

	toggle(key: keyof ClientModsConfig) {
		if (typeof this.config[key] === "boolean") {
			this.config[key] = !this.config[key] as any;
			this.save();
		}
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
