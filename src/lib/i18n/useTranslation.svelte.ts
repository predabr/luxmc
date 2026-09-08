import { untrack } from "svelte";
import { t as i18nT, subscribeLocale, getLocale } from "$lib/i18n";

let globalLocaleVersion = $state(0);

subscribeLocale(() => {
	globalLocaleVersion++;
});

export function useTranslation() {
	return {
		t(key: string, params?: Record<string, unknown>): string {
			void globalLocaleVersion;
			return untrack(() => i18nT(key, params));
		}
	};
}
