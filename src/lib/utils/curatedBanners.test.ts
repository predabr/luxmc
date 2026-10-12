import { describe, expect, it } from "vitest";
import { resolveProfileBanner } from "./curatedBanners";

describe("resolveProfileBanner", () => {
	it("prioritizes dedicated custom banner when provided", () => {
		const banner = resolveProfileBanner({
			name: "Kimetsu no Yaiba",
			banner: "https://example.com/custom.png"
		});
		expect(banner).toBe("https://example.com/custom.png");
	});

	it("does not infer a project cover from its name: Kimetsu no Yaiba / Demon Slayer", () => {
		const banner = resolveProfileBanner({
			name: "Kimetsu no Yaiba (Demon-Slayer)"
		});
		expect(banner).toBe("/bg_day.jpg");
	});

	it("does not infer a project cover from its name: Vulkan Optimized", () => {
		const banner = resolveProfileBanner({
			name: "Vulkan Optimized"
		});
		expect(banner).toBe("/bg_day.jpg");
	});

	it("does not infer a project cover from its name: Sunlit Valley", () => {
		const banner = resolveProfileBanner({
			name: "Society: Sunlit Valley"
		});
		expect(banner).toBe("/bg_day.jpg");
	});

	it("does not infer a project cover from its name: RLCraft", () => {
		const banner = resolveProfileBanner({
			name: "RLCraft"
		});
		expect(banner).toBe("/bg_day.jpg");
	});

	it("resolves preset modpack banners", () => {
		expect(resolveProfileBanner({ name: "My FO Pack", icon: "modpack_fo" })).toBe("/modpack_fo.webp");
		expect(resolveProfileBanner({ name: "My Cobblemon", icon: "modpack_cobblemon" })).toBe("/modpack_cobblemon.webp");
		expect(resolveProfileBanner({ name: "My Better MC", icon: "modpack_better_mc" })).toBe("/modpack_better_mc.webp");
		expect(resolveProfileBanner({ name: "Renamed Vanilla Perfected", icon: "/modpack_vanilla_perfected_icon.webp" })).toBe("/modpack_vanilla_perfected.png");
	});

	it("falls back to remote icon URL if no banner matches", () => {
		expect(resolveProfileBanner({ name: "Custom Pack", icon: "https://example.com/pack.png" })).toBe("https://example.com/pack.png");
	});

	it("resolves vanilla banner for vanilla instances", () => {
		expect(resolveProfileBanner({ name: "Survival", loader: "vanilla" })).toBe("/vanilla_banner.png");
	});

	it("does not assign a named modpack cover without provider metadata", () => {
		expect(resolveProfileBanner({ name: "Vanilla Perfected", loader: "fabric", icon: "grass_block" })).toBe("/bg_day.jpg");
	});

	it("preserves artwork for modpacks whose names contain vanilla", () => {
		expect(resolveProfileBanner({ name: "Vanilla Adventure", loader: "fabric", icon: "https://example.com/pack.webp" })).toBe("https://example.com/pack.webp");
		expect(resolveProfileBanner({ name: "Vanilla Adventure", loader: "fabric" })).toBe("/bg_day.jpg");
	});

	it("defaults to bg_day.jpg when nothing is specified", () => {
		expect(resolveProfileBanner(null)).toBe("/bg_day.jpg");
		expect(resolveProfileBanner({ name: "Unknown" })).toBe("/bg_day.jpg");
		expect(resolveProfileBanner({ name: "Forge adventure pack" })).toBe("/bg_day.jpg");
	});
});
