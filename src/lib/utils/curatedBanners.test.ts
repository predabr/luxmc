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

	it("resolves official curated banner for Kimetsu no Yaiba / Demon Slayer", () => {
		const banner = resolveProfileBanner({
			name: "Kimetsu no Yaiba (Demon-Slayer)"
		});
		expect(banner).toContain("BH_KnYDS_header.webp");
	});

	it("resolves official curated banner for Vulkan Optimized", () => {
		const banner = resolveProfileBanner({
			name: "Vulkan Optimized"
		});
		expect(banner).toContain("VulkanMod");
	});

	it("resolves official curated banner for Sunlit Valley", () => {
		const banner = resolveProfileBanner({
			name: "Society: Sunlit Valley"
		});
		expect(banner).toContain("forgecdn.net");
	});

	it("resolves official curated banner for RLCraft", () => {
		const banner = resolveProfileBanner({
			name: "RLCraft"
		});
		expect(banner).toContain("BH_RL_Header.webp");
	});

	it("resolves preset modpack banners", () => {
		expect(resolveProfileBanner({ name: "My FO Pack", icon: "modpack_fo" })).toBe("/modpack_fo.webp");
		expect(resolveProfileBanner({ name: "My Cobblemon", icon: "modpack_cobblemon" })).toBe("/modpack_cobblemon.webp");
		expect(resolveProfileBanner({ name: "My Better MC", icon: "modpack_better_mc" })).toBe("/modpack_better_mc.webp");
	});

	it("falls back to remote icon URL if no banner matches", () => {
		expect(resolveProfileBanner({ name: "Custom Pack", icon: "https://example.com/pack.png" })).toBe("https://example.com/pack.png");
	});

	it("resolves vanilla banner for vanilla instances", () => {
		expect(resolveProfileBanner({ name: "Survival", loader: "vanilla" })).toBe("/vanilla_banner.png");
	});

	it("uses the Vanilla Perfected artwork instead of the vanilla fallback", () => {
		expect(resolveProfileBanner({ name: "Vanilla Perfected", loader: "fabric", icon: "grass_block" })).toBe("/modpack_vanilla_perfected.png");
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
