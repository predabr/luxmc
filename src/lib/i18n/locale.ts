export type Locale = "en" | "pt-BR" | "es";

export function resolveLocale(language: unknown): Locale {
	const primary = typeof language === "string" ? language.trim().replaceAll("_", "-").toLowerCase().split("-")[0] : "";
	return primary === "pt" ? "pt-BR" : primary === "es" ? "es" : "en";
}

export function detectBrowserLocale(): Locale {
	return resolveLocale(typeof navigator === "undefined" ? undefined : navigator.language || navigator.languages?.[0]);
}

export function isSupportedLocale(language: unknown): language is Locale {
	return language === "en" || language === "pt-BR" || language === "es";
}
