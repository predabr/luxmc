import { describe, expect, it } from "vitest";
import { resolveUpdateAssetUrl, type UpdateAsset } from "./updaterAssets";

const assets: UpdateAsset[] = [
	{ name: "Luxmc_2.0.2_amd64.AppImage", browser_download_url: "appimage", size: 1 },
	{ name: "luxmc-2.0.2-1-x86_64.pkg.tar.zst", browser_download_url: "arch", size: 1 },
	{ name: "Luxmc_2.0.2_amd64.deb", browser_download_url: "deb", size: 1 },
	{ name: "Luxmc-2.0.2-1.x86_64.rpm", browser_download_url: "rpm", size: 1 },
	{ name: "Luxmc_2.0.2_x64-setup.exe", browser_download_url: "exe", size: 1 },
	{ name: "Luxmc_2.0.2_universal.dmg", browser_download_url: "dmg", size: 1 }
];

describe("resolveUpdateAssetUrl", () => {
	it.each([
		["appimage", "appimage"],
		["pacman", "arch"],
		["debian", "deb"],
		["rpm", "rpm"],
		["windows", "exe"],
		["macos", "dmg"]
	] as const)("selects the %s package", (mode, expected) => {
		expect(resolveUpdateAssetUrl(assets, mode)).toBe(expected);
	});

	it("requires manual download for unknown system installations", () => {
		expect(resolveUpdateAssetUrl(assets, "system")).toBe("");
		expect(resolveUpdateAssetUrl(assets, "manual")).toBe("");
	});
	it("selects the stable installer instead of launching a portable executable", () => {
		const windows: UpdateAsset[] = [
			{ name: "Luxmc-x64.exe", browser_download_url: "portable", size: 1 },
			{ name: "Lux MC Launcher.exe", browser_download_url: "installer", size: 1 }
		];
		expect(resolveUpdateAssetUrl(windows, "windows")).toBe("installer");
		expect(resolveUpdateAssetUrl(windows.slice(0, 1), "windows")).toBe("");
	});
});
