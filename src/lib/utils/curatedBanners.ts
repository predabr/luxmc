export const curatedBanners: Record<string, string> = {
	"kimetsu": "https://cdn.modrinth.com/data/8jHBWQlK/images/1ca292dc6ecd81cbb25ee090a4743c7f047cdaa0.png",
	"demon slayer": "https://cdn.modrinth.com/data/8jHBWQlK/images/1ca292dc6ecd81cbb25ee090a4743c7f047cdaa0.png",
	"demon-slayer": "https://cdn.modrinth.com/data/8jHBWQlK/images/1ca292dc6ecd81cbb25ee090a4743c7f047cdaa0.png",
	"vulkan optimized": "https://cdn.modrinth.com/data/u76hn1pF/images/b4f944e79a18071d4e7fb855a2315d0db65b342f.png",
	"vulkan": "https://cdn.modrinth.com/data/u76hn1pF/images/b4f944e79a18071d4e7fb855a2315d0db65b342f.png",
	"sunlit valley": "https://cdn.modrinth.com/data/FpghCeHO/images/6710936da674a87ac0f5e337026a839cb5b49eb6.png",
	"society": "https://cdn.modrinth.com/data/FpghCeHO/images/6710936da674a87ac0f5e337026a839cb5b49eb6.png",
	"rlcraft": "https://media.forgecdn.net/attachments/267/928/rlcraft-banner.png",
	"skyfactory": "https://media.forgecdn.net/attachments/258/182/skyfactory4_banner.png",
	"all the mods": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
	"atm11": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
	"atm10": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
	"atm9": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
	"atm": "https://media.forgecdn.net/attachments/636/123/atm10_banner.png",
	"better mc": "/modpack_better_mc.webp",
	"better minecraft": "/modpack_better_mc.webp",
	"bmc": "/modpack_better_mc.webp",
	"cobblemon": "/modpack_cobblemon.webp",
	"pixelmon": "https://media.forgecdn.net/attachments/305/760/banner.png",
	"fabulously optimized": "/modpack_fo.webp",
	"fo": "/modpack_fo.webp",
	"dawncraft": "https://media.forgecdn.net/attachments/474/883/banner.png",
	"medieval mc": "https://media.forgecdn.net/attachments/418/850/banner.png",
	"prominence": "https://media.forgecdn.net/attachments/625/441/prominence_banner.png",
	"vault hunters": "https://media.forgecdn.net/attachments/420/55/vh3_banner.png",
	"simply optimized": "https://cdn.modrinth.com/data/bKUGbHwI/images/b07cbde1b24bf20a320ff7b57b98df9ad01b54a7.png",
	"create astral": "https://media.forgecdn.net/attachments/514/235/banner.png",
	"create:": "https://media.forgecdn.net/attachments/514/235/banner.png",
	"enigmatica": "https://media.forgecdn.net/attachments/354/121/banner.png",
	"stoneblock": "https://media.forgecdn.net/attachments/246/388/banner.png",
	"roguelike adventures": "https://media.forgecdn.net/attachments/267/39/banner.png",
	"rad": "https://media.forgecdn.net/attachments/267/39/banner.png",
	"sevtech": "https://media.forgecdn.net/attachments/231/139/banner.png",
	"ddss": "https://media.forgecdn.net/attachments/278/847/banner.png",
	"vanilla": "/vanilla_banner.png"
};

export function resolveProfileBanner(profile?: {
	name?: string;
	banner?: string;
	icon?: string;
	loader?: string;
} | null): string {
	if (!profile) return "/bg_day.jpg";

	if (
		profile.banner &&
		profile.banner.trim().length > 0 &&
		profile.banner !== profile.icon &&
		profile.banner !== "/grass_block.png" &&
		profile.banner !== "grass_block"
	) {
		return profile.banner;
	}

	const nameLower = (profile.name || "").toLowerCase();
	for (const [key, bannerUrl] of Object.entries(curatedBanners)) {
		if (nameLower.includes(key)) {
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

	if (profile.loader === "vanilla" || nameLower.includes("vanilla")) {
		return "/vanilla_banner.png";
	}

	return "/bg_day.jpg";
}
