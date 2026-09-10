export type ThemeName = "default-dark" | "default-light";
export type AccentTheme = "indigo" | "blue" | "emerald" | "violet";

export interface AppSettings {
	theme: ThemeName;
	density: "compact" | "comfortable" | "spacious";
	animations: boolean;
	blur: boolean;
	sidebarPosition: "left" | "right";
	language: "en" | "pt-BR";
	accentTheme: AccentTheme;
	activeProfileId: string | null;
	javaPath?: string;
	maxRamMb?: number;
	minRamMb?: number;
	closeOnLaunch?: boolean;
	showCrashLogs?: boolean;
	autoCheckUpdates?: boolean;
	discordRpc?: boolean;
	performanceMode?: boolean;
	customBackground?: string;
	jvmArgs?: string;
	gamemode?: boolean;
	mangohud?: boolean;
	waylandNative?: boolean;
	hugeTlb?: boolean;
	autoBackup?: boolean;
	streamerMode?: boolean;
	sfxVolume?: number;
	customMicrosoftClientId?: string;
}

const defaults: AppSettings = {
	theme: "default-dark",
	density: "comfortable",
	animations: true,
	blur: true,
	sidebarPosition: "left",
	language: "en",
	accentTheme: "indigo",
	activeProfileId: null,
};

function createSettingsStore() {
	let value = $state<AppSettings>({ ...defaults });

	return {
		get value() {
			return value;
		},
		set value(next: AppSettings) {
			value = next;
		},
		patch(p: Partial<AppSettings>) {
			value = { ...value, ...p };
		},
		reset() {
			value = { ...defaults };
		}
	};
}

export const settings = createSettingsStore();
