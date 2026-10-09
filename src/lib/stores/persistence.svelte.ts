import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { browser } from "$app/environment";
import { LazyStore } from "@tauri-apps/plugin-store";
import { settings, registerSettingsListener, type AppSettings, type ThemeName } from "./settings.svelte";
import { profiles } from "./profiles.svelte";
import { appState } from "./app.svelte";
import { setupI18n, notifyLocaleChange, type Locale } from "$lib/i18n";
import { setActiveLocale } from "$lib/i18n/useTranslation.svelte";
import { detectBrowserLocale, isSupportedLocale, resolveLocale } from "$lib/i18n/locale";
import { appSystemLocale } from "$lib/api/system";
import { settingsGet, settingsSet } from "$lib/api/settings";
import { mergeSavedSettings } from "$lib/utils/settingsPersistence";

import { themeStore } from "./theme.svelte";

const STORE_FILE = "settings.json";
const STORE_KEY = "app";

let store: LazyStore | null = null;
let hydrated = false;

function getStore(): LazyStore {
	if (!store) store = new LazyStore(STORE_FILE);
	return store;
}

export async function bootstrapSettings() {
	if (!browser) return;
	const s = getStore();
	const [native, local] = await Promise.allSettled([settingsGet(), s.get<Partial<AppSettings>>(STORE_KEY)]);
	const stored = mergeSavedSettings<AppSettings>(native.status === "fulfilled" ? native.value : null, local.status === "fulfilled" ? local.value ?? null : null);
	const merged: AppSettings = { ...settings.value, ...stored };
	merged.languageMode = stored.languageMode || (isSupportedLocale(stored.language) ? "manual" : "system");
	merged.language = merged.languageMode === "system" ? await detectSystemLocale() : resolveLocale(stored.language);
	settings.value = merged;
	appState.performanceMode = merged.performanceMode === true;
	setupI18n(merged.language);
	setActiveLocale(merged.language);

	if (merged.accentTheme) {
		themeStore.setAccent(merged.accentTheme, false);
	}
	if (merged.theme) {
		const tId = merged.theme === "default-light" ? "light" : "dark";
		themeStore.setTheme(tId, false);
	}
	if (stored.wallpaperLibrary) themeStore.restoreWallpaperLibrary(stored.wallpaperLibrary);
	if (stored.customWallpaperUrl && merged.customBackground === "custom") {
		themeStore.setCustomWallpaper(stored.customWallpaperUrl, stored.customWallpaperType === "video" ? "video" : "image", undefined, false);
	} else if (themeStore.background === "custom" && themeStore.customWallpaperUrl && stored.customWallpaperUrl === undefined) {
		settings.patch({ customBackground: "custom", customWallpaperUrl: themeStore.customWallpaperUrl, customWallpaperType: themeStore.customWallpaperType });
	} else if (merged.customBackground) {
		themeStore.setBackground(merged.customBackground, false);
	}
	hydrated = true;
	themeStore.importSavedWallpapers();
	schedulePersist();
}

registerSettingsListener(() => schedulePersist());

let lastSaved = "";
let saveChain: Promise<void> = Promise.resolve();

export async function persistNow(): Promise<void> {
	if (!browser || !hydrated) return;
	const run = async () => {
		try {
			const s = getStore();
			const toSave = {
				...settings.value,
				activeProfileId: profiles.activeId || settings.value.activeProfileId
			};
			const currentStr = JSON.stringify(toSave);
			if (currentStr === lastSaved) return;
			const dated = { ...toSave, settingsSavedAt: Date.now() };
			await settingsSet(dated);
			await s.set(STORE_KEY, dated);
			await s.save();
			lastSaved = currentStr;
		} catch (error) {
			console.error(uiText("ui.5534995aef529ef0"), error);
		}
	};
	const next = saveChain.then(run, run);
	saveChain = next;
	await next;
}

let persistTimer: ReturnType<typeof setTimeout> | null = null;
export function schedulePersist() {
	if (!browser || !hydrated) return;
	if (persistTimer) clearTimeout(persistTimer);
	persistTimer = setTimeout(() => {
		persistTimer = null;
		void persistNow();
	}, 500);
}

export async function setTheme(theme: ThemeName) {
	settings.patch({ theme });
	await persistNow();
}

export async function detectSystemLocale(): Promise<Locale> {
	try {
		const language = await appSystemLocale();
		return language ? resolveLocale(language) : detectBrowserLocale();
	} catch { return detectBrowserLocale(); }
}

export async function useSystemLocale() {
	await setLocale(await detectSystemLocale(), "system");
}

export async function setLocale(locale: Locale, languageMode: "manual" | "system" = "manual") {
	settings.patch({ language: locale, languageMode });
	setupI18n(locale);
	setActiveLocale(locale);
	if (browser) {
		localStorage.setItem("luxmc.locale", locale);
	}
	await persistNow();
}

export function startAutoPersist() {
	return () => {};
}
