export interface Profile {
	id: string;
	name: string;
	icon: string;
	mcVersion: string;
	loader: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
	loaderVersion?: string;
	javaPath?: string | null;
	jvmArgs?: string | null;
	resolution?: { width: number; height: number; fullscreen: boolean };
	resolutionW?: number | null;
	resolutionH?: number | null;
	fullscreen?: boolean;
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
	banner?: string;
}

function savedBanner(id: string): string | undefined {
    if (typeof localStorage === "undefined") return undefined;
    try { return localStorage.getItem(`luxmc_banner_${id}`) || undefined; } catch { return undefined; }
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
			list = next.map(profile => ({ ...profile, banner: savedBanner(profile.id) || profile.banner }));
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
		},
		setBanner(id: string, value: string) {
            if (value && new URL(value).protocol !== "https:") throw new Error("Use uma imagem HTTPS.");
            if (value) localStorage.setItem(`luxmc_banner_${id}`, value);
            else localStorage.removeItem(`luxmc_banner_${id}`);
            list = list.map(profile => profile.id === id ? { ...profile, banner: value || undefined } : profile);
        },
        async refresh() {
			try {
				const { profilesList } = await import("$lib/api/instances");
				const rows = await profilesList();
				list = rows.map((p) => ({
					...list.find(existing => existing.id === p.id),
					id: p.id,
                    banner: savedBanner(p.id),
					name: p.name,
					icon: p.icon,
					mcVersion: p.mcVersion,
					loader: p.loader as "vanilla" | "fabric" | "forge" | "neoforge" | "quilt",
					loaderVersion: p.loaderVersion ?? undefined,
					javaPath: p.javaPath ?? undefined,
					jvmArgs: p.jvmArgs ?? undefined,
					resolution: p.resolutionW && p.resolutionH ? { width: p.resolutionW, height: p.resolutionH, fullscreen: p.fullscreen } : undefined,
					gameDir: p.gameDir,
					createdAt: new Date(p.createdAt).getTime(),
					updatedAt: new Date(p.updatedAt).getTime(),
					favorite: p.favorite ?? false,
					ramMb: p.ramMb ?? undefined, modCount: p.modCount, diskUsage: p.diskUsage,
					autoOptimize: p.autoOptimize, useVulkan: p.useVulkan,
					notes: p.notes ?? undefined, group: p.instanceGroup ?? undefined,
					lastPlayed: p.lastPlayed ? new Date(p.lastPlayed).getTime() : undefined, launchCount: p.launchCount,
				}));
			} catch {
			}
		}
	};
}

export const profiles = createProfileStore();
