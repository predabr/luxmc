export interface ThemeOption {
	id: string;
	name: string;
	bg: string;
	bgElevated: string;
	border: string;
	previewColor: string;
}

export interface AccentOption {
	id: string;
	name: string;
	hex: string;
	rgb: string;
	rgbLight: string;
	rgbDark: string;
}

export const THEMES: Record<string, ThemeOption> = {
	dark: {
		id: "dark",
		name: "Tema Escuro (Preto Profundo)",
		bg: "10 10 12",
		bgElevated: "18 18 22",
		border: "32 32 38",
		previewColor: "#0a0a0c"
	},
	light: {
		id: "light",
		name: "Tema Claro (Branco Neve)",
		bg: "245 247 250",
		bgElevated: "255 255 255",
		border: "218 225 238",
		previewColor: "#ffffff"
	}
};

export const ACCENTS: Record<string, AccentOption> = {
	gold: {
		id: "gold",
		name: "Dourado Luxmc",
		hex: "#e2b86b",
		rgb: "226 184 107",
		rgbLight: "240 205 140",
		rgbDark: "195 152 75"
	},
	cyan: {
		id: "cyan",
		name: "Ciano Neon",
		hex: "#06b6d4",
		rgb: "6 182 212",
		rgbLight: "34 211 238",
		rgbDark: "8 145 178"
	},
	emerald: {
		id: "emerald",
		name: "Esmeralda",
		hex: "#10b981",
		rgb: "16 185 129",
		rgbLight: "52 211 153",
		rgbDark: "5 150 105"
	},
	rose: {
		id: "rose",
		name: "Rosa Neon",
		hex: "#f43f5e",
		rgb: "244 63 94",
		rgbLight: "251 113 133",
		rgbDark: "225 29 72"
	},
	violet: {
		id: "violet",
		name: "Violeta Elétrico",
		hex: "#8b5cf6",
		rgb: "139 92 246",
		rgbLight: "167 139 250",
		rgbDark: "124 58 237"
	},
	orange: {
		id: "orange",
		name: "Lava Flame",
		hex: "#f97316",
		rgb: "249 115 22",
		rgbLight: "251 146 60",
		rgbDark: "234 88 12"
	},
	blue: {
		id: "blue",
		name: "Azul Diamante",
		hex: "#3b82f6",
		rgb: "59 130 246",
		rgbLight: "96 165 250",
		rgbDark: "37 99 235"
	}
};

let activeTheme = $state("dark");
let activeAccent = $state("gold");

function applyThemeVariables(tId: string, aId: string) {
	if (typeof document === "undefined") return;
	const root = document.documentElement;

	const t = THEMES[tId] || THEMES.dark;
	const a = ACCENTS[aId] || ACCENTS.gold;

	root.style.setProperty("--bg", t.bg);
	root.style.setProperty("--bg-elevated", t.bgElevated);
	root.style.setProperty("--border", t.border);

	if (tId === "light") {
		root.style.setProperty("--bg-subtle", "238 242 248");
		root.style.setProperty("--bg-overlay", "220 226 236");
		root.style.setProperty("--fg", "15 23 42");
		root.style.setProperty("--fg-muted", "71 85 105");
		root.style.setProperty("--fg-subtle", "100 116 139");
		root.classList.add("light");
		root.classList.remove("dark");
		root.style.colorScheme = "light";
	} else {
		root.style.setProperty("--bg-subtle", "22 25 33");
		root.style.setProperty("--bg-overlay", "8 9 14");
		root.style.setProperty("--fg", "230 235 245");
		root.style.setProperty("--fg-muted", "145 155 180");
		root.style.setProperty("--fg-subtle", "95 105 130");
		root.classList.add("dark");
		root.classList.remove("light");
		root.style.colorScheme = "dark";
	}

	root.style.setProperty("--brand-500", a.rgb);
	root.style.setProperty("--brand-400", a.rgbLight);
	root.style.setProperty("--brand-600", a.rgbDark);
	root.style.setProperty("--accent-color", a.hex);

	// Remove all accent classes and add active accent
	Object.keys(ACCENTS).forEach((k) => root.classList.remove(`accent-${k}`));
	root.classList.add(`accent-${a.id}`);
}

export const themeStore = {
	get theme() { return activeTheme; },
	get accent() { return activeAccent; },

	get currentAccentData() {
		return ACCENTS[activeAccent] || ACCENTS.gold;
	},

	setTheme(tId: string) {
		if (THEMES[tId]) {
			activeTheme = tId;
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_theme", tId);
			}
			applyThemeVariables(activeTheme, activeAccent);
		}
	},

	setAccent(aId: string) {
		if (ACCENTS[aId]) {
			activeAccent = aId;
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_accent", aId);
			}
			applyThemeVariables(activeTheme, activeAccent);
		}
	},

	init() {
		if (typeof window === "undefined") return;
		const savedTheme = localStorage.getItem("luxmc_theme");
		const savedAccent = localStorage.getItem("luxmc_accent");
		if (savedTheme && THEMES[savedTheme]) activeTheme = savedTheme;
		if (savedAccent && ACCENTS[savedAccent]) activeAccent = savedAccent;
		applyThemeVariables(activeTheme, activeAccent);
	}
};
