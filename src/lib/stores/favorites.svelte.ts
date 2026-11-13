import type { ModSearchResultItem } from '$lib/api/types';

export interface FavoriteEntry extends ModSearchResultItem {
	contentType: string;
	savedAt: number;
}

const STORAGE_KEY = 'luxmc_catalog_favorites';
const MAX_FAVORITES = 300;

function load(): FavoriteEntry[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];
		return parsed.filter((entry) => entry && typeof entry.sourceId === 'string' && typeof entry.title === 'string');
	} catch {
		return [];
	}
}

class FavoritesStore {
	items = $state<FavoriteEntry[]>(load());

	#persist() {
		try {
			localStorage.setItem(STORAGE_KEY, JSON.stringify(this.items.slice(0, MAX_FAVORITES)));
		} catch {
			// Sem armazenamento disponível: os favoritos permanecem apenas nesta sessão.
		}
	}

	key(source: string, sourceId: string): string {
		return `${source}:${sourceId}`;
	}

	has(source: string, sourceId: string): boolean {
		return this.items.some((entry) => entry.source === source && entry.sourceId === sourceId);
	}

	forType(contentType: string): FavoriteEntry[] {
		return this.items.filter((entry) => entry.contentType === contentType);
	}

	toggle(item: ModSearchResultItem, contentType: string): boolean {
		const existing = this.items.find((entry) => entry.source === item.source && entry.sourceId === item.sourceId);
		if (existing) {
			this.items = this.items.filter((entry) => entry !== existing);
			this.#persist();
			return false;
		}
		const entry: FavoriteEntry = {
			slug: item.slug,
			title: item.title,
			description: item.description,
			downloads: item.downloads,
			iconUrl: item.iconUrl,
			bannerUrl: item.bannerUrl,
			author: item.author,
			categories: item.categories,
			versions: item.versions,
			source: item.source,
			sourceId: item.sourceId,
			contentType,
			savedAt: Date.now()
		};
		this.items = [entry, ...this.items].slice(0, MAX_FAVORITES);
		this.#persist();
		return true;
	}
}

export const favorites = new FavoritesStore();
