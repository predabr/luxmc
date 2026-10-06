import { detectBrowserLocale } from "$lib/i18n/locale";

export type ThemeName = "default-dark" | "default-light";
export type AccentTheme = "gold" | "cyan" | "emerald" | "rose" | "violet" | "orange" | "blue";

export interface AppSettings {
	theme: ThemeName;
	density: "compact" | "comfortable" | "spacious";
	animations: boolean;
	blur: boolean;
	sidebarPosition: "left" | "right";
	language: "en" | "pt-BR" | "es";
	languageMode?: "system" | "manual";
	accentTheme: AccentTheme;
	activeProfileId: string | null;
	javaPath?: string;
    rightSidebarWidth?: 280 | 320 | 360;
    pauseSkinWhileGaming?: boolean;
    shareCustomAvatar?: boolean;
    publishGameActivity?: boolean;
	maxRamMb?: number;
	minRamMb?: number;
	closeOnLaunch?: boolean;
	showCrashLogs?: boolean;
	autoCheckUpdates?: boolean;
	discordRpc?: boolean;
    discordClientId?: string;
	hideDiscordDetails?: boolean;
	anonymousTelemetry?: boolean;
	performanceMode?: boolean;
	wallpaperFps?: 15 | 24 | 30 | 60;
	wallpaperWidth?: 960 | 1280 | 1920;
	pauseWallpaperOnBlur?: boolean;
	animatedWallpaperBlur?: boolean;
    interfaceOpacity?: number;
    wallpaperDim?: number;
    capePhysics?: boolean;
    capeWindStrength?: number;
	customBackground?: string;
	customWallpaperUrl?: string;
	customWallpaperType?: "image" | "video";
	wallpaperLibrary?: Array<{ url: string; type: "image" | "video"; name: string }>;
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
	releaseChannel?: "stable" | "beta";
	concurrentDownloads?: number;
	closeWarningOnGameRunning?: boolean;
	showLogsOnLaunch?: "never" | "on_crash" | "always";
	defaultResWidth?: number;
	defaultResHeight?: number;
	startFullscreen?: boolean;
}

const defaults: AppSettings = {
	theme: "default-dark",
	density: "comfortable",
	animations: true,
	blur: false,
	sidebarPosition: "left",
	language: detectBrowserLocale(),
	languageMode: "system",
	accentTheme: "blue",
    soundEnabled: true,
    sfxVolume: 0.5,
	activeProfileId: null,
	pauseWallpaperOnBlur: true,
	autoCheckUpdates: true,
	liveWallpaper: false,
	soundscapesEnabled: false,
	soundscapeVolume: 0.2,
	launcherActionOnLaunch: "hide_reopen",
	releaseChannel: "stable",
	concurrentDownloads: 24,
	closeWarningOnGameRunning: true,
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

