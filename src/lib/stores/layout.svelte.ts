export type SectionId = "hero" | "quickInstances" | "favoriteServer" | "curatedPacks" | "gamingStats" | "newsFeed";
export type LayoutPreset = "gamer" | "compact" | "full";

export interface LayoutSectionItem {
	id: SectionId;
	title: string;
	description: string;
	enabled: boolean;
	icon: string;
	width: "full" | "half";
}

const STORAGE_KEY = "luxmc_dashboard_layout_v1";

const DEFAULT_SECTIONS: LayoutSectionItem[] = [
	{
		id: "hero",
		title: "Destaque & Inicialização",
		description: "Banner principal com status da instância ativa e botão de início rápido",
		enabled: true,
		icon: "play",
		width: "full"
	},
	{
		id: "quickInstances",
		title: "Acesso Rápido a Instâncias",
		description: "Carrossel com suas instâncias instaladas para troca rápida",
		enabled: true,
		icon: "boxes",
		width: "full"
	},
	{
		id: "curatedPacks",
		title: "Modpacks Recomendados",
		description: "Coleções em destaque prontas para baixar e jogar",
		enabled: true,
		icon: "package",
		width: "full"
	},
	{
		id: "favoriteServer",
		title: "Servidor Favorito & Ping",
		description: "Monitor em tempo real do seu servidor com ping, jogadores e entrada rápida",
		enabled: true,
		icon: "signal",
		width: "half"
	},
	{
		id: "gamingStats",
		title: "Tempo de Jogo & Monitor",
		description: "Estatísticas em tempo real do seu tempo de jogo semanal",
		enabled: true,
		icon: "clock",
		width: "half"
	},
	{
		id: "newsFeed",
		title: "Notícias & Patch Notes",
		description: "Últimas atualizações e release notes do Luxmc Launcher",
		enabled: true,
		icon: "newspaper",
		width: "full"
	}
];

function loadSavedSections(): LayoutSectionItem[] {
	if (typeof window === "undefined") return DEFAULT_SECTIONS;
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return DEFAULT_SECTIONS;
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return DEFAULT_SECTIONS;
		
		const validIds = new Set(DEFAULT_SECTIONS.map((s) => s.id));
		const merged: LayoutSectionItem[] = [];
		for (const item of parsed) {
			if (item && validIds.has(item.id)) {
				const def = DEFAULT_SECTIONS.find((s) => s.id === item.id);
				if (def) {
					merged.push({
						...def,
						enabled: typeof item.enabled === "boolean" ? item.enabled : def.enabled,
						width: item.width === "half" ? "half" : "full"
					});
					validIds.delete(item.id);
				}
			}
		}
		for (const id of validIds) {
			const def = DEFAULT_SECTIONS.find((s) => s.id === id);
			if (def) merged.push(def);
		}
		return merged;
	} catch {
		return DEFAULT_SECTIONS;
	}
}

class LayoutStore {
	sections = $state<LayoutSectionItem[]>(loadSavedSections());
	isCompactMode = $state<boolean>(false);

	constructor() {
		if (typeof window !== "undefined") {
			this.sections = loadSavedSections();
		}
	}

	persist() {
		if (typeof window === "undefined") return;
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(this.sections));
		} catch {
			// ignore quota
		}
	}

	toggleSection(id: SectionId) {
		const target = this.sections.find((s) => s.id === id);
		if (target) {
			target.enabled = !target.enabled;
			this.persist();
		}
	}

	setSectionWidth(id: SectionId, width: "full" | "half") {
		const target = this.sections.find((s) => s.id === id);
		if (target) {
			target.width = width;
			this.persist();
		}
	}

	moveSection(fromIndex: number, toIndex: number) {
		if (fromIndex < 0 || fromIndex >= this.sections.length) return;
		if (toIndex < 0 || toIndex >= this.sections.length) return;
		const clone = [...this.sections];
		const [moved] = clone.splice(fromIndex, 1);
		clone.splice(toIndex, 0, moved);
		this.sections = clone;
		this.persist();
	}

	moveUp(id: SectionId) {
		const idx = this.sections.findIndex((s) => s.id === id);
		if (idx > 0) {
			this.moveSection(idx, idx - 1);
		}
	}

	moveDown(id: SectionId) {
		const idx = this.sections.findIndex((s) => s.id === id);
		if (idx >= 0 && idx < this.sections.length - 1) {
			this.moveSection(idx, idx + 1);
		}
	}

	resetToDefaults() {
		this.sections = DEFAULT_SECTIONS.map((s) => ({ ...s }));
		this.persist();
	}

	reorderSections(newSections: LayoutSectionItem[]) {
		this.sections = newSections;
		this.persist();
	}

	applyPreset(preset: LayoutPreset) {
		const clone = DEFAULT_SECTIONS.map((s) => ({ ...s }));
		if (preset === "gamer") {
			const order: SectionId[] = ["hero", "quickInstances", "favoriteServer", "gamingStats", "curatedPacks", "newsFeed"];
			this.sections = order.map((id) => {
				const s = clone.find((c) => c.id === id)!;
				if (id === "gamingStats") s.width = "half";
				if (id === "curatedPacks") { s.enabled = false; }
				return s;
			});
		} else if (preset === "compact") {
			this.sections = clone.map((s) => ({
				...s,
				enabled: s.id === "hero" || s.id === "quickInstances",
				width: "full" as const,
			}));
		} else {
			this.sections = clone.map((s) => ({ ...s, enabled: true }));
		}
		this.persist();
	}
}

export const layoutStore = new LayoutStore();
