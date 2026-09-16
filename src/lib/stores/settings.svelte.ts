export type ThemeName = "default-dark" | "default-light";
export type AccentTheme = "gold" | "cyan" | "emerald" | "rose" | "violet" | "orange" | "blue";

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
	soundEnabled?: boolean;
	liveWallpaper?: boolean;
	soundscapesEnabled?: boolean;
	soundscapeVolume?: number;
	customMicrosoftClientId?: string;
}

const defaults: AppSettings = {
	theme: "default-dark",
	density: "comfortable",
	animations: true,
	blur: true,
	sidebarPosition: "left",
	language: "en",
	accentTheme: "gold",
	activeProfileId: null,
	liveWallpaper: true,
	soundscapesEnabled: false,
	soundscapeVolume: 0.2,
};

let onSettingsChanged: (() => void) | null = null;
export function registerSettingsListener(fn: () => void) {
	onSettingsChanged = fn;
}

function createSettingsStore() {
	let value = $state<AppSettings>({ ...defaults });

	return {
		get value() {
			return value;
		},
		set value(next: AppSettings) {
			value = next;
			if (onSettingsChanged) onSettingsChanged();
		},
		patch(p: Partial<AppSettings>) {
			value = { ...value, ...p };
			if (onSettingsChanged) onSettingsChanged();
		},
		reset() {
			value = { ...defaults };
			if (onSettingsChanged) onSettingsChanged();
		}
	};
}

export const settings = createSettingsStore();
