export interface Profile {
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
	loaderVersion?: string;
	javaPath?: string;
	jvmArgs?: string;
	resolution?: { width: number; height: number; fullscreen: boolean };
	gameDir: string;
	createdAt: number;
	updatedAt: number;
	favorite?: boolean;
	group?: string;
	notes?: string;
	lastPlayed?: number;
	launchCount?: number;
	modCount?: number;
	diskUsage?: number;
	ramMb?: number;
	autoOptimize?: boolean;
	useVulkan?: boolean;
}

function createProfileStore() {
	let list = $state<Profile[]>([]);
	let activeId = $state<string | null>(null);

	return {
		get list() {
			return list;
		},
		get activeId() {
			return activeId;
		},
		set list(next: Profile[]) {
			list = next;
		},
		set activeId(id: string | null) {
			activeId = id;
		},
		get active(): Profile | null {
			return list.find((p) => p.id === activeId) ?? null;
		},
		add(p: Profile) {
			list = [...list, p];
		},
		update(id: string, patch: Partial<Profile>) {
			list = list.map((p) =>
				p.id === id ? { ...p, ...patch, updatedAt: Date.now() } : p
			);
		},
		remove(id: string) {
			list = list.filter((p) => p.id !== id);
			if (activeId === id) activeId = list[0]?.id ?? null;
		},
		toggleFavorite(id: string) {
			list = list.map((p) =>
				p.id === id ? { ...p, favorite: !p.favorite } : p
			);
		},
		setLastPlayed(id: string) {
			list = list.map((p) =>
				p.id === id ? { ...p, lastPlayed: Date.now() } : p
			);
		}
	};
}

export const profiles = createProfileStore();
