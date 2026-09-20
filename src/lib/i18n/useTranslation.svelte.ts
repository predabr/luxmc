import { browser } from "$app/environment";
import { settings } from "$lib/stores/settings.svelte";
import en from "./en.json";
import ptBR from "./pt-BR.json";
import es from "./es.json";

const dictionaries: Record<string, any> = {
	en,
	"pt-BR": ptBR,
	es
};

export type Locale = "en" | "pt-BR" | "es";

let activeLocale = $state<Locale>("pt-BR");

if (browser) {
	try {
		const saved = localStorage.getItem("luxmc.locale") as Locale | null;
		if (saved && saved in dictionaries) {
			activeLocale = saved;
		} else {
			const nav = navigator.language.toLowerCase();
			if (nav.startsWith("es")) activeLocale = "es";
			else if (nav.startsWith("pt")) activeLocale = "pt-BR";
			else activeLocale = "en";
		}
	} catch {}
}

export function setActiveLocale(loc: Locale) {
	if (loc in dictionaries) {
		activeLocale = loc;
		if (browser) {
			try {
				localStorage.setItem("luxmc.locale", loc);
			} catch {}
		}
	}
}

export function useTranslation() {
	return {
		t(key: string, params?: Record<string, unknown>): string {
			const loc = activeLocale;
			const dict = dictionaries[loc] || dictionaries["pt-BR"] || dictionaries.en;
			const parts = key.split(".");
			let val: any = dict;
			for (const p of parts) {
				if (val && typeof val === "object" && p in val) {
					val = val[p];
				} else {
					val = undefined;
					break;
				}
			}
			if (typeof val !== "string") {
				let fb: any = dictionaries["pt-BR"];
				for (const p of parts) {
					if (fb && typeof fb === "object" && p in fb) fb = fb[p];
					else { fb = undefined; break; }
				}
				if (typeof fb !== "string") {
					let enFb: any = dictionaries.en;
					for (const p of parts) {
						if (enFb && typeof enFb === "object" && p in enFb) enFb = enFb[p];
						else { enFb = undefined; break; }
					}
					val = typeof enFb === "string" ? enFb : key;
				} else {
					val = fb;
				}
			}
			if (params && typeof val === "string") {
				let res = val;
				for (const [k, v] of Object.entries(params)) {
					res = res.replace(new RegExp(`{{${k}}}`, "g"), String(v));
				}
				return res;
			}
			return typeof val === "string" ? val : key;
		},
		get currentLanguage(): Locale {
			return activeLocale;
		}
	};
}

