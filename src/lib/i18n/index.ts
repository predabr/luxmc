import { browser } from "$app/environment";
import i18next, { type i18n as I18nInstance } from "i18next";
import en from "./en.json";
import ptBR from "./pt-BR.json";

const resources = {
	en: { translation: en },
	"pt-BR": { translation: ptBR }
} as const;

export type Locale = keyof typeof resources;

export const LOCALES: Locale[] = ["en", "pt-BR"];

let _locale: "en" | "pt-BR" = "en";

let localeListeners: Array<() => void> = [];

export function notifyLocaleChange() {
	for (const fn of localeListeners) fn();
}

export function subscribeLocale(fn: () => void) {
	localeListeners = [...localeListeners, fn];
	return () => {
		localeListeners = localeListeners.filter((l) => l !== fn);
	};
}

export function detectInitialLocale(): Locale {
	if (!browser) return "en";
	const stored = localStorage.getItem("luxmc.locale") as Locale | null;
	if (stored && stored in resources) return stored;
	const nav = browser ? navigator.language : "en";
	if (nav.toLowerCase().startsWith("pt")) return "pt-BR";
	return "en";
}

let initialised = false;

export function setupI18n(initial: Locale = detectInitialLocale()): I18nInstance {
	_locale = initial;
	if (initialised) {
		void i18next.changeLanguage(initial).then(() => {
			notifyLocaleChange();
		});
		return i18next;
	}
	initialised = true;
	void i18next.init({
		resources,
		lng: initial,
		fallbackLng: "en",
		interpolation: { escapeValue: false }
	}).then(() => {
		notifyLocaleChange();
	});
	return i18next;
}

export function getLocale(): "en" | "pt-BR" {
	return _locale;
}

export function t(key: string, params?: Record<string, unknown>): string {
	return i18next.t(key, params);
}

export function getI18n(): I18nInstance {
	return i18next;
}
