export type ThemeName = "default-dark" | "default-light";
export type AccentTheme = "gold" | "cyan" | "emerald" | "rose" | "violet" | "orange" | "blue";

export interface AppSettings {
	theme: ThemeName;
	density: "compact" | "comfortable" | "spacious";
	animations: boolean;
	blur: boolean;
	sidebarPosition: "left" | "right";
	language: "en" | "pt-BR" | "es";
	accentTheme: AccentTheme;
	activeProfileId: string | null;
	javaPath?: string;
	maxRamMb?: number;
	minRamMb?: number;
	closeOnLaunch?: boolean;
	showCrashLogs?: boolean;
	autoCheckUpdates?: boolean;
	discordRpc?: boolean;
	hideDiscordDetails?: boolean;
	anonymousTelemetry?: boolean;
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
	launcherActionOnLaunch?: "keep_open" | "hide_reopen" | "close";
	showLogsOnLaunch?: "never" | "on_crash" | "always";
	defaultResWidth?: number;
	defaultResHeight?: number;
	startFullscreen?: boolean;
}

const defaults: AppSettings = {
	theme: "default-dark",
	density: "comfortable",
	animations: true,
	blur: true,
	sidebarPosition: "left",
	language: "en",
	accentTheme: "blue",
	activeProfileId: null,
	liveWallpaper: false,
	soundscapesEnabled: false,
	soundscapeVolume: 0.2,
	launcherActionOnLaunch: "hide_reopen",
	showLogsOnLaunch: "on_crash",
	defaultResWidth: 1920,
	defaultResHeight: 1080,
	startFullscreen: true,
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
