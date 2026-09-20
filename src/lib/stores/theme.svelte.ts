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

export interface BackgroundOption {
	id: string;
	name: string;
	preview: string;
	style: string;
}

export const THEMES: Record<string, ThemeOption> = {
	dark: {
		id: "dark",
		name: "Tema Escuro (Obsidiana)",
		bg: "10 10 12",
		bgElevated: "17 18 22",
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
	gold: { id: "gold", name: "Ouro", hex: "#e2b86b", rgb: "226 184 107", rgbLight: "242 211 156", rgbDark: "180 139 64" },
	rose: { id: "rose", name: "Rosa", hex: "#f43f5e", rgb: "244 63 94", rgbLight: "251 113 133", rgbDark: "225 29 72" },
	violet: { id: "violet", name: "Violeta", hex: "#8b5cf6", rgb: "139 92 246", rgbLight: "167 139 250", rgbDark: "124 58 237" },
	orange: { id: "orange", name: "Laranja", hex: "#f97316", rgb: "249 115 22", rgbLight: "251 146 60", rgbDark: "234 88 12" },
	blue: {
		id: "blue",
		name: "Azul Diamante",
		hex: "#3b82f6",
		rgb: "59 130 246",
		rgbLight: "96 165 250",
		rgbDark: "37 99 235"
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
	purple: {
		id: "purple",
		name: "Ametista",
		hex: "#a855f7",
		rgb: "168 85 247",
		rgbLight: "192 132 252",
		rgbDark: "147 51 234"
	}
};

export const BACKGROUNDS: Record<string, BackgroundOption> = {
	obsidian: {
		id: "obsidian",
		name: "Preto Obsidiana",
		preview: "#0c0c0e",
		style: "background: radial-gradient(circle at 50% -10%, rgba(59, 130, 246, 0.14) 0%, transparent 65%), radial-gradient(circle at 85% 110%, rgba(59, 130, 246, 0.08) 0%, transparent 60%), #090a0f;"
	},
	cosmos: {
		id: "cosmos",
		name: "Cosmos Profundo",
		preview: "#08090f",
		style: "background: radial-gradient(circle at 20% -10%, rgba(59, 130, 246, 0.22) 0%, transparent 55%), radial-gradient(circle at 80% 110%, rgba(168, 85, 247, 0.18) 0%, transparent 55%), #05060b;"
	},
	night: {
		id: "night",
		name: "Minecraft Noite",
		preview: "#0a1128",
		style: "background-image: linear-gradient(rgba(10, 12, 18, 0.82), rgba(10, 12, 18, 0.94)), url('/bg_night.jpg'); background-size: cover; background-position: center;"
	},
	day: {
		id: "day",
		name: "Minecraft Dia",
		preview: "#1e3a5f",
		style: "background-image: linear-gradient(rgba(10, 12, 18, 0.82), rgba(10, 12, 18, 0.94)), url('/bg_day.jpg'); background-size: cover; background-position: center;"
	},
	aurora: {
		id: "aurora",
		name: "Aurora Boreal",
		preview: "#04151f",
		style: "background: radial-gradient(ellipse at 50% -20%, rgba(56, 189, 248, 0.25) 0%, transparent 70%), radial-gradient(ellipse at 80% 80%, rgba(16, 185, 129, 0.2) 0%, transparent 65%), #040810;"
	}
};

let activeTheme = $state("dark");
let activeAccent = $state("blue");
let activeBackground = $state("obsidian");

function applyThemeVariables(tId: string, aId: string, bgId: string) {
	if (typeof document === "undefined") return;
	const root = document.documentElement;

	const t = THEMES[tId] || THEMES.dark;
	const a = ACCENTS[aId] || ACCENTS.blue;

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

    const base = a.rgb.split(" ").map(Number);
    for (const [shade, amount] of [[50, 0.95], [100, 0.88], [200, 0.72], [300, 0.48]] as const) {
        root.style.setProperty(`--brand-${shade}`, base.map(channel => Math.round(channel + (255 - channel) * amount)).join(" "));
    }
    for (const [shade, amount] of [[700, 0.76], [800, 0.6], [900, 0.42], [950, 0.26]] as const) {
        root.style.setProperty(`--brand-${shade}`, base.map(channel => Math.round(channel * amount)).join(" "));
    }
	root.style.setProperty("--brand-500", a.rgb);
	root.style.setProperty("--brand-400", a.rgbLight);
	root.style.setProperty("--brand-600", a.rgbDark);
	root.style.setProperty("--accent-color", a.hex);

	Object.keys(ACCENTS).forEach((k) => root.classList.remove(`accent-${k}`));
	root.classList.add(`accent-${a.id}`);
}

export const themeStore = {
	get theme() { return activeTheme; },
	get accent() { return activeAccent; },
	get background() { return activeBackground; },

	get currentAccentData() {
		return ACCENTS[activeAccent] || ACCENTS.blue;
	},

	get currentBackgroundStyle() {
		if (activeTheme === "light") {
			return "background: radial-gradient(circle at 15% 0%, rgba(59, 130, 246, 0.1) 0%, transparent 50%), radial-gradient(circle at 85% 100%, rgba(59, 130, 246, 0.06) 0%, transparent 50%), #f4f6fa; background-color: #f4f6fa;";
		}
		return BACKGROUNDS[activeBackground]?.style || BACKGROUNDS.obsidian.style;
	},

	setTheme(tId: string, persist = true) {
		let resolved: string | null = null;
		if (tId === "dark" || tId === "default-dark") resolved = "dark";
		else if (tId === "light" || tId === "default-light") resolved = "light";

		if (resolved) {
			activeTheme = resolved;
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_theme", resolved);
			}
			applyThemeVariables(activeTheme, activeAccent, activeBackground);
			if (persist) {
				import("./settings.svelte").then(({ settings }) => {
					settings.patch({ theme: resolved === "light" ? "default-light" : "default-dark" });
				}).catch(() => {});
				import("./persistence.svelte").then(({ schedulePersist }) => {
					schedulePersist();
				}).catch(() => {});
			}
		}
	},

	setAccent(aId: string, persist = true) {
		if (ACCENTS[aId]) {
			activeAccent = aId;
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_accent", aId);
			}
			applyThemeVariables(activeTheme, activeAccent, activeBackground);
			if (persist) {
				import("./settings.svelte").then(({ settings }) => {
					settings.patch({ accentTheme: aId as import("./settings.svelte").AccentTheme });
				}).catch(() => {});
				import("./persistence.svelte").then(({ schedulePersist }) => {
					schedulePersist();
				}).catch(() => {});
			}
		}
	},

	setBackground(bgId: string, persist = true) {
		if (BACKGROUNDS[bgId]) {
			activeBackground = bgId;
			if (typeof window !== "undefined") {
				localStorage.setItem("luxmc_background", bgId);
			}
			applyThemeVariables(activeTheme, activeAccent, activeBackground);
			if (persist) {
				import("./settings.svelte").then(({ settings }) => {
					settings.patch({ customBackground: bgId });
				}).catch(() => {});
				import("./persistence.svelte").then(({ schedulePersist }) => {
					schedulePersist();
				}).catch(() => {});
			}
		}
	},

	init() {
		if (typeof window === "undefined") return;
		const savedTheme = localStorage.getItem("luxmc_theme");
		const savedAccent = localStorage.getItem("luxmc_accent");
		const savedBg = localStorage.getItem("luxmc_background");
		if (savedTheme) {
			if (savedTheme === "light" || savedTheme === "default-light") activeTheme = "light";
			else if (savedTheme === "dark" || savedTheme === "default-dark") activeTheme = "dark";
		}
		if (savedAccent && ACCENTS[savedAccent]) activeAccent = savedAccent;
		if (savedBg && BACKGROUNDS[savedBg]) activeBackground = savedBg;
		applyThemeVariables(activeTheme, activeAccent, activeBackground);
	}
};
