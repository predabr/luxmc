import { browser } from "$app/environment";
import { LazyStore } from "@tauri-apps/plugin-store";
import { settings, type AppSettings, type ThemeName } from "./settings.svelte";
import { profiles } from "./profiles.svelte";
import { setupI18n, notifyLocaleChange, type Locale } from "$lib/i18n";

const STORE_FILE = "settings.json";
const STORE_KEY = "app";

let store: LazyStore | null = null;

function getStore(): LazyStore {
	if (!store) store = new LazyStore(STORE_FILE);
	return store;
}

export async function bootstrapSettings() {
	if (!browser) return;
	const s = getStore();
	const stored = (await s.get<Partial<AppSettings>>(STORE_KEY)) ?? {};
	const merged: AppSettings = { ...settings.value, ...stored };
	settings.value = merged;
	setupI18n(merged.language);
}

export async function persistNow() {
	if (!browser) return;
	const s = getStore();
	settings.value.activeProfileId = profiles.activeId;
	await s.set(STORE_KEY, settings.value);
	await s.save();
}

let persistTimer: ReturnType<typeof setTimeout> | null = null;
export function schedulePersist() {
	if (!browser) return;
	if (persistTimer) clearTimeout(persistTimer);
	persistTimer = setTimeout(() => {
		void persistNow();
	}, 200);
}

export async function setTheme(theme: ThemeName) {
	settings.patch({ theme });
	await persistNow();
}

export async function setLocale(locale: Locale) {
	settings.patch({ language: locale });
	setupI18n(locale);
	if (browser) {
		localStorage.setItem("luxmc.locale", locale);
	}
	await persistNow();
}

export function startAutoPersist() {
	if (!browser) return () => {};
	let last = JSON.stringify(settings.value);
	const id = setInterval(() => {
		const cur = JSON.stringify(settings.value);
		if (cur !== last) {
			last = cur;
			void persistNow();
		}
	}, 800);
	return () => clearInterval(id);
}
