import { t as i18nT, subscribeLocale, getLocale } from "$lib/i18n";
import { settings } from "$lib/stores/settings.svelte";

let globalLocaleVersion = $state(0);

subscribeLocale(() => {
	globalLocaleVersion++;
});

export function useTranslation() {
	return {
		t(key: string, params?: Record<string, unknown>): string {
			const _ver = globalLocaleVersion;
			const currentLng = settings.value.language || getLocale();
			void _ver;
			return i18nT(key, { lng: currentLng, ...params });
		},
		get currentLanguage(): import("$lib/i18n").Locale {
			void globalLocaleVersion;
			return (settings.value.language as import("$lib/i18n").Locale) || getLocale();
		}
	};
}
