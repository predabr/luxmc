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

	it("resolves curated banner for Kimetsu no Yaiba / Demon Slayer", () => {
		const banner = resolveProfileBanner({
			name: "Kimetsu no Yaiba (Demon-Slayer)"
		});
		expect(banner).toContain("cdn.modrinth.com");
		expect(banner).toContain("1ca292dc6ecd81cbb25ee090a4743c7f047cdaa0");
	});

	it("resolves curated banner for Vulkan Optimized", () => {
		const banner = resolveProfileBanner({
			name: "Vulkan Optimized"
		});
		expect(banner).toContain("cdn.modrinth.com");
	});

	it("resolves curated banner for Sunlit Valley", () => {
		const banner = resolveProfileBanner({
			name: "Society: Sunlit Valley"
		});
		expect(banner).toContain("cdn.modrinth.com");
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

	it("defaults to bg_day.jpg when nothing is specified", () => {
		expect(resolveProfileBanner(null)).toBe("/bg_day.jpg");
		expect(resolveProfileBanner({ name: "Unknown" })).toBe("/bg_day.jpg");
	});
});
