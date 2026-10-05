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

	const nameLower = (profile.name || "").toLowerCase();
	for (const [key, bannerUrl] of Object.entries(curatedBanners)) {
		if (matchesCuratedBanner(nameLower, key)) {
			return bannerUrl;
		}
	}

	if (profile.icon) {
		if (profile.icon === "modpack_better_mc") return "/modpack_better_mc.webp";
		if (profile.icon === "modpack_cobblemon") return "/modpack_cobblemon.webp";
		if (profile.icon === "modpack_fo") return "/modpack_fo.webp";
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
