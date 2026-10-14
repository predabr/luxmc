export function isRenderableBanner(value: string): boolean {
	if (!value) return false;
	if (value.startsWith("/")) return true;
	try {
		const protocol = new URL(value).protocol;
		return protocol === "https:" || protocol === "asset:";
	} catch {
		return false;
	}
}

export function isProjectArtwork(image: { url: string; title?: string | null; description?: string | null; link?: string }): boolean {
	if (!isRenderableBanner(image.url)) return false;
	const metadata = `${image.title || ''} ${image.description || ''} ${image.link || ''}`;
	return !/(?:nodecraft|promocard|donation|sponsor|badge|\/partners\/|shields\.io|_important|_summary|_features)/i.test(image.url)
		&& !/(?:\b(?:sponsor(?:ed)?|partner(?:ship)?|advertisement|discount|coupon|donate|donation|hosting|patrocin(?:ador|ado)|desconto|cupom)\b|nodecraft|bisecthosting|apexhosting|server\.pro)/i.test(metadata);
}

export const curatedBanners: Record<string, string> = {
	"vanilla perfected": "/modpack_vanilla_perfected.png",
	"kimetsu": "https://www.bisecthosting.com/images/CF/Kimetsu_no_Yaiba/BH_KnYDS_header.webp",
	"demon slayer": "https://www.bisecthosting.com/images/CF/Kimetsu_no_Yaiba/BH_KnYDS_header.webp",
	"demon-slayer": "https://www.bisecthosting.com/images/CF/Kimetsu_no_Yaiba/BH_KnYDS_header.webp",
	"vulkan optimized": "https://opengraph.githubassets.com/1/xCollateral/VulkanMod",
	"vulkan": "https://opengraph.githubassets.com/1/xCollateral/VulkanMod",
	"sunlit valley": "https://media.forgecdn.net/attachments/description/1093382/description_fecbf4ad-b78d-40b0-a02d-3b9301f78884.png",
	"society": "https://media.forgecdn.net/attachments/description/1093382/description_fecbf4ad-b78d-40b0-a02d-3b9301f78884.png",
	"rlcraft": "https://www.bisecthosting.com/images/CF/RLCraft/BH_RL_Header.webp",
	"all the mods": "https://media.forgecdn.net/avatars/1182/438/638755918649288941.png",
	"atm11": "https://media.forgecdn.net/avatars/1182/438/638755918649288941.png",
	"atm10": "https://media.forgecdn.net/avatars/1182/438/638755918649288941.png",
	"atm9": "https://media.forgecdn.net/avatars/1182/438/638755918649288941.png",
	"atm": "https://media.forgecdn.net/avatars/1182/438/638755918649288941.png",
	"better mc": "/modpack_better_mc.webp",
	"better minecraft": "/modpack_better_mc.webp",
	"bmc": "/modpack_better_mc.webp",
	"cobblemon": "/modpack_cobblemon.webp",
	"fabulously optimized": "/modpack_fo.webp",
	"fo": "/modpack_fo.webp",
	"simply optimized": "https://cdn.modrinth.com/data/1KVo5zza/images/044a5b9cb85394be95656a1ebcfe135466420da9.png",
	"prominence": "https://media.forgecdn.net/avatars/1969/863/639217114177376898.webp",
	"deceasedcraft": "https://media.forgecdn.net/avatars/1577/605/639022081315858417.png",
};

export function matchesCuratedBanner(name: string, key: string): boolean {
	const lower = name.toLowerCase();
	if (key.length > 3) return lower.includes(key);
	return lower.split(/[^a-z0-9]+/).includes(key);
}

export function matchesCatalogBanner(name: string, slug: string, key: string): boolean {
	const normalize = (value: string) => value.toLowerCase().trim().replace(/[^a-z0-9]+/g, " ").trim();
	return normalize(slug || name) === normalize(key);
}

export function projectLandscape(item: { source?: string; sourceId?: string; iconUrl?: string | null }): string | null {
	const presets: Record<string, string> = {
		modpack_fo: "/modpack_fo.webp",
		"/modpack_fo_icon.png": "/modpack_fo.webp",
		modpack_better_mc: "/modpack_better_mc.webp",
		"/modpack_bmc_icon.webp": "/modpack_better_mc.webp",
		modpack_cobblemon: "/modpack_cobblemon.webp",
		"/modpack_cobblemon_icon.png": "/modpack_cobblemon.webp",
		"/modpack_vanilla_perfected_icon.webp": "/modpack_vanilla_perfected.png"
	};
	if (!item.source && !item.sourceId && item.iconUrl && presets[item.iconUrl]) return presets[item.iconUrl];
	if ((item.source === "modrinth" && item.sourceId === "1KVo5zza") || (item.source === "curseforge" && item.sourceId === "396246")) return "/modpack_fo.webp";
	try {
		const url = new URL(item.iconUrl || "");
		if (url.hostname === "cdn.modrinth.com" && /^\/data\/1KVo5zza\//.test(url.pathname)) return "/modpack_fo.webp";
	} catch {}
	return null;
}

export function catalogBannerCandidates(item: { title: string; slug: string; source?: string; sourceId?: string; bannerUrl?: string | null; iconUrl?: string | null; galleryUrls?: string[] }): string[] {
	const candidates: string[] = [];
	const landscape = projectLandscape(item);
	if (landscape) candidates.push(landscape);
	for (const url of item.galleryUrls || []) if (url !== item.iconUrl && isProjectArtwork({ url }) && !candidates.includes(url)) candidates.push(url);
	if (item.bannerUrl && item.bannerUrl !== item.iconUrl && isProjectArtwork({ url: item.bannerUrl }) && !candidates.includes(item.bannerUrl)) candidates.push(item.bannerUrl);
	if (item.iconUrl && isRenderableBanner(item.iconUrl) && !candidates.includes(item.iconUrl)) candidates.push(item.iconUrl);
	return candidates;
}

export function resolveProfileBanner(profile?: {
	name?: string;
	banner?: string;
	icon?: string;
	loader?: string;
} | null): string {
	if (!profile) return "/bg_day.jpg";

	if (
		profile.banner &&
		isRenderableBanner(profile.banner.trim()) &&
		profile.banner.trim().length > 0 &&
		profile.banner !== profile.icon &&
		profile.banner !== "/grass_block.png" &&
		profile.banner !== "grass_block"
	) {
		return profile.banner;
	}

	if (profile.icon) {
		const landscape = projectLandscape({ iconUrl: profile.icon });
		if (landscape) return landscape;
		if (
			profile.icon.startsWith("http://") ||
			profile.icon.startsWith("https://") ||
			profile.icon.startsWith("/modpack_")
		) {
			return profile.icon;
		}
	}

	if (profile.loader === "vanilla") {
		return "/vanilla_banner.png";
	}

	return "/bg_day.jpg";
}
