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
	useGamemode?: boolean;
	useMangohud?: boolean;
	forceDedicatedGpu?: boolean;
	useGamescope?: boolean;
	gamescopeWidth?: number | null;
	gamescopeHeight?: number | null;
	gamescopeFsr?: boolean;
	forceFullVerification?: boolean;
	preLaunchHook?: string | null;
	postExitHook?: string | null;
	banner?: string;
}

function isRenderableBanner(value: string): boolean {
	if (!value) return false;
	if (value.startsWith("/")) return true;
	try {
		const protocol = new URL(value).protocol;
		return protocol === "https:" || protocol === "asset:";
	} catch {
		return false;
	}
}

export function isSafeBannerUrl(value: string): boolean {
	if (!value) return true;
	try {
		return new URL(value).protocol === "https:";
	} catch {
		return false;
	}
}

function savedBanner(id: string): string | undefined {
    if (typeof localStorage === "undefined") return undefined;
    try {
        const stored = localStorage.getItem(`luxmc_banner_${id}`);
        if (!stored) return undefined;
        if (!isRenderableBanner(stored)) {
            localStorage.removeItem(`luxmc_banner_${id}`);
            return undefined;
        }
        return stored;
    } catch { return undefined; }
}

function createProfileStore() {
	let list = $state<Profile[]>([]);
	let activeId = $state<string | null>(null);
	const artworkAttempts = new Map<string, number>();
	const defaultIcons = new Set(["", "default", "grass", "grass_block", "/grass_block.png"]);
	async function restoreArtwork() {
		const { modsSearch } = await import("$lib/api/mods");
		const { profilesUpdate } = await import("$lib/api/instances");
		for (const profile of list) {
			if (profile.loader === "vanilla" || !defaultIcons.has(profile.icon) || (artworkAttempts.get(profile.id) ?? 0) > Date.now()) continue;
			artworkAttempts.set(profile.id, Date.now() + 300_000);
			try {
				const matches = await modsSearch(profile.name, "", 10, 0, "modpack");
				const match = matches.find(item => item.title.trim().toLowerCase() === profile.name.trim().toLowerCase() && item.iconUrl?.startsWith("https://"));
				const current = list.find(item => item.id === profile.id);
				if (!match?.iconUrl || !current || current.name !== profile.name || !defaultIcons.has(current.icon)) continue;
				await profilesUpdate({ id: profile.id, icon: match.iconUrl });
				list = list.map(item => item.id === profile.id && defaultIcons.has(item.icon) ? { ...item, icon: match.iconUrl! } : item);
			} catch {}
		}
	}

	return {
		get list() {
			return list;
		},
		get activeId() {
			return activeId;
		},
		set list(next: Profile[]) {
			list = next.map(profile => {
				const banner = savedBanner(profile.id) || profile.banner;
				return { ...profile, banner: banner && isRenderableBanner(banner) ? banner : undefined };
			});
			if (typeof window !== "undefined") void restoreArtwork().catch(() => {});
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
		setBanner(id: string, value: string): boolean {
            const trimmed = value.trim();
            if (trimmed && !isSafeBannerUrl(trimmed)) return false;
            try {
                if (trimmed) localStorage.setItem(`luxmc_banner_${id}`, trimmed);
                else localStorage.removeItem(`luxmc_banner_${id}`);
            } catch {}
            list = list.map(profile => profile.id === id ? { ...profile, banner: trimmed || undefined } : profile);
            return true;
        },
        async refresh() {
			try {
				const { profilesList } = await import("$lib/api/instances");
				const rows = await profilesList();
				const existing = new Map(list.map((p) => [p.id, p] as const));
				list = rows.map((p) => ({
					...existing.get(p.id),
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
					useGamemode: p.useGamemode, useMangohud: p.useMangohud,
					forceDedicatedGpu: p.forceDedicatedGpu, useGamescope: p.useGamescope,
					gamescopeWidth: p.gamescopeWidth, gamescopeHeight: p.gamescopeHeight,
					gamescopeFsr: p.gamescopeFsr, forceFullVerification: p.forceFullVerification,
					preLaunchHook: p.preLaunchHook ?? null, postExitHook: p.postExitHook ?? null,
					notes: p.notes ?? undefined, group: p.instanceGroup ?? undefined,
					lastPlayed: p.lastPlayed ? new Date(p.lastPlayed).getTime() : undefined, launchCount: p.launchCount,
				}));
				void restoreArtwork().catch(() => {});
			} catch {
			}
		}
	};
}

export const profiles = createProfileStore();
